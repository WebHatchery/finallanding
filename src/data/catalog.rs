//! The immutable game-data catalog: every JSON file, loaded once and indexed.

use super::balance::Balance;
use super::buildings::{BuildingDef, RecipeDef};
use super::campaign::{ActDef, CampaignDef, DifficultyDef, EndingDef, SiteDef};
use super::events::{EventDef, ExpeditionSiteDef};
use super::people::{AmbitionDef, NameLists, TraitDef};
use super::society::{PolicyDef, SocietyDef, TextDef, ThoughtDef};
use super::techs::TechDef;
use super::validation;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Deserialize)]
struct BuildingFile {
    buildings: Vec<BuildingDef>,
    recipes: Vec<RecipeDef>,
}

#[derive(Deserialize)]
struct TechFile {
    techs: Vec<TechDef>,
}

#[derive(Deserialize)]
struct PeopleFile {
    traits: Vec<TraitDef>,
    ambitions: Vec<AmbitionDef>,
    names: NameLists,
}

#[derive(Deserialize)]
struct EventFile {
    events: Vec<EventDef>,
    expedition_sites: Vec<ExpeditionSiteDef>,
}

pub struct GameData {
    pub balance: Balance,
    pub buildings: Vec<BuildingDef>,
    pub recipes: Vec<RecipeDef>,
    pub techs: Vec<TechDef>,
    pub traits: Vec<TraitDef>,
    pub ambitions: Vec<AmbitionDef>,
    pub names: NameLists,
    pub events: Vec<EventDef>,
    pub expedition_sites: Vec<ExpeditionSiteDef>,
    pub campaign: CampaignDef,
    pub society: SocietyDef,
    pub text: TextDef,
    building_index: HashMap<String, usize>,
    tech_index: HashMap<String, usize>,
    trait_index: HashMap<String, usize>,
    thought_index: HashMap<String, usize>,
    event_index: HashMap<String, usize>,
}

fn index_by<T>(items: &[T], id: impl Fn(&T) -> &str) -> HashMap<String, usize> {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| (id(item).to_owned(), index))
        .collect()
}

impl GameData {
    /// Parse and validate every embedded data file.
    pub fn load() -> Result<GameData, String> {
        let balance: Balance = macroquad_toolkit::include_json!("../../assets/data/balance.json")?;
        let buildings: BuildingFile =
            macroquad_toolkit::include_json!("../../assets/data/buildings.json")?;
        let techs: TechFile = macroquad_toolkit::include_json!("../../assets/data/techs.json")?;
        let people: PeopleFile = macroquad_toolkit::include_json!("../../assets/data/people.json")?;
        let events: EventFile = macroquad_toolkit::include_json!("../../assets/data/events.json")?;
        let campaign: CampaignDef =
            macroquad_toolkit::include_json!("../../assets/data/campaign.json")?;
        let society: SocietyDef =
            macroquad_toolkit::include_json!("../../assets/data/society.json")?;
        let text: TextDef = macroquad_toolkit::include_json!("../../assets/data/text.json")?;

        let data = GameData {
            building_index: index_by(&buildings.buildings, |b| &b.id),
            tech_index: index_by(&techs.techs, |t| &t.id),
            trait_index: index_by(&people.traits, |t| &t.id),
            thought_index: index_by(&society.thoughts, |t| &t.id),
            event_index: index_by(&events.events, |e| &e.id),
            balance,
            buildings: buildings.buildings,
            recipes: buildings.recipes,
            techs: techs.techs,
            traits: people.traits,
            ambitions: people.ambitions,
            names: people.names,
            events: events.events,
            expedition_sites: events.expedition_sites,
            campaign,
            society,
            text,
        };
        validation::validate(&data)?;
        Ok(data)
    }

    pub fn building(&self, id: &str) -> Option<&BuildingDef> {
        self.building_index
            .get(id)
            .map(|index| &self.buildings[*index])
    }

    pub fn tech(&self, id: &str) -> Option<&TechDef> {
        self.tech_index.get(id).map(|index| &self.techs[*index])
    }

    pub fn tech_position(&self, id: &str) -> Option<usize> {
        self.tech_index.get(id).copied()
    }

    pub fn trait_def(&self, id: &str) -> Option<&TraitDef> {
        self.trait_index.get(id).map(|index| &self.traits[*index])
    }

    pub fn thought(&self, id: &str) -> Option<&ThoughtDef> {
        self.thought_index
            .get(id)
            .map(|index| &self.society.thoughts[*index])
    }

    pub fn event(&self, id: &str) -> Option<&EventDef> {
        self.event_index.get(id).map(|index| &self.events[*index])
    }

    pub fn recipe(&self, id: &str) -> Option<&RecipeDef> {
        self.recipes.iter().find(|recipe| recipe.id == id)
    }

    pub fn policy(&self, id: &str) -> Option<&PolicyDef> {
        self.society.policies.iter().find(|policy| policy.id == id)
    }

    pub fn site(&self, id: &str) -> Option<&SiteDef> {
        self.campaign.sites.iter().find(|site| site.id == id)
    }

    pub fn difficulty(&self, id: &str) -> Option<&DifficultyDef> {
        self.campaign.difficulties.iter().find(|d| d.id == id)
    }

    pub fn ending(&self, id: &str) -> Option<&EndingDef> {
        self.campaign.endings.iter().find(|ending| ending.id == id)
    }

    pub fn act(&self, number: u8) -> Option<&ActDef> {
        self.campaign.acts.iter().find(|act| act.number == number)
    }

    pub fn expedition_site(&self, id: &str) -> Option<&ExpeditionSiteDef> {
        self.expedition_sites.iter().find(|site| site.id == id)
    }

    pub fn ambition(&self, id: &str) -> Option<&AmbitionDef> {
        self.ambitions.iter().find(|ambition| ambition.id == id)
    }

    /// UI label by key; a missing key falls back to the key itself so a gap is
    /// visible on screen rather than crashing the game.
    pub fn label<'a>(&'a self, key: &'a str) -> &'a str {
        self.text.labels.get(key).map(String::as_str).unwrap_or(key)
    }

    /// One chronicle template variant, chosen by `variant`.
    pub fn chronicle_template(&self, key: &str, variant: u64) -> Option<&str> {
        let lines = self.text.chronicle.get(key)?;
        if lines.is_empty() {
            return None;
        }
        Some(lines[(variant % lines.len() as u64) as usize].as_str())
    }
}

static GAME_DATA: OnceLock<GameData> = OnceLock::new();

/// The shared, validated game data. Embedded data that fails validation is a
/// build defect, so it panics with the validation message.
pub fn game_data() -> &'static GameData {
    GAME_DATA.get_or_init(|| match GameData::load() {
        Ok(data) => data,
        Err(error) => panic!("embedded game data is invalid: {error}"),
    })
}

/// Replace `{key}` placeholders in a template.
pub fn fill_template(template: &str, values: &[(&str, &str)]) -> String {
    let mut text = template.to_owned();
    for (key, value) in values {
        text = text.replace(&format!("{{{key}}}"), value);
    }
    text
}
