//! Applying the effects of a tick: delivered messages, gifts, treatments,
//! attacks on fauna, and the outcome of every conversation.

use super::chronicle_text::line;
use super::{Effects, Interaction, Sim};
use crate::agents::beliefs::Bond;
use crate::agents::messages::{Message, MessageKind};
use crate::agents::plans::Topic;
use crate::agents::Agent;
use crate::colony::chronicle::Category;
use crate::data::{game_data, Need, Resource};
use crate::world::{AgentId, Calendar};

/// How well two survivors' traits fit, from `a`'s point of view.
pub fn compatibility(a: &Agent, b: &Agent) -> f32 {
    let social = &game_data().balance.social;
    let mut score = 0.0;
    for trait_id in &a.traits {
        let Some(def) = game_data().trait_def(trait_id) else {
            continue;
        };
        for other in &b.traits {
            if def.bonds_with.contains(other) {
                score += social.trait_bond_bonus;
            }
            if def.clashes_with.contains(other) {
                score -= social.trait_clash_penalty;
            }
        }
    }
    score
}

fn pair_mut(agents: &mut [Agent], a: AgentId, b: AgentId) -> Option<(&mut Agent, &mut Agent)> {
    let ia = agents.iter().position(|x| x.id == a)?;
    let ib = agents.iter().position(|x| x.id == b)?;
    if ia == ib {
        return None;
    }
    if ia < ib {
        let (left, right) = agents.split_at_mut(ib);
        Some((&mut left[ia], &mut right[0]))
    } else {
        let (left, right) = agents.split_at_mut(ia);
        Some((&mut right[0], &mut left[ib]))
    }
}

struct Outcome {
    milestones: Vec<(String, Vec<AgentId>, Category, u8)>,
    messages: Vec<Message>,
}

fn note_bond_change(before: Bond, after: Bond, a: &Agent, b: &Agent, tick: u64, out: &mut Outcome) {
    if before == after {
        return;
    }
    let key = match after {
        Bond::Friend if before != Bond::CloseFriend => "became_friends",
        Bond::CloseFriend => "became_close_friends",
        Bond::Rival if before != Bond::Enemy => "became_rivals",
        Bond::Enemy => "became_enemies",
        _ => return,
    };
    let text = line(key, tick, &[("a", &a.given_name), ("b", &b.given_name)]);
    out.milestones
        .push((text, vec![a.id, b.id], Category::Relationship, 1));
}

fn small_talk(
    a: &mut Agent,
    b: &mut Agent,
    rng: &mut macroquad_toolkit::rng::SeededRng,
    tick: u64,
    out: &mut Outcome,
) {
    let social = &game_data().balance.social;
    let range = social.chat_opinion_range;
    let mood = (a.mood + b.mood - 100.0) / 50.0 * 2.0;
    // A hot temper or a clash of traits can turn small talk into an argument.
    let friction = (a.personality.temper + b.personality.temper).max(0.0) * 0.25
        - compatibility(a, b).min(0.0) * 0.02;
    if rng.chance(friction.min(0.45)) {
        insult(a, b, rng, tick, out);
        return;
    }
    let delta_a = rng.range_f32(range[0], range[1]) + compatibility(a, b) * 0.8 + mood;
    let delta_b = rng.range_f32(range[0], range[1]) + compatibility(b, a) * 0.8 + mood;
    a.beliefs.relation_mut(b.id).change_opinion(delta_a);
    b.beliefs.relation_mut(a.id).change_opinion(delta_b);
    let (a_id, b_id) = (a.id, b.id);
    for (x, y) in [(&mut *a, b_id), (&mut *b, a_id)] {
        let relation = x.beliefs.relation_mut(y);
        relation.familiarity = (relation.familiarity + social.familiarity_per_chat).min(100.0);
    }
    b.needs.change(Need::Social, social.chat_social_gain * 0.8);
    if delta_a > 3.0 {
        a.mind
            .add("good_chat", Some(b.id), Calendar::ticks_per_day());
    }
    if delta_b > 3.0 {
        b.mind
            .add("good_chat", Some(a.id), Calendar::ticks_per_day());
    }
    share_gossip(a, b, rng, tick, out);
    share_knowledge(a, b, rng, tick, out);
}

