//! The colony AI's chronicle: a searchable history and daily summaries.

use crate::world::AgentId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Story,
    Relationship,
    Discovery,
    Danger,
    Loss,
    Life,
    Colony,
    Mind,
}

impl Category {
    pub const ALL: [Category; 8] = [
        Category::Story,
        Category::Relationship,
        Category::Discovery,
        Category::Danger,
        Category::Loss,
        Category::Life,
        Category::Colony,
        Category::Mind,
    ];

    pub fn label_key(self) -> &'static str {
        match self {
            Category::Story => "cat_story",
            Category::Relationship => "cat_relationship",
            Category::Discovery => "cat_discovery",
            Category::Danger => "cat_danger",
            Category::Loss => "cat_loss",
            Category::Life => "cat_life",
            Category::Colony => "cat_colony",
            Category::Mind => "cat_mind",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub day: u32,
    pub hour: u8,
    pub category: Category,
    pub text: String,
    pub agents: Vec<AgentId>,
    /// 0 routine, 1 notable, 2 major (shown as a toast).
    pub importance: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyReport {
    pub day: u32,
    pub lines: Vec<String>,
    pub mood: f32,
    pub population: usize,
}

const MAX_ENTRIES: usize = 6000;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Chronicle {
    pub entries: Vec<Entry>,
    pub reports: Vec<DailyReport>,
}

impl Chronicle {
    pub fn record(&mut self, entry: Entry) {
        self.entries.push(entry);
        if self.entries.len() > MAX_ENTRIES {
            // Drop the oldest routine entries first so major moments survive.
            if let Some(index) = self.entries.iter().position(|e| e.importance == 0) {
                self.entries.remove(index);
            } else {
                self.entries.remove(0);
            }
        }
    }

    pub fn entries_on(&self, day: u32) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(move |entry| entry.day == day)
    }

    pub fn latest_report(&self) -> Option<&DailyReport> {
        self.reports.last()
    }

    pub fn about(&self, agent: AgentId) -> impl Iterator<Item = &Entry> {
        self.entries
            .iter()
            .rev()
            .filter(move |entry| entry.agents.contains(&agent))
    }
}
