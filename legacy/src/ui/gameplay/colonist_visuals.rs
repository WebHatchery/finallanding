//! colonist visuals domain.

use super::*;

pub fn job_color(job_preference: crate::data::colonist::JobPreference) -> Color {
    match job_preference {
        crate::data::colonist::JobPreference::Explorer => PURPLE,
        crate::data::colonist::JobPreference::Builder => YELLOW,
        crate::data::colonist::JobPreference::Cook => GREEN,
        crate::data::colonist::JobPreference::Hauler => GRAY,
        crate::data::colonist::JobPreference::None => WHITE,
    }
}

pub fn colonist_activity_summary(colonist: &Colonist) -> &'static str {
    match colonist.state {
        ColonistState::Idle => "Idle",
        ColonistState::Moving { .. } => "Moving",
        ColonistState::Working => "Working",
        ColonistState::Eating => "Eating",
        ColonistState::Sleeping => "Resting",
        ColonistState::OnMission { .. } => "On mission",
    }
}

pub fn sprite_pose_for_state(state: ColonistState) -> SpritePose {
    match state {
        ColonistState::Idle => SpritePose::Idle,
        ColonistState::Moving { .. } => SpritePose::Moving,
        ColonistState::Working => SpritePose::Working,
        ColonistState::Eating => SpritePose::Eating,
        ColonistState::Sleeping => SpritePose::Sleeping,
        ColonistState::OnMission { .. } => SpritePose::Moving,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocialBodyLanguage {
    Supported(i32),
    Tense(i32),
}

impl SocialBodyLanguage {
    pub fn intensity(self) -> i32 {
        match self {
            SocialBodyLanguage::Supported(value) | SocialBodyLanguage::Tense(value) => value.abs(),
        }
    }

    pub fn color(self, alpha: f32) -> Color {
        match self {
            SocialBodyLanguage::Supported(_) => Color::new(
                style::BAR_GREEN.r,
                style::BAR_GREEN.g,
                style::BAR_GREEN.b,
                alpha,
            ),
            SocialBodyLanguage::Tense(_) => Color::new(
                style::ALERT_RED.r,
                style::ALERT_RED.g,
                style::ALERT_RED.b,
                alpha,
            ),
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            SocialBodyLanguage::Supported(_) => "+",
            SocialBodyLanguage::Tense(_) => "!",
        }
    }
}

pub fn sprite_pose_for_colonist_frame(
    colonist: &Colonist,
    social_signal: Option<SocialBodyLanguage>,
    tick: u64,
) -> SpritePose {
    if let Some(signal) = social_signal {
        return match signal {
            SocialBodyLanguage::Supported(_) => {
                if social_pose_uses_alternate_frame(tick) {
                    SpritePose::SupportedReach
                } else {
                    SpritePose::Supported
                }
            }
            SocialBodyLanguage::Tense(_) => {
                if social_pose_uses_alternate_frame(tick) {
                    SpritePose::TenseGuarded
                } else {
                    SpritePose::Tense
                }
            }
        };
    }

    sprite_pose_for_state(colonist.state)
}

pub fn social_pose_uses_alternate_frame(tick: u64) -> bool {
    (tick / 45) % 2 == 1
}

pub fn strongest_relationship_value(colonist: &Colonist) -> Option<i32> {
    colonist
        .relationships
        .values()
        .max_by_key(|value| value.abs())
        .copied()
}

pub fn social_color(value: i32, alpha: f32) -> Color {
    style::relationship_color_alpha(value, alpha)
}