/// Speakers pass on their strongest feeling about someone else.
fn share_gossip(
    a: &Agent,
    b: &Agent,
    rng: &mut macroquad_toolkit::rng::SeededRng,
    tick: u64,
    out: &mut Outcome,
) {
    if !rng.chance(0.35) {
        return;
    }
    let strongest = a
        .beliefs
        .relations
        .iter()
        .filter(|(id, _)| **id != b.id)
        .max_by(|x, y| x.1.opinion.abs().total_cmp(&y.1.opinion.abs()));
    if let Some((about, relation)) = strongest {
        out.messages.push(Message {
            from: a.id,
            to: b.id,
            kind: MessageKind::Gossip {
                about: *about,
                opinion: relation.opinion,
            },
            tick,
        });
    }
}

/// Conversation spreads beliefs: where a useful grove or seam lies.
fn share_knowledge(
    a: &Agent,
    b: &Agent,
    rng: &mut macroquad_toolkit::rng::SeededRng,
    tick: u64,
    out: &mut Outcome,
) {
    if !rng.chance(0.5) {
        return;
    }
    let unknown: Vec<_> = a
        .beliefs
        .nodes
        .iter()
        .filter(|(id, belief)| belief.amount > 1.0 && !b.beliefs.nodes.contains_key(id))
        .collect();
    if let Some((node, belief)) = rng.choose(&unknown) {
        out.messages.push(Message {
            from: a.id,
            to: b.id,
            kind: MessageKind::InformNode {
                node: **node,
                belief: (*belief).clone(),
            },
            tick,
        });
    }
}

fn flirt(a: &mut Agent, b: &mut Agent, tick: u64, out: &mut Outcome) {
    let social = &game_data().balance.social;
    let receptive = b.beliefs.opinion_of(a.id) > 15.0 && b.partner.is_none() && !b.is_child;
    if receptive {
        let gain = social.romance_gain_per_flirt * (1.0 + b.personality.romance * 0.5);
        b.beliefs.relation_mut(a.id).romance += gain;
        a.beliefs.relation_mut(b.id).romance += social.romance_gain_per_flirt;
        b.beliefs.relation_mut(a.id).change_opinion(2.0);
        if a.beliefs
            .relation(b.id)
            .is_some_and(|r| r.romance >= social.partner_romance_threshold)
        {
            out.messages.push(Message {
                from: a.id,
                to: b.id,
                kind: MessageKind::Propose,
                tick,
            });
        }
    } else {
        b.beliefs.relation_mut(a.id).change_opinion(-2.0);
        a.mind
            .add("frustrated", Some(b.id), Calendar::ticks_per_day());
    }
}

fn insult(
    a: &mut Agent,
    b: &mut Agent,
    rng: &mut macroquad_toolkit::rng::SeededRng,
    tick: u64,
    out: &mut Outcome,
) {
    b.beliefs.relation_mut(a.id).change_opinion(-12.0);
    a.beliefs.relation_mut(b.id).change_opinion(-3.0);
    b.mind
        .add("insulted", Some(a.id), Calendar::ticks_per_day());
    let fights = (b.personality.temper > 0.25 || a.personality.temper > 0.4) && rng.chance(0.5);
    let key = if fights {
        a.health.damage(6.0);
        b.health.damage(6.0);
        a.mind.add("fought", Some(b.id), Calendar::ticks_per_day());
        b.mind.add("fought", Some(a.id), Calendar::ticks_per_day());
        "fight"
    } else {
        "insult"
    };
    let text = line(key, tick, &[("a", &a.given_name), ("b", &b.given_name)]);
    out.milestones
        .push((text, vec![a.id, b.id], Category::Relationship, 1));
}

fn reconcile(
    a: &mut Agent,
    b: &mut Agent,
    rng: &mut macroquad_toolkit::rng::SeededRng,
    tick: u64,
    out: &mut Outcome,
) {
    let chance = 0.4 + b.personality.kindness * 0.3 - b.personality.temper * 0.3
        + (b.beliefs.opinion_of(a.id) + 50.0) / 200.0;
    if rng.chance(chance.clamp(0.05, 0.95)) {
        a.beliefs.relation_mut(b.id).change_opinion(12.0);
        b.beliefs.relation_mut(a.id).change_opinion(12.0);
        let text = line(
            "reconciled",
            tick,
            &[("a", &a.given_name), ("b", &b.given_name)],
        );
        out.milestones
            .push((text, vec![a.id, b.id], Category::Relationship, 1));
    } else {
        a.beliefs.relation_mut(b.id).change_opinion(-3.0);
    }
}

