//! The colony's record of native species: when each was first found, by whom,
//! and how much of it has been gathered since. Gathering is what inspires
//! technologies, so this log decides the shape of each run's tree.

use crate::data::game_data;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FoundSpecies {
    pub id: String,
    pub day: u32,
    /// Who first gathered it, or where the sample came from.
    pub by: String,
    pub gathered: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FindLog {
    /// Species in the order the colony found them.
    pub found: Vec<FoundSpecies>,
}

impl FindLog {
    pub fn get(&self, id: &str) -> Option<&FoundSpecies> {
        self.found.iter().find(|f| f.id == id)
    }

    pub fn gathered(&self, id: &str) -> f32 {
        self.get(id).map_or(0.0, |f| f.gathered)
    }

    /// Total gathered across a set of species, as a technology's inspiration
    /// counts any of them.
    pub fn gathered_any(&self, ids: &[String]) -> f32 {
        ids.iter().map(|id| self.gathered(id)).sum()
    }

    /// Count gathered units of a species; true when it is the first find.
    pub fn record(&mut self, id: &str, amount: f32, day: u32, by: &str) -> bool {
        if let Some(found) = self.found.iter_mut().find(|f| f.id == id) {
            found.gathered += amount;
            return false;
        }
        self.found.push(FoundSpecies {
            id: id.to_owned(),
            day,
            by: by.to_owned(),
            gathered: amount,
        });
        true
    }

    /// The plantable food the colony knows best, which a native orchard grows.
    pub fn favourite_crop(&self) -> Option<&str> {
        self.found
            .iter()
            .filter(|f| game_data().find(&f.id).is_some_and(|d| d.crop.is_some()))
            .max_by(|a, b| a.gathered.total_cmp(&b.gathered))
            .map(|f| f.id.as_str())
    }
}
