//! Plans for work the colony has posted: building, gathering, hauling,
//! cooking, research and medicine.

use super::{known_nodes, nearest, option, storage};
use crate::agents::deliberation::View;
use crate::agents::goals::Goal;
use crate::agents::plans::{PlanOption, Step, Target};
use crate::agents::Agent;
use crate::data::{NodeKind, Resource};
use crate::world::{AgentId, Structure, StructureId, Tile};

fn build_plans(agent: &Agent, view: &View, site: StructureId) -> Vec<PlanOption> {
    let mut plans = Vec::new();
    let Some(job) = view.colony.jobs.build_job(site) else {
        return plans;
    };
    if job.ready_to_build {
        plans.push(option(
            "construct",
            1.0,
            vec![Step::GoTo(Target::Structure(site)), Step::Construct(site)],
        ));
        return plans;
    }
    let capacity = agent.carry_capacity(view.colony.modifiers.carry);
    for (resource, missing) in job.missing.nonzero() {
        let amount = missing.min(capacity);
        if let Some(item) = view
            .world
            .items
            .iter()
            .find(|i| i.resource == resource && i.tile.distance(agent.tile) < 20.0)
        {
            plans.push(option(
                "deliver_from_ground",
                1.1,
                vec![
                    Step::GoTo(Target::Tile(item.tile)),
                    Step::PickUp(item.tile),
                    Step::GoTo(Target::Structure(site)),
                    Step::Deliver(site),
                ],
            ));
        }
        if agent.beliefs.believes_stock(resource, 1.0) {
            if let Some(store) = storage(agent, view) {
                plans.push(option(
                    "deliver_from_stores",
                    1.0,
                    vec![
                        Step::GoTo(Target::Structure(store)),
                        Step::TakeStock { resource, amount },
                        Step::GoTo(Target::Structure(site)),
                        Step::Deliver(site),
                    ],
                ));
            }
        }
        if !plans.is_empty() {
            break;
        }
    }
    plans
}

