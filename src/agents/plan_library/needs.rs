//! Plans for bodily needs: eating, sleeping, warmth, safety, care and rest.

use super::{friend_nearby, known_nodes, nearest, option, storage};
use crate::agents::deliberation::View;
use crate::agents::goals::HelpNeed;
use crate::agents::messages::MessageKind;
use crate::agents::plans::{FoodSource, PlanOption, Step, Target};
use crate::agents::Agent;
use crate::data::{game_data, NodeKind, Resource};
use crate::world::{AgentId, StructureId, Tile};

pub(super) fn eat_plans(agent: &Agent, view: &View) -> Vec<PlanOption> {
    let mut plans = Vec::new();
    let gourmand = if agent.has_trait("gourmand") {
        0.3
    } else {
        0.0
    };
    if agent.carried(Resource::Food) >= 1.0 {
        plans.push(option(
            "eat_carried",
            1.3,
            vec![Step::Eat(FoodSource::Carried)],
        ));
    }
    if agent.beliefs.believes_stock(Resource::Meals, 1.0) {
        if let Some(hall) = nearest(agent, view, |s| s.def().kitchen) {
            plans.push(option(
                "mess_meal",
                1.0 + gourmand,
                vec![
                    Step::GoTo(Target::Structure(hall)),
                    Step::TakeMeal,
                    Step::Eat(FoodSource::Meal),
                ],
            ));
        }
    }
    if agent.beliefs.believes_stock(Resource::Food, 1.0) {
        if let Some(store) = storage(agent, view) {
            plans.push(option(
                "raw_stores",
                0.7,
                vec![
                    Step::GoTo(Target::Structure(store)),
                    Step::TakeStock {
                        resource: Resource::Food,
                        amount: 1.0,
                    },
                    Step::Eat(FoodSource::Raw),
                ],
            ));
        }
    }
    if let Some(node) = known_nodes(agent, view, NodeKind::Glowfruit).first() {
        plans.push(option(
            "forage",
            0.45,
            vec![Step::GoTo(Target::Node(*node)), Step::Forage(*node)],
        ));
    }
    if let Some(friend) = friend_nearby(agent, view, 25.0) {
        plans.push(option(
            "ask_friend_for_food",
            0.2,
            vec![
                Step::Send {
                    to: friend,
                    kind: MessageKind::RequestHelp {
                        need: HelpNeed::Food,
                        place: agent.tile,
                    },
                },
                Step::WaitForHelp {
                    ticks: game_data().balance.agents.help_wait_ticks,
                },
                Step::Eat(FoodSource::Carried),
            ],
        ));
    }
    plans
}

/// Where another survivor sleeps, from the colony's housing register.
fn home_of(view: &View, agent: AgentId) -> Option<StructureId> {
    view.world
        .built()
        .find(|s| s.residents.contains(&agent))
        .map(|s| s.id)
}

pub(super) fn sleep_plans(agent: &Agent, view: &View) -> Vec<PlanOption> {
    let mut plans = Vec::new();
    if let Some(home) = agent
        .home
        .filter(|h| view.world.structure(*h).is_some_and(|s| s.is_built()))
    {
        let comfort = view
            .world
            .structure(home)
            .map(|s| s.def().comfort)
            .unwrap_or(0.0);
        plans.push(option(
            "own_bed",
            1.0 + comfort,
            vec![Step::GoTo(Target::Structure(home)), Step::Sleep(Some(home))],
        ));
    }
    let partner_home = agent
        .partner
        .and_then(|p| view.others.iter().find(|s| s.id == p))
        .and(agent.partner)
        .and_then(|p| home_of(view, p))
        .filter(|h| Some(*h) != agent.home);
    if let Some(home) =
        partner_home.filter(|h| view.world.structure(*h).is_some_and(|s| s.free_beds() > 0))
    {
        plans.push(option(
            "partner_bed",
            1.5,
            vec![Step::GoTo(Target::Structure(home)), Step::Sleep(Some(home))],
        ));
    }
    let in_family_home = agent
        .home
        .and_then(|h| view.world.structure(h))
        .is_some_and(|s| s.def().family_only);
    if agent.partner.is_some() && partner_home.is_none() && !in_family_home {
        if let Some(home) = nearest(agent, view, |s| s.def().family_only && s.free_beds() >= 2) {
            plans.push(option(
                "family_home",
                1.6,
                vec![Step::GoTo(Target::Structure(home)), Step::Sleep(Some(home))],
            ));
        }
    }
    let has_family = agent.partner.is_some() || !agent.parents.is_empty();
    if let Some(bed) = nearest(agent, view, |s| {
        s.free_beds() > 0 && (has_family || !s.def().family_only)
    }) {
        plans.push(option(
            "free_bed",
            0.8,
            vec![Step::GoTo(Target::Structure(bed)), Step::Sleep(Some(bed))],
        ));
    }
    if let Some(fire) = nearest(agent, view, |s| s.def().heat > 0.0 && !s.def().indoor) {
        plans.push(option(
            "sleep_by_fire",
            0.4,
            vec![Step::GoTo(Target::Structure(fire)), Step::Sleep(None)],
        ));
    }
    plans.push(option("sleep_rough", 0.15, vec![Step::Sleep(None)]));
    plans
}

