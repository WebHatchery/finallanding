//! Headless campaign runs: play several seeds with the scripted colony AI and
//! summarise pacing, outcomes and colony life as Markdown.

use super::plan;
use crate::colony::campaign::Outcome;
use crate::colony::RunSetup;
use crate::sim::Sim;
use crate::world::Calendar;

/// Average real minutes per in-game day for the documented speed mix.
pub const MIXED_SPEED_MINUTES_PER_DAY: f32 = 5.2;
const PLANS_PER_DAY: u64 = 6;

#[derive(Clone, Debug)]
pub struct RunSummary {
    pub site: String,
    pub difficulty: String,
    pub seed: u64,
    pub act_days: Vec<u32>,
    pub final_day: u32,
    pub outcome: String,
    pub population: usize,
    pub peak_population: usize,
    pub deaths: u32,
    pub births: u32,
    pub techs: usize,
    pub relics: f32,
    pub expeditions: u32,
    pub friendships: u32,
    pub average_mood: f32,
    /// Native species of the landing.
    pub species: Vec<String>,
    /// Species in the order the colony first gathered them.
    pub finds: Vec<String>,
    /// Inspired technologies revealed, and how many of them were researched.
    pub inspired: (usize, usize),
}

impl RunSummary {
    pub fn estimated_hours(&self) -> f32 {
        self.final_day as f32 * MIXED_SPEED_MINUTES_PER_DAY / 60.0
    }
}

/// Play one campaign to its end, or to `max_days`.
pub fn run_campaign(setup: RunSetup, max_days: u32) -> RunSummary {
    let mut sim = Sim::new(setup.clone());
    let mut act_days = vec![1];
    let ticks_per_plan = Calendar::ticks_per_day() / PLANS_PER_DAY;
    while !sim.colony.campaign.is_over() && sim.calendar.day() <= max_days {
        plan(&mut sim);
        sim.run_ticks(ticks_per_plan);
        if sim.colony.campaign.act as usize > act_days.len() {
            act_days.push(sim.calendar.day());
        }
    }
    let outcome = match &sim.colony.campaign.outcome {
        Some(Outcome::Victory { ending, .. }) => format!("Victory ({ending})"),
        Some(Outcome::Failure { reason, .. }) => format!("Failure ({reason})"),
        None => format!("Unfinished (act {})", sim.colony.campaign.act),
    };
    RunSummary {
        site: setup.site,
        difficulty: setup.difficulty,
        seed: setup.seed,
        act_days,
        final_day: sim.calendar.day(),
        outcome,
        population: sim.population(),
        peak_population: sim.colony.stats.peak_population,
        deaths: sim.colony.stats.deaths,
        births: sim.colony.stats.births,
        techs: sim.colony.tree.researched_count(),
        relics: sim.colony.stats.relics_found,
        expeditions: sim.colony.expeditions.completed,
        friendships: crate::sim::campaign::friendship_count(&sim),
        average_mood: sim.average_mood(),
        species: sim.world.species.iter().map(|id| find_name(id)).collect(),
        finds: sim
            .colony
            .finds
            .found
            .iter()
            .map(|f| find_name(&f.id))
            .collect(),
        inspired: inspired_counts(&sim),
    }
}

fn find_name(id: &str) -> String {
    crate::data::game_data()
        .find(id)
        .map_or_else(|| id.to_owned(), |f| f.name.clone())
}

fn inspired_counts(sim: &Sim) -> (usize, usize) {
    let inspired = sim.colony.tree.nodes.iter().filter(|node| {
        crate::data::game_data()
            .tech(&node.id)
            .is_some_and(|t| t.is_inspired())
    });
    inspired.fold((0, 0), |(revealed, researched), node| {
        (
            revealed + usize::from(node.revealed),
            researched + usize::from(node.researched),
        )
    })
}

pub fn markdown(runs: &[RunSummary]) -> String {
    let mut out = String::from("# The Final Landing — Campaign Pacing Report\n\n");
    out.push_str("Headless campaigns played by the scripted colony AI (`src/autoplay.rs`) through the same commands as the player. ");
    out.push_str(&format!(
        "Estimated hours assume the documented speed mix of {MIXED_SPEED_MINUTES_PER_DAY} real minutes per in-game day, before pauses.\n\n"
    ));
    out.push_str("| Site | Difficulty | Seed | Act start days | Final day | Est. hours | Outcome | Pop (peak) | Deaths | Births | Techs | Relics | Expeditions | Friendships | Mood |\n");
    out.push_str("| --- | --- | ---: | --- | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for run in runs {
        let acts = run
            .act_days
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join(" / ");
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {:.1} | {} | {} ({}) | {} | {} | {} | {:.0} | {} | {} | {:.0} |\n",
            run.site, run.difficulty, run.seed, acts, run.final_day, run.estimated_hours(), run.outcome,
            run.population, run.peak_population, run.deaths, run.births, run.techs, run.relics,
            run.expeditions, run.friendships, run.average_mood,
        ));
    }
    out.push_str("\n## Discovery\n\nEach landing draws its own native species; the colony finds them in an order set by where their territories fall. Inspired technologies are revealed by gathering those species.\n\n");
    out.push_str(
        "| Site | Seed | Native species | Order found | Inspired techs revealed / researched |\n",
    );
    out.push_str("| --- | ---: | --- | --- | ---: |\n");
    for run in runs {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} / {} |\n",
            run.site,
            run.seed,
            run.species.join(", "),
            run.finds.join(" → "),
            run.inspired.0,
            run.inspired.1,
        ));
    }
    out
}

/// The standard report matrix: every site at Standard, plus Gentle and Harsh.
pub fn standard_matrix(max_days: u32) -> Vec<RunSummary> {
    let mut runs = Vec::new();
    let setups = [
        ("verdant_basin", "standard", 11),
        ("frost_shelf", "standard", 23),
        ("ashen_steppe", "standard", 37),
        ("verdant_basin", "gentle", 41),
        ("verdant_basin", "harsh", 53),
    ];
    for (site, difficulty, seed) in setups {
        runs.push(run_campaign(
            RunSetup {
                colony_name: "Report".into(),
                site: site.into(),
                difficulty: difficulty.into(),
                seed,
            },
            max_days,
        ));
    }
    runs
}