fn comfort(a: &mut Agent, b: &mut Agent) {
    b.mind
        .add("comforted", Some(a.id), Calendar::ticks_per_day());
    b.beliefs.relation_mut(a.id).change_opinion(8.0);
    a.beliefs.relation_mut(b.id).change_opinion(3.0);
    b.needs.change(Need::Social, 15.0);
    a.mind
        .add("helped_friend", Some(b.id), Calendar::ticks_per_day());
}

fn resolve(sim: &mut Sim, interaction: &Interaction, out: &mut Outcome) {
    let tick = sim.calendar.tick;
    let Sim { agents, rng, .. } = sim;
    let Some((a, b)) = pair_mut(agents, interaction.from, interaction.to) else {
        return;
    };
    let before = (
        a.beliefs
            .relation(b.id)
            .map(|r| r.bond())
            .unwrap_or(Bond::Stranger),
        b.beliefs
            .relation(a.id)
            .map(|r| r.bond())
            .unwrap_or(Bond::Stranger),
    );
    match interaction.topic {
        Topic::SmallTalk => small_talk(a, b, rng, tick, out),
        Topic::Flirt => flirt(a, b, tick, out),
        Topic::Insult => insult(a, b, rng, tick, out),
        Topic::Reconcile => reconcile(a, b, rng, tick, out),
        Topic::Comfort => comfort(a, b),
    }
    let after_a = a
        .beliefs
        .relation(b.id)
        .map(|r| r.bond())
        .unwrap_or(Bond::Stranger);
    let after_b = b
        .beliefs
        .relation(a.id)
        .map(|r| r.bond())
        .unwrap_or(Bond::Stranger);
    // Friendship is announced when both feel it; hostility when either does.
    let hostile = |bond: Bond| matches!(bond, Bond::Rival | Bond::Enemy);
    let mutual_warmth =
        after_a == after_b && !hostile(after_a) && (before.0 != after_a || before.1 != after_b);
    let a_turned_hostile = hostile(after_a) && before.0 != after_a;
    if mutual_warmth || a_turned_hostile {
        note_bond_change(before.0, after_a, a, b, tick, out);
    } else if hostile(after_b) && before.1 != after_b {
        note_bond_change(before.1, after_b, b, a, tick, out);
    }
}

fn apply_gifts(sim: &mut Sim, gifts: Vec<(AgentId, AgentId, Resource, f32)>) {
    for (from, to, resource, amount) in gifts {
        let Some((giver, receiver)) = pair_mut(&mut sim.agents, from, to) else {
            continue;
        };
        if let Some((held, held_amount)) = receiver.carrying.take() {
            sim.world.drop_item(receiver.tile, held, held_amount);
        }
        receiver.carrying = Some((resource, amount));
        receiver.beliefs.relation_mut(from).change_opinion(6.0);
        giver
            .mind
            .add("helped_friend", Some(to), Calendar::ticks_per_day());
    }
}

fn apply_treatments(sim: &mut Sim, treatments: Vec<(AgentId, AgentId, bool)>) {
    for (medic, patient, with_medicine) in treatments {
        let Some(agent) = sim.agent_mut(patient) else {
            continue;
        };
        agent.health.treated = agent.health.ill;
        agent.health.heal(if with_medicine { 20.0 } else { 8.0 });
        agent
            .mind
            .add("cared_for", Some(medic), Calendar::ticks_per_day());
        agent.beliefs.relation_mut(medic).change_opinion(5.0);
    }
}

pub fn apply_effects(sim: &mut Sim, effects: Effects) {
    let Effects {
        messages,
        interactions,
        chronicle,
        gifts,
        treatments,
        attacks,
    } = effects;
    let mut out = Outcome {
        milestones: Vec::new(),
        messages,
    };
    for interaction in &interactions {
        resolve(sim, interaction, &mut out);
    }
    apply_gifts(sim, gifts);
    apply_treatments(sim, treatments);
    for (agent, creature, damage) in attacks {
        if let Some(target) = sim.world.creatures.iter_mut().find(|c| c.id == creature) {
            target.health -= damage;
            if target.health <= 0.0 {
                if let Some(hero) = sim.agent_mut(agent) {
                    hero.stats.creatures_driven_off += 1;
                }
            }
        }
    }
    for message in out.messages {
        if let Some(recipient) = sim
            .agents
            .iter_mut()
            .find(|a| a.id == message.to && a.is_present())
        {
            recipient.inbox.push(message);
        }
    }
    for entry in chronicle {
        sim.colony.chronicle.record(entry);
    }
    for (text, agents, category, importance) in out.milestones {
        sim.record(category, importance, text, agents);
    }
}