pub(super) fn warm_plans(agent: &Agent, view: &View) -> Vec<PlanOption> {
    let mut plans = Vec::new();
    if let Some(heated) = nearest(agent, view, |s| s.def().indoor && s.def().heat > 0.0) {
        plans.push(option(
            "heated_room",
            1.0,
            vec![
                Step::GoTo(Target::Structure(heated)),
                Step::Warm { ticks: 60 },
            ],
        ));
    }
    if let Some(fire) = nearest(agent, view, |s| s.def().heat > 0.0 && !s.def().indoor) {
        plans.push(option(
            "fireside",
            0.9,
            vec![
                Step::GoTo(Target::Structure(fire)),
                Step::Warm { ticks: 50 },
            ],
        ));
    }
    if let Some(shelter) = nearest(agent, view, |s| s.def().indoor) {
        plans.push(option(
            "shelter",
            0.6,
            vec![
                Step::GoTo(Target::Structure(shelter)),
                Step::Warm { ticks: 60 },
            ],
        ));
    }
    if let Some(friend) = friend_nearby(agent, view, 30.0) {
        plans.push(option(
            "huddle",
            0.3,
            vec![Step::GoTo(Target::Agent(friend)), Step::Warm { ticks: 30 }],
        ));
    }
    plans.push(option("keep_moving", 0.1, vec![Step::Warm { ticks: 20 }]));
    plans
}

fn flee_tile(agent: &Agent, view: &View) -> Option<Tile> {
    let (threat, _) = agent.beliefs.nearest_threat(agent.tile)?;
    let dx = (agent.tile.x - threat.tile.x).signum();
    let dy = (agent.tile.y - threat.tile.y).signum();
    let away = agent.tile.offset(dx * 8, dy * 8);
    view.world.map.nearest_walkable(away, 4)
}

pub(super) fn flee_plans(agent: &Agent, view: &View) -> Vec<PlanOption> {
    let mut plans = Vec::new();
    let threat_tile = agent
        .beliefs
        .nearest_threat(agent.tile)
        .map(|(t, _)| t.tile);
    let safe = nearest(agent, view, |s| {
        s.def().indoor && threat_tile.is_none_or(|t| s.footprint.distance_to(t) > 4.0)
    });
    if let Some(shelter) = safe {
        plans.push(option(
            "shelter_indoors",
            1.0,
            vec![
                Step::GoTo(Target::Structure(shelter)),
                Step::Hide { ticks: 40 },
            ],
        ));
    }
    if let Some(tile) = flee_tile(agent, view) {
        plans.push(option(
            "run_away",
            0.6,
            vec![Step::GoTo(Target::Tile(tile)), Step::Hide { ticks: 25 }],
        ));
    }
    plans.push(option("freeze", 0.1, vec![Step::Hide { ticks: 15 }]));
    plans
}

pub(super) fn care_plans(agent: &Agent, view: &View) -> Vec<PlanOption> {
    let mut plans = Vec::new();
    let occupied = |id: StructureId| {
        view.others
            .iter()
            .filter(|s| s.id != agent.id && s.inside == Some(id) && s.needs_care)
            .count() as u32
    };
    if let Some(ward) = nearest(agent, view, |s| s.def().care_beds > occupied(s.id)) {
        plans.push(option(
            "infirmary",
            1.0,
            vec![
                Step::GoTo(Target::Structure(ward)),
                Step::Recover {
                    place: Some(ward),
                    ticks: 180,
                },
            ],
        ));
    }
    if let Some(bed) = agent
        .home
        .or_else(|| nearest(agent, view, |s| s.def().beds > 0))
    {
        plans.push(option(
            "bed_rest",
            0.6,
            vec![
                Step::GoTo(Target::Structure(bed)),
                Step::Recover {
                    place: Some(bed),
                    ticks: 150,
                },
            ],
        ));
    }
    if let Some(friend) = friend_nearby(agent, view, 20.0) {
        plans.push(option(
            "ask_for_care",
            0.4,
            vec![
                Step::Send {
                    to: friend,
                    kind: MessageKind::RequestHelp {
                        need: HelpNeed::Care,
                        place: agent.tile,
                    },
                },
                Step::WaitForHelp {
                    ticks: game_data().balance.agents.help_wait_ticks,
                },
            ],
        ));
    }
    plans.push(option(
        "rest_here",
        0.1,
        vec![Step::Recover {
            place: None,
            ticks: 90,
        }],
    ));
    plans
}

pub(super) fn wander_tile(agent: &Agent, view: &View) -> Tile {
    let seed = (agent.id as u64 * 31 + view.calendar.tick / 60) as i32;
    let dx = seed.wrapping_mul(7919) % 9 - 4;
    let dy = seed.wrapping_mul(104_729) % 9 - 4;
    let target = agent.tile.offset(dx * 2, dy * 2);
    view.world
        .map
        .nearest_walkable(target, 3)
        .filter(|t| view.world.map.is_explored(*t))
        .unwrap_or(agent.tile)
}

pub(super) fn relax_plans(agent: &Agent, view: &View) -> Vec<PlanOption> {
    let mut plans = Vec::new();
    if let Some(place) = nearest(agent, view, |s| s.def().recreation > 0.0) {
        let quality = view
            .world
            .structure(place)
            .map(|s| s.def().recreation + s.def().beauty * 0.3)
            .unwrap_or(0.5);
        plans.push(option(
            "recreation",
            0.6 + quality,
            vec![
                Step::GoTo(Target::Structure(place)),
                Step::Relax {
                    place: Some(place),
                    ticks: 60,
                },
            ],
        ));
    }
    plans.push(option(
        "stroll",
        0.3,
        vec![
            Step::GoTo(Target::Tile(wander_tile(agent, view))),
            Step::Relax {
                place: None,
                ticks: 40,
            },
        ],
    ));
    plans
}
