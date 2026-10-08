//! Personality weights derived from traits, and trainable skills.

use crate::data::{game_data, Need, Skill, NEED_COUNT, SKILL_COUNT};
use serde::{Deserialize, Serialize};

/// The aggregated influence of a survivor's traits on every decision.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Personality {
    pub sociability: f32,
    pub bravery: f32,
    pub kindness: f32,
    pub diligence: f32,
    pub curiosity: f32,
    pub romance: f32,
    pub temper: f32,
    pub optimism: f32,
    pub leadership: f32,
    pub night_owl: bool,
    pub need_decay: [f32; NEED_COUNT],
    pub skill_speed: [f32; SKILL_COUNT],
}

impl Personality {
    pub fn from_traits(traits: &[String]) -> Self {
        let data = game_data();
        let mut personality = Personality {
            need_decay: [1.0; NEED_COUNT],
            skill_speed: [1.0; SKILL_COUNT],
            ..Default::default()
        };
        for trait_id in traits {
            let Some(def) = data.trait_def(trait_id) else {
                continue;
            };
            let m = &def.modifiers;
            personality.sociability += m.sociability;
            personality.bravery += m.bravery;
            personality.kindness += m.kindness;
            personality.diligence += m.diligence;
            personality.curiosity += m.curiosity;
            personality.romance += m.romance;
            personality.temper += m.temper;
            personality.optimism += m.optimism;
            personality.leadership += m.leadership;
            personality.night_owl |= m.night_owl;
            for (need, amount) in &m.need_decay {
                personality.need_decay[need.index()] += amount;
            }
            for (skill, amount) in &m.skill_speed {
                personality.skill_speed[skill.index()] += amount;
            }
        }
        for decay in personality.need_decay.iter_mut() {
            *decay = decay.max(0.2);
        }
        personality
    }

    pub fn decay(&self, need: Need) -> f32 {
        self.need_decay[need.index()]
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Skills {
    pub levels: [u8; SKILL_COUNT],
    pub xp: [f32; SKILL_COUNT],
    pub passions: Vec<Skill>,
}

impl Skills {
    pub fn level(&self, skill: Skill) -> u8 {
        self.levels[skill.index()]
    }

    pub fn has_passion(&self, skill: Skill) -> bool {
        self.passions.contains(&skill)
    }

    /// Multiplier on work speed for a skill.
    pub fn speed(&self, skill: Skill, personality: &Personality) -> f32 {
        let per_level = game_data().balance.work.skill_speed_per_level;
        (0.6 + self.level(skill) as f32 * per_level) * personality.skill_speed[skill.index()]
    }

    /// Add practice; returns true when the skill gains a level.
    pub fn practise(&mut self, skill: Skill, ticks: f32) -> bool {
        let work = &game_data().balance.work;
        let index = skill.index();
        let passion = if self.has_passion(skill) {
            work.passion_xp_multiplier
        } else {
            1.0
        };
        self.xp[index] += ticks * work.xp_per_tick * passion;
        let needed = work.xp_per_level * (1.0 + self.levels[index] as f32 * 0.15);
        if self.xp[index] >= needed && self.levels[index] < 20 {
            self.xp[index] -= needed;
            self.levels[index] += 1;
            return true;
        }
        false
    }

    pub fn best(&self) -> Skill {
        Skill::ALL
            .into_iter()
            .max_by_key(|skill| self.level(*skill))
            .unwrap_or(Skill::Construction)
    }
}
