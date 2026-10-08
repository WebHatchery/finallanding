//! Player commands: the colony AI's only levers. None of them control a
//! survivor directly.

use super::expeditions::{cancel_call, open_call};
use super::vote::hold_vote;
use super::{jobs, Sim};
use crate::agents::goals::WorkType;
use crate::data::game_data;
use crate::world::{PlacementIssue, StructureId, Tile};

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    PlaceBlueprint { building: String, origin: Tile },
    CancelBlueprint(StructureId),
    SetPriority { work: WorkType, level: u8 },
    SetPolicy { policy: String, option: String },
    SetResearchFocus(Option<String>),
    OpenExpedition(String),
    CancelExpedition,
    ProposeFuture(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandError {
    UnknownBuilding,
    Locked,
    Placement(PlacementIssue),
    Rejected,
}

/// Whether a building is unlocked by research.
pub fn is_unlocked(sim: &Sim, building: &str) -> bool {
    game_data()
        .building(building)
        .is_some_and(|def| def.is_player_buildable() && sim.colony.is_tech_researched(&def.tech))
}

pub fn apply(sim: &mut Sim, command: Command) -> Result<(), CommandError> {
    match command {
        Command::PlaceBlueprint { building, origin } => {
            let def = game_data()
                .building(&building)
                .ok_or(CommandError::UnknownBuilding)?;
            if !is_unlocked(sim, &building) {
                return Err(CommandError::Locked);
            }
            sim.world
                .check_placement(def, origin)
                .map_err(CommandError::Placement)?;
            let free = def.cost.is_empty();
            sim.world
                .add_structure(def, origin, sim.calendar.day(), false);
            if free && def.work <= 0.0 {
                let id = sim.world.next_structure_id - 1;
                sim.world.complete_structure(id);
            }
            jobs::refresh(sim);
            Ok(())
        }
        Command::CancelBlueprint(id) => {
            let is_blueprint = sim.world.structure(id).is_some_and(|s| !s.is_built());
            if !is_blueprint {
                return Err(CommandError::Rejected);
            }
            if let Some(refund) = sim.world.remove_structure(id) {
                sim.colony.stock.add_bag(&refund);
            }
            jobs::refresh(sim);
            Ok(())
        }
        Command::SetPriority { work, level } => {
            sim.colony.priorities[work.index()] = level.min(3);
            Ok(())
        }
        Command::SetPolicy { policy, option } => {
            let def = game_data().policy(&policy).ok_or(CommandError::Rejected)?;
            let available = sim.colony.is_tech_researched(&def.tech);
            if !available || !def.options.iter().any(|o| o.id == option) {
                return Err(CommandError::Rejected);
            }
            sim.colony.policies.insert(policy, option);
            Ok(())
        }
        Command::SetResearchFocus(focus) => {
            sim.colony.research.focus = focus;
            Ok(())
        }
        Command::OpenExpedition(site) => open_call(sim, &site)
            .then_some(())
            .ok_or(CommandError::Rejected),
        Command::CancelExpedition => {
            cancel_call(sim);
            Ok(())
        }
        Command::ProposeFuture(ending) => {
            let ready = sim.colony.campaign.act >= 4 && sim.world.count_built("council_hall") > 0;
            if !ready {
                return Err(CommandError::Rejected);
            }
            hold_vote(sim, &ending)
                .then_some(())
                .ok_or(CommandError::Rejected)
        }
    }
}
