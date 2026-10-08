//! The plan library: alternative ways of achieving each goal.
//!
//! Every plan has a context condition (checked here against the survivor's
//! beliefs and the world they can see) and a preference. When a plan fails,
//! the intention excludes it and the next applicable plan is tried.

use super::beliefs::FailedTarget;
use super::deliberation::View;
use super::goals::{BreakKind, Goal, HelpNeed};
use super::plans::{PlanOption, Step, Target, Topic};
use super::Agent;
use crate::data::{game_data, NodeKind, Resource};
use crate::world::{AgentId, Structure, StructureId};

mod needs;
mod work;

use needs::{care_plans, eat_plans, flee_plans, relax_plans, sleep_plans, wander_tile, warm_plans};
pub use work::frontier_tile;
use work::{treat_plans, work_plans};

const FAILURE_MEMORY_TICKS: u64 = 240;

fn option(id: &'static str, preference: f32, steps: Vec<Step>) -> PlanOption {
    PlanOption {
        id,
        steps,
        preference,
    }
}

/// Nearest built, operational structure matching `wanted`, skipping places
/// this survivor recently failed to use.
fn nearest(agent: &Agent, view: &View, wanted: impl Fn(&Structure) -> bool) -> Option<StructureId> {
    let now = view.calendar.tick;
    view.world
        .built()
        .filter(|s| s.is_operational() && wanted(s))
        .filter(|s| {
            !agent
                .beliefs
                .recently_failed(FailedTarget::Structure(s.id), now, FAILURE_MEMORY_TICKS)
        })
        .min_by(|a, b| {
            a.footprint
                .distance_to(agent.tile)
                .total_cmp(&b.footprint.distance_to(agent.tile))
        })
        .map(|s| s.id)
}

fn storage(agent: &Agent, view: &View) -> Option<StructureId> {
    nearest(agent, view, |s| s.def().storage > 0.0)
}

fn friend_nearby(agent: &Agent, view: &View, min_opinion: f32) -> Option<AgentId> {
    view.others
        .iter()
        .filter(|s| s.id != agent.id && s.is_available_to_talk() && !s.is_child)
        .filter(|s| agent.beliefs.opinion_of(s.id) >= min_opinion)
        .min_by(|a, b| {
            a.tile
                .distance(agent.tile)
                .total_cmp(&b.tile.distance(agent.tile))
        })
        .map(|s| s.id)
}

fn known_nodes(agent: &Agent, view: &View, kind: NodeKind) -> Vec<u32> {
    let now = view.calendar.tick;
    agent
        .beliefs
        .known_nodes(kind, agent.tile)
        .into_iter()
        .filter(|(id, _)| {
            !agent
                .beliefs
                .recently_failed(FailedTarget::Node(*id), now, FAILURE_MEMORY_TICKS)
        })
        .map(|(id, _)| id)
        .take(3)
        .collect()
}

fn social_plans(agent: &Agent, goal: &Goal) -> Vec<PlanOption> {
    let lashing_out = matches!(agent.mental_break, Some((BreakKind::LashOut, _)));
    let (id, topic, other): (&'static str, Topic, AgentId) = match *goal {
        Goal::Chat(other) if lashing_out => ("pick_a_fight", Topic::Insult, other),
        Goal::Chat(other) => ("chat", Topic::SmallTalk, other),
        Goal::Comfort(other) => ("comfort", Topic::Comfort, other),
        Goal::Court(other) => ("court", Topic::Flirt, other),
        Goal::Reconcile(other) => ("make_peace", Topic::Reconcile, other),
        _ => return Vec::new(),
    };
    vec![option(
        id,
        1.0,
        vec![
            Step::GoTo(Target::Agent(other)),
            Step::Talk { with: other, topic },
        ],
    )]
}

