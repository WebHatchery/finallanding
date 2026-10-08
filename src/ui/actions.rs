//! Intents the UI returns; the game applies them. UI code never mutates state.

use crate::colony::chronicle::Category;
use crate::data::BuildingCategory;
use crate::sim::commands::Command;
use crate::world::{AgentId, CreatureId, NodeId, StructureId, Tile};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overlay {
    Colony,
    Research,
    Colonists,
    Relations,
    Expeditions,
    Chronicle,
    Council,
    Help,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    Agent(AgentId),
    Structure(StructureId),
    Node(NodeId),
    Creature(CreatureId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InspectorTab {
    Mind,
    Needs,
    Bonds,
    Life,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiAction {
    OpenNewColony,
    BackToTitle,
    ContinueSave,
    StartColony,
    SetupSite(String),
    SetupDifficulty(String),
    RerollSeed,
    Quit,
    SetSpeed(usize),
    TogglePause,
    Open(Overlay),
    CloseOverlay,
    ToggleBuild,
    BuildCategory(BuildingCategory),
    ArmTool(String),
    DisarmTool,
    PlaceAt(Tile),
    Select(Selection),
    Deselect,
    InspectorTab(InspectorTab),
    ToggleFollow,
    Command(Command),
    ZoomBy(f32),
    Recenter,
    JumpTo(Tile),
    ChronicleFilter(Option<Category>),
    ChronicleDay(Option<u32>),
    Scroll(f32),
    FocusTech(String),
    RelationFocus(Option<AgentId>),
    HelpPage(usize),
    TutorialNext,
    TutorialClose,
    ToggleTracker,
    SaveAndExit,
}