fn gather_plans(agent: &Agent, view: &View, kind: NodeKind) -> Vec<PlanOption> {
    let ids: [&'static str; 3] = ["gather_nearest", "gather_second", "gather_third"];
    let Some(store) = storage(agent, view) else {
        return Vec::new();
    };
    known_nodes(agent, view, kind)
        .into_iter()
        .enumerate()
        .map(|(index, node)| {
            option(
                ids[index],
                1.0 - index as f32 * 0.2,
                vec![
                    Step::GoTo(Target::Node(node)),
                    Step::Harvest(node),
                    Step::GoTo(Target::Structure(store)),
                    Step::Deposit,
                ],
            )
        })
        .collect()
}

/// The nearest unexplored tile bordering explored ground.
pub fn frontier_tile(agent: &Agent, view: &View) -> Option<Tile> {
    let map = &view.world.map;
    let mut best: Option<(f32, Tile)> = None;
    let spread = (agent.id as i32 * 13) % 7;
    for radius in (4..60).step_by(3) {
        for step in 0..(radius * 4) {
            let angle = (step as f32 + spread as f32) / (radius * 4) as f32 * std::f32::consts::TAU;
            let tile = Tile::new(
                agent.tile.x + (angle.cos() * radius as f32) as i32,
                agent.tile.y + (angle.sin() * radius as f32) as i32,
            );
            if !map.in_bounds(tile)
                || map.is_explored(tile)
                || !map.terrain_at(tile).is_some_and(|t| t.is_walkable())
            {
                continue;
            }
            let distance = tile.distance(agent.tile);
            if best.is_none_or(|(d, _)| distance < d) {
                best = Some((distance, tile));
            }
        }
        if best.is_some() {
            break;
        }
    }
    best.map(|(_, tile)| tile)
}

pub(super) fn work_plans(agent: &Agent, view: &View, goal: &Goal) -> Vec<PlanOption> {
    let store = storage(agent, view);
    match *goal {
        Goal::Build(site) => build_plans(agent, view, site),
        Goal::Repair(site) => vec![option(
            "repair",
            1.0,
            vec![Step::GoTo(Target::Structure(site)), Step::Repair(site)],
        )],
        Goal::Farm(plot) => {
            let mut steps = vec![Step::GoTo(Target::Structure(plot)), Step::Tend(plot)];
            if let Some(store) = store {
                steps.extend([Step::GoTo(Target::Structure(store)), Step::Deposit]);
            }
            vec![option("tend_plot", 1.0, steps)]
        }
        Goal::Gather(kind) => gather_plans(agent, view, kind),
        Goal::Scout => frontier_tile(agent, view)
            .map(|tile| {
                vec![option(
                    "scout_frontier",
                    1.0,
                    vec![Step::GoTo(Target::Tile(tile)), Step::Scout(tile)],
                )]
            })
            .unwrap_or_default(),
        Goal::Haul => haul_plans(agent, view, store),
        Goal::Cook => cook_plans(agent, view, store),
        Goal::Craft(station) => vec![option(
            "craft",
            1.0,
            vec![Step::GoTo(Target::Structure(station)), Step::Craft(station)],
        )],
        Goal::Research => research_plans(agent, view),
        Goal::Treat(patient) => treat_plans(agent, patient, store),
        _ => Vec::new(),
    }
}

/// Several haul plans over nearby items, rotated per survivor so haulers
/// spread out instead of racing for the same crate.
fn haul_plans(agent: &Agent, view: &View, store: Option<StructureId>) -> Vec<PlanOption> {
    let Some(store) = store else {
        return Vec::new();
    };
    let mut items: Vec<Tile> = view.colony.jobs.items.clone();
    items.sort_by(|a, b| a.distance(agent.tile).total_cmp(&b.distance(agent.tile)));
    items.truncate(4);
    if items.is_empty() {
        return Vec::new();
    }
    let rotation = agent.id as usize % items.len();
    items.rotate_left(rotation);
    let ids: [&'static str; 3] = ["haul_item", "haul_next_item", "haul_far_item"];
    items
        .into_iter()
        .take(3)
        .enumerate()
        .map(|(index, tile)| {
            option(
                ids[index],
                1.0 - index as f32 * 0.1,
                vec![
                    Step::GoTo(Target::Tile(tile)),
                    Step::PickUp(tile),
                    Step::GoTo(Target::Structure(store)),
                    Step::Deposit,
                ],
            )
        })
        .collect()
}

fn cook_plans(agent: &Agent, view: &View, store: Option<StructureId>) -> Vec<PlanOption> {
    let mut plans = Vec::new();
    let Some(kitchen) = nearest(agent, view, |s| s.def().kitchen) else {
        return plans;
    };
    if agent.carried(Resource::Food) >= 1.0 {
        plans.push(option(
            "cook_carried",
            1.2,
            vec![Step::GoTo(Target::Structure(kitchen)), Step::Cook(kitchen)],
        ));
    }
    if let Some(store) = store.filter(|_| agent.beliefs.believes_stock(Resource::Food, 2.0)) {
        plans.push(option(
            "cook_from_stores",
            1.0,
            vec![
                Step::GoTo(Target::Structure(store)),
                Step::TakeStock {
                    resource: Resource::Food,
                    amount: 6.0,
                },
                Step::GoTo(Target::Structure(kitchen)),
                Step::Cook(kitchen),
            ],
        ));
    }
    if let Some(node) = known_nodes(agent, view, NodeKind::Forage).first() {
        plans.push(option(
            "cook_foraged",
            0.5,
            vec![
                Step::GoTo(Target::Node(*node)),
                Step::Harvest(*node),
                Step::GoTo(Target::Structure(kitchen)),
                Step::Cook(kitchen),
            ],
        ));
    }
    plans
}

fn research_plans(agent: &Agent, view: &View) -> Vec<PlanOption> {
    let mut plans = Vec::new();
    let board = &view.colony.jobs;
    if board.relic_study {
        if let Some(lab) = nearest(agent, view, |s| {
            s.def().research_branch == Some(crate::data::Branch::Xenology)
        }) {
            plans.push(option(
                "study_relics",
                1.1,
                vec![Step::GoTo(Target::Structure(lab)), Step::StudyRelic(lab)],
            ));
        }
    }
    if board.research_open {
        let mut labs: Vec<&Structure> = view
            .world
            .built()
            .filter(|s| s.is_operational() && s.def().research > 0.0)
            .collect();
        labs.sort_by(|a, b| {
            a.footprint
                .distance_to(agent.tile)
                .total_cmp(&b.footprint.distance_to(agent.tile))
        });
        let ids: [&'static str; 2] = ["research_nearest_lab", "research_other_lab"];
        for (index, lab) in labs.into_iter().take(2).enumerate() {
            let quality = lab.def().research;
            plans.push(option(
                ids[index],
                0.8 + quality * 0.2 - index as f32 * 0.1,
                vec![
                    Step::GoTo(Target::Structure(lab.id)),
                    Step::Research(lab.id),
                ],
            ));
        }
    }
    plans
}

pub(super) fn treat_plans(
    agent: &Agent,
    patient: AgentId,
    store: Option<StructureId>,
) -> Vec<PlanOption> {
    let mut plans = Vec::new();
    if let Some(store) = store.filter(|_| agent.beliefs.believes_stock(Resource::Medicine, 1.0)) {
        plans.push(option(
            "treat_with_medicine",
            1.0,
            vec![
                Step::GoTo(Target::Structure(store)),
                Step::TakeStock {
                    resource: Resource::Medicine,
                    amount: 1.0,
                },
                Step::GoTo(Target::Agent(patient)),
                Step::Treat(patient),
            ],
        ));
    }
    plans.push(option(
        "bedside_care",
        0.5,
        vec![Step::GoTo(Target::Agent(patient)), Step::Treat(patient)],
    ));
    plans
}