fn help_plans(agent: &Agent, view: &View, requester: AgentId, need: HelpNeed) -> Vec<PlanOption> {
    match need {
        HelpNeed::Food => {
            let mut plans = Vec::new();
            if agent.carried(Resource::Food) >= 1.0 {
                plans.push(option(
                    "hand_over_food",
                    1.2,
                    vec![Step::GoTo(Target::Agent(requester)), Step::Give(requester)],
                ));
            }
            if let Some(store) =
                storage(agent, view).filter(|_| agent.beliefs.believes_stock(Resource::Food, 1.0))
            {
                plans.push(option(
                    "fetch_food",
                    1.0,
                    vec![
                        Step::GoTo(Target::Structure(store)),
                        Step::TakeStock {
                            resource: Resource::Food,
                            amount: 2.0,
                        },
                        Step::GoTo(Target::Agent(requester)),
                        Step::Give(requester),
                    ],
                ));
            }
            if let Some(node) = known_nodes(agent, view, NodeKind::Forage).first() {
                plans.push(option(
                    "forage_for_friend",
                    0.6,
                    vec![
                        Step::GoTo(Target::Node(*node)),
                        Step::Harvest(*node),
                        Step::GoTo(Target::Agent(requester)),
                        Step::Give(requester),
                    ],
                ));
            }
            plans
        }
        HelpNeed::Care => treat_plans(agent, requester, storage(agent, view)),
        HelpNeed::Defence(creature) => {
            vec![option("defend_friend", 1.0, vec![Step::Attack(creature)])]
        }
    }
}

fn break_plans(agent: &Agent, view: &View, kind: BreakKind) -> Vec<PlanOption> {
    let ticks = game_data().balance.mood.break_ticks;
    match kind {
        BreakKind::Binge => match storage(agent, view) {
            Some(store) => vec![option(
                "binge",
                1.0,
                vec![
                    Step::GoTo(Target::Structure(store)),
                    Step::TakeStock {
                        resource: Resource::Food,
                        amount: 4.0,
                    },
                    Step::Break { kind, ticks },
                ],
            )],
            None => vec![option(
                "sulk",
                1.0,
                vec![Step::Break {
                    kind: BreakKind::Sulk,
                    ticks,
                }],
            )],
        },
        BreakKind::Wander => vec![option(
            "wander_off",
            1.0,
            vec![
                Step::GoTo(Target::Tile(wander_tile(agent, view))),
                Step::Break { kind, ticks },
            ],
        )],
        _ => vec![option("sulk", 1.0, vec![Step::Break { kind, ticks }])],
    }
}

/// Every applicable plan for a goal, best preference first.
pub fn plans_for(goal: &Goal, agent: &Agent, view: &View) -> Vec<PlanOption> {
    let mut plans = match *goal {
        Goal::Eat => eat_plans(agent, view),
        Goal::Sleep => sleep_plans(agent, view),
        Goal::WarmUp => warm_plans(agent, view),
        Goal::Flee => flee_plans(agent, view),
        Goal::SeekCare => care_plans(agent, view),
        Goal::Relax => relax_plans(agent, view),
        Goal::Defend(creature) => vec![option("drive_off", 1.0, vec![Step::Attack(creature)])],
        Goal::Help { requester, need } => help_plans(agent, view, requester, need),
        Goal::Volunteer => nearest(agent, view, |s| s.def().expedition)
            .map(|gate| {
                vec![option(
                    "sign_up",
                    1.0,
                    vec![Step::GoTo(Target::Structure(gate)), Step::JoinExpedition],
                )]
            })
            .unwrap_or_default(),
        Goal::Break(kind) => break_plans(agent, view, kind),
        Goal::Idle => vec![option(
            "loiter",
            1.0,
            vec![
                Step::GoTo(Target::Tile(wander_tile(agent, view))),
                Step::Wait { ticks: 20 },
            ],
        )],
        Goal::Chat(_) | Goal::Comfort(_) | Goal::Court(_) | Goal::Reconcile(_) => {
            social_plans(agent, goal)
        }
        _ => work_plans(agent, view, goal),
    };
    plans.sort_by(|a, b| b.preference.total_cmp(&a.preference));
    plans
}
