//! The survivors as Prometheus/BDI agents: situated, reactive, proactive,
//! flexible, robust and social.

use finallanding::agents::beliefs::NodeBelief;
use finallanding::agents::deliberation::Candidate;
use finallanding::agents::goals::{Goal, HelpNeed};
use finallanding::agents::intention::{DecisionKind, Intention};
use finallanding::agents::messages::{Message, MessageKind};
use finallanding::agents::plans::Topic;
use finallanding::agents::Activity;
use finallanding::colony::RunSetup;
use finallanding::data::{CreatureKind, Need, NodeKind, Resource};
use finallanding::sim::cognition::should_switch;
use finallanding::sim::{creatures, social, Effects, Interaction, Sim};
use finallanding::world::Tile;

fn colony(seed: u64) -> Sim {
    Sim::new(RunSetup {
        colony_name: "Test".into(),
        site: "verdant_basin".into(),
        difficulty: "standard".into(),
        seed,
    })
}

#[test]
fn survivors_only_know_what_they_have_perceived() {
    let mut sim = colony(3);
    sim.run_ticks(6);
    let sight = finallanding::data::game_data().balance.agents.sight_radius + 2.0;
    for agent in sim.present() {
        for belief in agent.beliefs.nodes.values() {
            let near_someone = sim
                .agents
                .iter()
                .any(|other| other.tile.distance(belief.tile) <= sight + 1.0);
            assert!(
                near_someone,
                "{} knows of a node nobody could have seen",
                agent.given_name
            );
        }
    }
    let total_nodes = sim.world.nodes.len();
    let known = sim.agents[0].beliefs.nodes.len();
    assert!(known < total_nodes, "the whole map is not known on landing");
}

#[test]
fn a_sighted_predator_interrupts_and_warns_others() {
    let mut sim = colony(5);
    sim.run_ticks(3);
    // A sleeper perceives nothing, so the watcher is someone awake.
    let awake = sim
        .present()
        .find(|a| a.activity != Activity::Sleeping)
        .expect("someone is awake at landing");
    let (watcher, tile) = (awake.id, awake.tile);
    creatures::spawn(&mut sim, CreatureKind::Ridgeback, 1);
    let spot = sim
        .world
        .map
        .nearest_walkable(tile.offset(3, 0), 3)
        .expect("open ground near the colony");
    let creature = sim.world.creatures.last_mut().expect("spawned");
    creature.tile = spot;
    creature.position = finallanding::world::Point::of_tile(spot);
    sim.run_ticks(6);
    let reacted = sim
        .agent(watcher)
        .and_then(|a| a.current_goal())
        .is_some_and(|goal| matches!(goal, Goal::Flee | Goal::Defend(_) | Goal::Help { .. }));
    assert!(reacted, "the survivor reacts to the threat");
    let warned = sim
        .present()
        .filter(|a| a.id != watcher && !a.beliefs.threats.is_empty())
        .count();
    assert!(warned > 0, "nearby survivors heard the warning");
}

#[test]
fn a_failed_plan_corrects_beliefs_and_tries_another_way() {
    let mut sim = colony(7);
    let friend = sim.agents[1].id;
    let ghost = 999_999;
    let agent = &mut sim.agents[0];
    agent.needs.set(Need::Food, 5.0);
    agent.beliefs.stock.set(Resource::Food, 0.0);
    agent.beliefs.stock.set(Resource::Meals, 0.0);
    agent.beliefs.relation_mut(friend).opinion = 60.0;
    agent.beliefs.nodes.clear();
    agent.beliefs.note_node(
        ghost,
        NodeBelief {
            kind: NodeKind::Forage,
            tile: agent.tile.offset(2, 0),
            amount: 20.0,
            seen_tick: 0,
        },
    );
    let id = agent.id;
    sim.run_ticks(20);
    let agent = sim.agent(id).expect("still here");
    let log: Vec<&DecisionKind> = agent
        .decisions
        .entries
        .iter()
        .rev()
        .map(|d| &d.kind)
        .collect();
    let failed = log
        .iter()
        .position(|k| matches!(k, DecisionKind::PlanFailed { plan, .. } if plan == "forage"));
    let failed = failed.expect("foraging a bush that does not exist fails");
    let retried = log[failed..]
        .iter()
        .any(|k| matches!(k, DecisionKind::ChosePlan { plan } if plan != "forage"));
    assert!(retried, "another plan is chosen after the failure");
    assert!(
        !agent.beliefs.nodes.contains_key(&ghost),
        "the false belief is forgotten"
    );
}

#[test]
fn committed_goals_resist_small_temptations_but_yield_to_breaks() {
    let current = Intention::new(Goal::Build(1), 0.5, 0);
    let slightly_better = Candidate {
        goal: Goal::Relax,
        utility: 0.55,
        reason: String::new(),
    };
    let much_better = Candidate {
        goal: Goal::Relax,
        utility: 0.9,
        reason: String::new(),
    };
    assert!(
        !should_switch(&current, 0.5, &slightly_better),
        "commitment keeps the survivor on task"
    );
    assert!(
        should_switch(&current, 0.5, &much_better),
        "a clearly better goal wins"
    );
    let eating = Intention::new(Goal::Eat, 0.5, 0);
    let tempting = Candidate {
        goal: Goal::Relax,
        utility: 0.8,
        reason: String::new(),
    };
    assert!(
        should_switch(&current, 0.5, &tempting),
        "work yields to a tempting break"
    );
    assert!(
        !should_switch(&eating, 0.5, &tempting),
        "survival goals hold against the same temptation"
    );
    let breakdown = Candidate {
        goal: Goal::Break(finallanding::agents::goals::BreakKind::Sulk),
        utility: 0.1,
        reason: String::new(),
    };
    assert!(
        should_switch(&eating, 0.5, &breakdown),
        "a breakdown overrides everything"
    );
}

#[test]
fn survivors_share_knowledge_and_answer_requests_for_help() {
    let mut sim = colony(9);
    let (a, b) = (sim.agents[0].id, sim.agents[1].id);
    let tile = Tile::new(1, 1);
    sim.agents[0].beliefs.note_node(
        4242,
        NodeBelief {
            kind: NodeKind::OreVein,
            tile,
            amount: 50.0,
            seen_tick: 0,
        },
    );
    sim.agents[1].beliefs.nodes.remove(&4242);
    for _ in 0..12 {
        let mut effects = Effects::default();
        effects.interactions.push(Interaction {
            from: a,
            to: b,
            topic: Topic::SmallTalk,
        });
        social::apply_effects(&mut sim, effects);
    }
    let told = sim
        .agent(b)
        .expect("present")
        .inbox
        .iter()
        .any(|m| matches!(m.kind, MessageKind::InformNode { node: 4242, .. }));
    assert!(told, "conversation passes on where the ore is");
    let helper = sim.agent_mut(b).expect("present");
    helper.inbox.clear();
    helper.personality.kindness = 1.0;
    helper.beliefs.relation_mut(a).opinion = 50.0;
    helper.needs.set(Need::Food, 90.0);
    helper.inbox.push(Message {
        from: a,
        to: b,
        kind: MessageKind::RequestHelp {
            need: HelpNeed::Food,
            place: tile,
        },
        tick: 0,
    });
    sim.step();
    assert_eq!(
        sim.agent(b).and_then(|h| h.help_task),
        Some((a, HelpNeed::Food))
    );
}
