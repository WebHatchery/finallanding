//! Handling messages from other survivors: the receiving side of each
//! interaction protocol.

use super::chronicle_text::line;
use super::Ctx;
use crate::agents::beliefs::Bond;
use crate::agents::goals::HelpNeed;
use crate::agents::intention::DecisionKind;
use crate::agents::messages::{Message, MessageKind};
use crate::agents::Agent;
use crate::colony::chronicle::Category;
use crate::data::{fill_template, game_data, Need, Skill};

fn reply(agent: &Agent, ctx: &mut Ctx, to: u32, kind: MessageKind) {
    ctx.effects.messages.push(Message {
        from: agent.id,
        to,
        kind,
        tick: ctx.calendar.tick,
    });
}

pub fn name_of(ctx: &Ctx, id: u32) -> String {
    ctx.others
        .iter()
        .find(|s| s.id == id)
        .map(|s| s.name.clone())
        .unwrap_or_default()
}

/// Decide whether to help: kindness, how much they like the asker, and how
/// pressed they are themselves.
fn willing_to_help(agent: &Agent, from: u32, need: HelpNeed) -> bool {
    if agent.help_task.is_some() || agent.is_child {
        return false;
    }
    let opinion = agent.beliefs.opinion_of(from);
    let pressed = agent.needs.get(Need::Food) < 30.0 || agent.health.needs_care();
    if pressed {
        return false;
    }
    let skill_bonus = match need {
        HelpNeed::Care => agent.skills.level(Skill::Medicine) as f32 * 2.0,
        HelpNeed::Defence(_) => agent.personality.bravery * 30.0,
        HelpNeed::Food => 0.0,
    };
    opinion + agent.personality.kindness * 40.0 + skill_bonus > 5.0
}

fn handle_request(agent: &mut Agent, ctx: &mut Ctx, message: &Message, need: HelpNeed) {
    if willing_to_help(agent, message.from, need) {
        agent.help_task = Some((message.from, need));
        agent.next_deliberation = ctx.calendar.tick;
        reply(agent, ctx, message.from, MessageKind::AcceptHelp { need });
    } else {
        reply(agent, ctx, message.from, MessageKind::DeclineHelp { need });
    }
}

fn handle_gossip(agent: &mut Agent, from: u32, about: u32, opinion: f32) {
    if about == agent.id {
        return;
    }
    let trust = (agent.beliefs.opinion_of(from) + 50.0) / 150.0;
    let weight = game_data().balance.social.gossip_weight * trust.clamp(0.0, 1.0);
    let mine = agent.beliefs.opinion_of(about);
    agent
        .beliefs
        .relation_mut(about)
        .change_opinion((opinion - mine) * weight);
}

fn handle_proposal(agent: &mut Agent, ctx: &mut Ctx, from: u32) {
    let social = &game_data().balance.social;
    let relation = agent.beliefs.relation(from).cloned().unwrap_or_default();
    let accepts = agent.partner.is_none()
        && !agent.is_child
        && relation.opinion > 40.0
        && relation.romance > social.partner_romance_threshold * 0.7;
    if accepts {
        agent.partner = Some(from);
        agent.mind.add(
            "new_partner",
            Some(from),
            crate::world::Calendar::ticks_per_day(),
        );
        reply(agent, ctx, from, MessageKind::AcceptProposal);
        let names = [("a", agent.given_name.clone()), ("b", name_of(ctx, from))];
        let values: Vec<(&str, &str)> = names.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let text = line("partnership", ctx.calendar.tick, &values);
        ctx.log(Category::Relationship, 2, text, vec![agent.id, from]);
    } else {
        agent.beliefs.relation_mut(from).romance *= 0.7;
        reply(agent, ctx, from, MessageKind::RejectProposal);
    }
}

/// Process this tick's inbox. Returns true when a message demands an
/// immediate rethink.
pub fn process(agent: &mut Agent, ctx: &mut Ctx) -> bool {
    let messages = std::mem::take(&mut agent.inbox);
    let mut urgent = false;
    for message in messages {
        match &message.kind {
            MessageKind::InformNode { node, belief } => {
                let trusted = agent.beliefs.opinion_of(message.from) > -15.0;
                let newer = agent
                    .beliefs
                    .nodes
                    .get(node)
                    .is_none_or(|b| b.seen_tick < belief.seen_tick);
                if trusted && newer {
                    agent.beliefs.note_node(*node, belief.clone());
                }
            }
            MessageKind::Warn(threat) => {
                if agent
                    .beliefs
                    .threats
                    .iter()
                    .all(|t| t.creature != threat.creature)
                {
                    urgent = threat.dangerous;
                }
                agent.beliefs.note_threat(threat.clone());
            }
            MessageKind::RequestHelp { need, .. } => handle_request(agent, ctx, &message, *need),
            MessageKind::AcceptHelp { .. } => {
                let text = fill_template(
                    game_data().label("decision_help_coming"),
                    &[("name", &name_of(ctx, message.from))],
                );
                agent
                    .decisions
                    .push(ctx.calendar.tick, DecisionKind::Message { text });
            }
            MessageKind::DeclineHelp { .. } => {
                let text = fill_template(
                    game_data().label("decision_help_declined"),
                    &[("name", &name_of(ctx, message.from))],
                );
                agent
                    .decisions
                    .push(ctx.calendar.tick, DecisionKind::Message { text });
                agent
                    .beliefs
                    .relation_mut(message.from)
                    .change_opinion(-2.0);
            }
            MessageKind::Delivered => {}
            MessageKind::Gossip { about, opinion } => {
                handle_gossip(agent, message.from, *about, *opinion)
            }
            MessageKind::Propose => handle_proposal(agent, ctx, message.from),
            MessageKind::AcceptProposal => {
                agent.partner = Some(message.from);
                agent.mind.add(
                    "new_partner",
                    Some(message.from),
                    crate::world::Calendar::ticks_per_day(),
                );
            }
            MessageKind::RejectProposal => {
                agent.mind.add(
                    "frustrated",
                    Some(message.from),
                    crate::world::Calendar::ticks_per_day(),
                );
                let relation = agent.beliefs.relation_mut(message.from);
                relation.romance *= 0.5;
                if relation.bond() == Bond::Stranger {
                    relation.change_opinion(-3.0);
                }
            }
        }
    }
    urgent
}
