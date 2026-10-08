//! Turning goals, plan steps and decisions into readable text.

use crate::agents::goals::Goal;
use crate::agents::intention::DecisionKind;
use crate::agents::plans::{Step, Target};
use crate::data::{fill_template, game_data};
use crate::sim::Sim;
use crate::world::Calendar;

fn agent_name(sim: &Sim, id: u32) -> String {
    sim.agent(id)
        .map(|a| a.given_name.clone())
        .unwrap_or_default()
}

fn structure_name(sim: &Sim, id: u32) -> String {
    sim.world
        .structure(id)
        .map(|s| s.def().name.clone())
        .unwrap_or_default()
}

fn node_name(sim: &Sim, id: u32) -> String {
    sim.world
        .node(id)
        .map(|n| {
            game_data()
                .label(&format!("node_{}", n.kind.key()))
                .to_owned()
        })
        .unwrap_or_else(|| game_data().label("somewhere").to_owned())
}

pub fn goal_text(sim: &Sim, goal: &Goal) -> String {
    let data = game_data();
    let base = data.label(goal.label_key()).to_owned();
    let target = match *goal {
        Goal::Build(id) | Goal::Repair(id) | Goal::Farm(id) | Goal::Craft(id) => {
            Some(structure_name(sim, id))
        }
        Goal::Gather(kind) => Some(data.label(&format!("node_{}", kind.key())).to_owned()),
        Goal::Treat(id)
        | Goal::Chat(id)
        | Goal::Comfort(id)
        | Goal::Court(id)
        | Goal::Reconcile(id) => Some(agent_name(sim, id)),
        Goal::Help { requester, .. } => Some(agent_name(sim, requester)),
        _ => None,
    };
    match target {
        Some(target) if !target.is_empty() => format!("{base}: {target}"),
        _ => base,
    }
}

fn target_text(sim: &Sim, target: &Target) -> String {
    match *target {
        Target::Tile(_) => game_data().label("somewhere").to_owned(),
        Target::Structure(id) => structure_name(sim, id),
        Target::Node(id) => node_name(sim, id),
        Target::Agent(id) => agent_name(sim, id),
    }
}

pub fn step_text(sim: &Sim, step: &Step) -> String {
    let data = game_data();
    let fill = |key: &str, value: String| fill_template(data.label(key), &[("target", &value)]);
    match step {
        Step::GoTo(target) => fill("step_goto", target_text(sim, target)),
        Step::TakeStock { resource, amount } => fill(
            "step_take",
            format!("{amount:.0} {}", data.label(resource.key())),
        ),
        Step::TakeMeal => data.label("step_take_meal").to_owned(),
        Step::Deposit => data.label("step_deposit").to_owned(),
        Step::Deliver(id) => fill("step_deliver", structure_name(sim, *id)),
        Step::Construct(id) => fill("step_construct", structure_name(sim, *id)),
        Step::Repair(id) => fill("step_repair", structure_name(sim, *id)),
        Step::Harvest(id) => fill("step_harvest", node_name(sim, *id)),
        Step::Forage(id) => fill("step_forage", node_name(sim, *id)),
        Step::PickUp(_) => data.label("step_pick_up").to_owned(),
        Step::Eat(_) => data.label("step_eat").to_owned(),
        Step::Sleep(_) => data.label("step_sleep").to_owned(),
        Step::Warm { .. } => data.label("step_warm").to_owned(),
        Step::Relax { .. } => data.label("step_relax").to_owned(),
        Step::Tend(id) => fill("step_tend", structure_name(sim, *id)),
        Step::Cook(_) => data.label("step_cook").to_owned(),
        Step::Craft(id) => fill("step_craft", structure_name(sim, *id)),
        Step::Research(_) => data.label("step_research").to_owned(),
        Step::StudyRelic(_) => data.label("step_study").to_owned(),
        Step::Treat(id) => fill("step_treat", agent_name(sim, *id)),
        Step::Recover { .. } => data.label("step_recover").to_owned(),
        Step::Talk { with, .. } => fill("step_talk", agent_name(sim, *with)),
        Step::Send { to, .. } => fill("step_send", agent_name(sim, *to)),
        Step::WaitForHelp { .. } => data.label("step_wait_help").to_owned(),
        Step::Give(id) => fill("step_give", agent_name(sim, *id)),
        Step::Attack(_) => data.label("step_attack").to_owned(),
        Step::Hide { .. } => data.label("step_hide").to_owned(),
        Step::Scout(_) => data.label("step_scout").to_owned(),
        Step::JoinExpedition => data.label("step_join").to_owned(),
        Step::Break { .. } => data.label("step_break").to_owned(),
        Step::Wait { .. } => data.label("step_wait").to_owned(),
    }
}

pub fn plan_text(plan: &str) -> String {
    game_data().label(&format!("plan_{plan}")).to_owned()
}

/// One line of the reasoning log, and whether it marks a setback.
pub fn decision_text(sim: &Sim, kind: &DecisionKind) -> (String, bool) {
    let data = game_data();
    match kind {
        DecisionKind::Adopted { goal, reason, .. } => {
            let mut text = fill_template(
                data.label("log_adopted"),
                &[("goal", &goal_text(sim, goal))],
            );
            if !reason.is_empty() {
                text.push_str(&format!(" ({reason})"));
            }
            (text, false)
        }
        DecisionKind::ChosePlan { plan } => (
            fill_template(data.label("log_plan"), &[("plan", &plan_text(plan))]),
            false,
        ),
        DecisionKind::PlanFailed { plan, reason } => (
            fill_template(
                data.label("log_failed"),
                &[
                    ("plan", &plan_text(plan)),
                    ("reason", data.label(reason.label_key())),
                ],
            ),
            true,
        ),
        DecisionKind::GaveUp { goal } => (
            fill_template(
                data.label("log_gave_up"),
                &[("goal", &goal_text(sim, goal))],
            ),
            true,
        ),
        DecisionKind::Achieved { goal } => (
            fill_template(
                data.label("log_achieved"),
                &[("goal", &goal_text(sim, goal))],
            ),
            false,
        ),
        DecisionKind::Interrupted { by } => (
            fill_template(data.label("log_interrupted"), &[("by", by)]),
            true,
        ),
        DecisionKind::Message { text } => (text.clone(), false),
    }
}

/// Clock time of a tick, as HH:MM.
pub fn clock(tick: u64) -> String {
    let calendar = Calendar { tick };
    format!(
        "{:02}:{:02}",
        calendar.hour() as u32,
        calendar.minute_of_hour()
    )
}
