//! Balance constants loaded from `assets/data/balance.json`.

use super::kinds::{CreatureKind, Need};
use super::resources::Resource;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Balance {
    pub time: TimeBalance,
    pub needs: NeedBalance,
    pub agents: AgentBalance,
    pub social: SocialBalance,
    pub mood: MoodBalance,
    pub health: HealthBalance,
    pub work: WorkBalance,
    pub research: ResearchBalance,
    pub farming: FarmBalance,
    pub expeditions: ExpeditionBalance,
    pub creatures: BTreeMap<CreatureKind, CreatureBalance>,
    pub colony: ColonyBalance,
    pub map: MapBalance,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeBalance {
    pub ticks_per_day: u32,
    pub days_per_season: u32,
    pub campaign_years: u32,
    /// Simulation ticks per real second for pause, 1x, 2x and 3x.
    pub speed_ticks_per_second: [f32; 4],
    pub dawn_hour: f32,
    pub dusk_hour: f32,
    pub work_start_hour: f32,
    pub work_end_hour: f32,
    pub sleep_hour: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeedBalance {
    /// Points lost per day at baseline (100 is fully satisfied).
    pub decay_per_day: BTreeMap<Need, f32>,
    pub comfortable_temperature: f32,
    /// Extra warmth loss per day for each degree below comfortable outdoors.
    pub cold_loss_per_degree: f32,
    pub shelter_warmth_gain_per_day: f32,
    pub heat_warmth_gain_per_day: f32,
    pub meal_restore: f32,
    pub raw_food_restore: f32,
    pub forage_restore: f32,
    pub bed_rest_per_day: f32,
    pub ground_rest_per_day: f32,
    pub recreation_gain_per_day: f32,
    pub safety_recovery_per_day: f32,
    pub safety_loss_on_threat: f32,
    pub urgent_threshold: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentBalance {
    pub tiles_per_tick: f32,
    pub sight_radius: f32,
    pub night_sight_factor: f32,
    pub storm_sight_factor: f32,
    pub perception_interval: u32,
    pub deliberation_interval: u32,
    pub commitment_bonus: f32,
    pub max_plan_attempts: u32,
    pub goal_backoff_ticks: u32,
    pub carry_capacity: f32,
    pub belief_forget_days: f32,
    pub help_wait_ticks: u32,
    pub child_growth_days: u32,
    pub adult_age_years: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SocialBalance {
    pub chat_ticks: u32,
    pub chat_social_gain: f32,
    pub chat_opinion_range: [f32; 2],
    pub familiarity_per_chat: f32,
    pub proximity_familiarity_per_day: f32,
    pub shared_work_opinion_per_day: f32,
    pub friend_threshold: f32,
    pub close_friend_threshold: f32,
    pub rival_threshold: f32,
    pub enemy_threshold: f32,
    pub romance_gain_per_flirt: f32,
    pub partner_romance_threshold: f32,
    pub gossip_weight: f32,
    pub trait_clash_penalty: f32,
    pub trait_bond_bonus: f32,
    pub child_chance_per_day: f32,
    pub opinion_decay_per_day: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoodBalance {
    pub baseline: f32,
    /// Expectations rise as the colony matures: mood lost per campaign act.
    pub expectation_per_act: f32,
    pub need_weight: f32,
    pub break_threshold: f32,
    pub extreme_threshold: f32,
    pub break_chance_per_day: f32,
    pub break_ticks: u32,
    pub departure_threshold: f32,
    pub departure_days: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthBalance {
    pub regen_per_day: f32,
    pub care_regen_per_day: f32,
    pub illness_damage_per_day: f32,
    pub illness_recovery_chance_per_day: f32,
    pub treated_recovery_bonus: f32,
    pub starvation_damage_per_day: f32,
    pub hypothermia_damage_per_day: f32,
    pub care_threshold: f32,
    pub old_age_years: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkBalance {
    pub construction_per_tick: f32,
    pub harvest_per_tick: BTreeMap<Resource, f32>,
    pub cook_ticks: u32,
    pub meals_per_food: f32,
    pub skill_speed_per_level: f32,
    pub xp_per_tick: f32,
    pub xp_per_level: f32,
    pub passion_xp_multiplier: f32,
    pub severe_weather_outdoor_factor: f32,
    pub unpowered_factor: f32,
    pub decay_per_day: f32,
    pub storm_damage: f32,
    pub repair_per_tick: f32,
    pub meal_stock_per_person: f32,
    /// Fraction of stored food and meals that spoils each day.
    pub food_spoilage_per_day: f32,
    pub meal_spoilage_per_day: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearchBalance {
    pub points_per_tick: f32,
    pub insight_per_tick: f32,
    pub eureka_threshold: f32,
    pub practice_discount_per_tech: f32,
    pub max_practice_discount: f32,
    pub hidden_fraction: f32,
    pub cross_branch_chance: f32,
    pub relic_study_points: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FarmBalance {
    pub plant_work: f32,
    pub harvest_work: f32,
    pub season_growth: [f32; 4],
    pub frost_kill_temperature: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpeditionBalance {
    pub min_party: usize,
    pub max_party: usize,
    pub volunteer_window_ticks: u32,
    pub injury_chance_per_danger: f32,
    pub death_chance_per_danger: f32,
    pub reward_scale: f32,
    pub recruit_chance: f32,
    pub tech_chance: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatureBalance {
    pub health: f32,
    pub damage_per_tick: f32,
    pub tiles_per_tick: f32,
    pub raid_amount: f32,
    pub aggressive: bool,
    pub linger_ticks: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColonyBalance {
    pub starting_survivors: usize,
    pub starting_resources: BTreeMap<Resource, f32>,
    pub collapse_mood: f32,
    pub collapse_days: u32,
    pub daily_broadcast_hour: f32,
    pub max_population: usize,
    pub defence_damage_per_tick: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MapBalance {
    pub width: i32,
    pub height: i32,
    pub tile_size: f32,
    pub clearing_radius: i32,
    pub node_regrow_per_day: f32,
}
