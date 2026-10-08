//! Small closed vocabularies shared by data files and the simulation.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Skill {
    Construction,
    Farming,
    Cooking,
    Science,
    Medicine,
    Crafting,
    Exploration,
    Social,
}

pub const SKILL_COUNT: usize = 8;

impl Skill {
    pub const ALL: [Skill; SKILL_COUNT] = [
        Skill::Construction,
        Skill::Farming,
        Skill::Cooking,
        Skill::Science,
        Skill::Medicine,
        Skill::Crafting,
        Skill::Exploration,
        Skill::Social,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn key(self) -> &'static str {
        match self {
            Skill::Construction => "construction",
            Skill::Farming => "farming",
            Skill::Cooking => "cooking",
            Skill::Science => "science",
            Skill::Medicine => "medicine",
            Skill::Crafting => "crafting",
            Skill::Exploration => "exploration",
            Skill::Social => "social",
        }
    }

    /// The research branch this kind of practice gives insight into.
    pub fn branch(self) -> Branch {
        match self {
            Skill::Construction | Skill::Crafting => Branch::Industry,
            Skill::Farming => Branch::Agronomy,
            Skill::Cooking | Skill::Exploration => Branch::Survival,
            Skill::Science => Branch::Xenology,
            Skill::Medicine => Branch::Medicine,
            Skill::Social => Branch::Society,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Branch {
    Survival,
    Agronomy,
    Industry,
    Medicine,
    Society,
    Xenology,
}

pub const BRANCH_COUNT: usize = 6;

impl Branch {
    pub const ALL: [Branch; BRANCH_COUNT] = [
        Branch::Survival,
        Branch::Agronomy,
        Branch::Industry,
        Branch::Medicine,
        Branch::Society,
        Branch::Xenology,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn key(self) -> &'static str {
        match self {
            Branch::Survival => "survival",
            Branch::Agronomy => "agronomy",
            Branch::Industry => "industry",
            Branch::Medicine => "medicine",
            Branch::Society => "society",
            Branch::Xenology => "xenology",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Need {
    Food,
    Rest,
    Warmth,
    Social,
    Recreation,
    Safety,
}

pub const NEED_COUNT: usize = 6;

impl Need {
    pub const ALL: [Need; NEED_COUNT] = [
        Need::Food,
        Need::Rest,
        Need::Warmth,
        Need::Social,
        Need::Recreation,
        Need::Safety,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn key(self) -> &'static str {
        match self {
            Need::Food => "food",
            Need::Rest => "rest",
            Need::Warmth => "warmth",
            Need::Social => "social",
            Need::Recreation => "recreation",
            Need::Safety => "safety",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub const ALL: [Season; 4] = [
        Season::Spring,
        Season::Summer,
        Season::Autumn,
        Season::Winter,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn key(self) -> &'static str {
        match self {
            Season::Spring => "spring",
            Season::Summer => "summer",
            Season::Autumn => "autumn",
            Season::Winter => "winter",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Weather {
    Clear,
    Overcast,
    Rain,
    Storm,
    Snow,
    Heatwave,
    Ashfall,
}

impl Weather {
    pub fn key(self) -> &'static str {
        match self {
            Weather::Clear => "clear",
            Weather::Overcast => "overcast",
            Weather::Rain => "rain",
            Weather::Storm => "storm",
            Weather::Snow => "snow",
            Weather::Heatwave => "heatwave",
            Weather::Ashfall => "ashfall",
        }
    }

    /// Outdoor work and travel are slowed or dangerous in severe weather.
    pub fn is_severe(self) -> bool {
        matches!(self, Weather::Storm | Weather::Ashfall)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingCategory {
    Shelter,
    Food,
    Industry,
    Science,
    Care,
    Power,
    Social,
    Defence,
    Expedition,
    Capstone,
}

impl BuildingCategory {
    pub const ALL: [BuildingCategory; 10] = [
        BuildingCategory::Shelter,
        BuildingCategory::Food,
        BuildingCategory::Industry,
        BuildingCategory::Science,
        BuildingCategory::Care,
        BuildingCategory::Power,
        BuildingCategory::Social,
        BuildingCategory::Defence,
        BuildingCategory::Expedition,
        BuildingCategory::Capstone,
    ];

    pub fn key(self) -> &'static str {
        match self {
            BuildingCategory::Shelter => "shelter",
            BuildingCategory::Food => "food",
            BuildingCategory::Industry => "industry",
            BuildingCategory::Science => "science",
            BuildingCategory::Care => "care",
            BuildingCategory::Power => "power",
            BuildingCategory::Social => "social",
            BuildingCategory::Defence => "defence",
            BuildingCategory::Expedition => "expedition",
            BuildingCategory::Capstone => "capstone",
        }
    }
}

/// Harvestable features of the landscape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Wreckage,
    FibreGrove,
    StoneOutcrop,
    OreVein,
    Forage,
    Ruin,
}

impl NodeKind {
    pub const ALL: [NodeKind; 6] = [
        NodeKind::Wreckage,
        NodeKind::FibreGrove,
        NodeKind::StoneOutcrop,
        NodeKind::OreVein,
        NodeKind::Forage,
        NodeKind::Ruin,
    ];

    pub fn key(self) -> &'static str {
        match self {
            NodeKind::Wreckage => "wreckage",
            NodeKind::FibreGrove => "fibre_grove",
            NodeKind::StoneOutcrop => "stone_outcrop",
            NodeKind::OreVein => "ore_vein",
            NodeKind::Forage => "forage",
            NodeKind::Ruin => "ruin",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreatureKind {
    Skitter,
    Ridgeback,
    Driftmaw,
}

impl CreatureKind {
    pub fn key(self) -> &'static str {
        match self {
            CreatureKind::Skitter => "skitter",
            CreatureKind::Ridgeback => "ridgeback",
            CreatureKind::Driftmaw => "driftmaw",
        }
    }
}
