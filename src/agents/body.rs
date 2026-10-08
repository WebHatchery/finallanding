//! Needs, health and the thoughts that make up a survivor's mood.

use crate::data::{game_data, Need, NEED_COUNT};
use serde::{Deserialize, Serialize};

/// Need satisfaction from 0 (desperate) to 100 (content).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Needs {
    pub values: [f32; NEED_COUNT],
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            values: [80.0; NEED_COUNT],
        }
    }
}

impl Needs {
    pub fn get(&self, need: Need) -> f32 {
        self.values[need.index()]
    }

    pub fn change(&mut self, need: Need, amount: f32) {
        let value = &mut self.values[need.index()];
        *value = (*value + amount).clamp(0.0, 100.0);
    }

    pub fn set(&mut self, need: Need, amount: f32) {
        self.values[need.index()] = amount.clamp(0.0, 100.0);
    }

    pub fn lowest(&self) -> (Need, f32) {
        Need::ALL
            .into_iter()
            .map(|need| (need, self.get(need)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap_or((Need::Food, 100.0))
    }

    /// Mood contribution of unmet needs: negative when needs are low.
    pub fn mood_pressure(&self) -> f32 {
        let weight = game_data().balance.mood.need_weight;
        Need::ALL
            .into_iter()
            .map(|need| (self.get(need) - 60.0).min(5.0) * weight / NEED_COUNT as f32 * 2.0)
            .sum()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Health {
    pub hp: f32,
    pub ill: bool,
    pub treated: bool,
    pub injuries: u32,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            hp: 100.0,
            ill: false,
            treated: false,
            injuries: 0,
        }
    }
}

impl Health {
    pub fn needs_care(&self) -> bool {
        self.ill || self.hp < game_data().balance.health.care_threshold
    }

    pub fn damage(&mut self, amount: f32) {
        self.hp = (self.hp - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.hp = (self.hp + amount).min(100.0);
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0.0
    }

    /// Work speed lost to pain and fever.
    pub fn capacity(&self) -> f32 {
        let base = 0.4 + 0.6 * (self.hp / 100.0);
        if self.ill {
            base * 0.7
        } else {
            base
        }
    }
}

/// A memory with a mood effect that fades after its duration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Thought {
    pub id: String,
    pub mood: f32,
    pub remaining_ticks: u64,
    pub about: Option<u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Mind {
    pub thoughts: Vec<Thought>,
}

impl Mind {
    /// Record a thought by id, respecting the definition's stacking limit.
    pub fn add(&mut self, id: &str, about: Option<u32>, ticks_per_day: u64) {
        let Some(def) = game_data().thought(id) else {
            return;
        };
        let same: Vec<usize> = self
            .thoughts
            .iter()
            .enumerate()
            .filter(|(_, thought)| thought.id == id)
            .map(|(index, _)| index)
            .collect();
        let duration = (def.days * ticks_per_day as f32) as u64;
        if same.len() as u32 >= def.max_stack {
            if let Some(oldest) = same.first() {
                self.thoughts[*oldest].remaining_ticks = duration;
                self.thoughts[*oldest].about = about;
            }
            return;
        }
        self.thoughts.push(Thought {
            id: id.to_owned(),
            mood: def.mood,
            remaining_ticks: duration,
            about,
        });
    }

    /// A thought with an explicit mood value (colony-wide events).
    pub fn add_custom(&mut self, id: &str, mood: f32, ticks: u64) {
        self.thoughts
            .retain(|thought| thought.id != id || thought.mood.signum() != mood.signum());
        self.thoughts.push(Thought {
            id: id.to_owned(),
            mood,
            remaining_ticks: ticks,
            about: None,
        });
    }

    pub fn tick(&mut self, ticks: u64) {
        for thought in self.thoughts.iter_mut() {
            thought.remaining_ticks = thought.remaining_ticks.saturating_sub(ticks);
        }
        self.thoughts.retain(|thought| thought.remaining_ticks > 0);
    }

    pub fn total(&self) -> f32 {
        self.thoughts.iter().map(|thought| thought.mood).sum()
    }

    pub fn has(&self, id: &str) -> bool {
        self.thoughts.iter().any(|thought| thought.id == id)
    }

    pub fn remove(&mut self, id: &str) {
        self.thoughts.retain(|thought| thought.id != id);
    }
}
