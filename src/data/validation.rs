//! Semantic validation of loaded game data: references, ranges and invariants.

use super::campaign::ObjectiveKind;
use super::catalog::GameData;
use std::collections::HashSet;

/// Required thought ids the simulation records by name.
pub const REQUIRED_THOUGHTS: &[&str] = &[
    "ate_meal",
    "ate_raw",
    "ate_foraged",
    "went_hungry",
    "slept_bed",
    "slept_ground",
    "freezing",
    "good_chat",
    "insulted",
    "comforted",
    "new_partner",
    "breakup",
    "friend_died",
    "partner_died",
    "ambition_fulfilled",
    "frustrated",
    "eureka",
    "expedition_home",
    "injured",
    "sick",
    "child_born",
    "vote_won",
    "vote_lost",
    "witnessed_attack",
    "helped_friend",
    "refused_help",
    "beautiful_place",
    "landed",
    "lonely",
    "learned",
];

fn check(condition: bool, message: impl FnOnce() -> String, errors: &mut Vec<String>) {
    if !condition {
        errors.push(message());
    }
}

fn unique_ids<'a>(label: &str, ids: impl Iterator<Item = &'a str>, errors: &mut Vec<String>) {
    let mut seen = HashSet::new();
    for id in ids {
        check(
            seen.insert(id),
            || format!("duplicate {label} id '{id}'"),
            errors,
        );
    }
}

pub fn validate(data: &GameData) -> Result<(), String> {
    let mut errors = Vec::new();
    validate_balance(data, &mut errors);
    validate_buildings(data, &mut errors);
    validate_techs(data, &mut errors);
    validate_finds(data, &mut errors);
    validate_people(data, &mut errors);
    validate_events(data, &mut errors);
    validate_campaign(data, &mut errors);
    validate_society(data, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn validate_balance(data: &GameData, errors: &mut Vec<String>) {
    let balance = &data.balance;
    check(
        balance.time.ticks_per_day >= 24,
        || "ticks_per_day too small".into(),
        errors,
    );
    check(
        balance.time.days_per_season > 0,
        || "days_per_season must be positive".into(),
        errors,
    );
    check(
        balance.map.width >= 48 && balance.map.height >= 32,
        || "map too small".into(),
        errors,
    );
    check(
        balance.agents.carry_capacity > 0.0,
        || "carry capacity must be positive".into(),
        errors,
    );
    check(
        balance.expeditions.min_party <= balance.expeditions.max_party,
        || "expedition party bounds inverted".into(),
        errors,
    );
}

fn validate_buildings(data: &GameData, errors: &mut Vec<String>) {
    unique_ids(
        "building",
        data.buildings.iter().map(|b| b.id.as_str()),
        errors,
    );
    unique_ids("recipe", data.recipes.iter().map(|r| r.id.as_str()), errors);
    let stations: HashSet<&str> = data
        .buildings
        .iter()
        .filter_map(|b| b.station.as_deref())
        .collect();
    for building in &data.buildings {
        check(
            building.size[0] > 0 && building.size[1] > 0,
            || format!("building '{}' has an empty footprint", building.id),
            errors,
        );
        if let Some(tech) = &building.tech {
            check(
                data.tech(tech).is_some(),
                || format!("building '{}' needs unknown tech '{tech}'", building.id),
                errors,
            );
        }
        if let Some(ending) = &building.capstone {
            check(
                data.ending(ending).is_some(),
                || format!("building '{}' names unknown ending '{ending}'", building.id),
                errors,
            );
        }
    }
    for recipe in &data.recipes {
        check(
            stations.contains(recipe.station.as_str()),
            || {
                format!(
                    "recipe '{}' uses unknown station '{}'",
                    recipe.id, recipe.station
                )
            },
            errors,
        );
        if let Some(tech) = &recipe.tech {
            check(
                data.tech(tech).is_some(),
                || format!("recipe '{}' needs unknown tech '{tech}'", recipe.id),
                errors,
            );
        }
        check(
            !recipe.outputs.is_empty(),
            || format!("recipe '{}' produces nothing", recipe.id),
            errors,
        );
    }
}

fn validate_techs(data: &GameData, errors: &mut Vec<String>) {
    unique_ids("tech", data.techs.iter().map(|t| t.id.as_str()), errors);
    for tech in &data.techs {
        check(
            (1..=5).contains(&tech.tier),
            || format!("tech '{}' tier out of range", tech.id),
            errors,
        );
        for prereq in &tech.fixed_prereqs {
            let valid = data
                .tech(prereq)
                .is_some_and(|other| other.tier < tech.tier);
            check(
                valid,
                || format!("tech '{}' has invalid prerequisite '{prereq}'", tech.id),
                errors,
            );
        }
        if let Some(ending) = &tech.capstone {
            check(
                data.ending(ending).is_some(),
                || format!("tech '{}' names unknown ending '{ending}'", tech.id),
                errors,
            );
        }
    }
    for tier in 1..=5u8 {
        check(
            data.techs.iter().any(|tech| tech.tier == tier),
            || format!("tech tier {tier} is empty"),
            errors,
        );
    }
}

/// Inspired technologies, and the species that inspire them. A technology
/// that unlocks content must be inspirable on every landing: it lists enough
/// species of a kind that any drawn set includes one of them.
fn validate_finds(data: &GameData, errors: &mut Vec<String>) {
    unique_ids("find", data.finds.iter().map(|f| f.id.as_str()), errors);
    const TERRAIN: [&str; 5] = ["grass", "soil", "sand", "water", "rock"];
    for find in &data.finds {
        check(
            find.yield_scale > 0.0,
            || format!("find '{}' has no yield", find.id),
            errors,
        );
        if let Some(near) = &find.near {
            check(
                TERRAIN.contains(&near.as_str()),
                || format!("find '{}' prefers unknown terrain '{near}'", find.id),
                errors,
            );
        }
    }
    for (kind, count) in &data.species_per_run {
        let available = data.finds_of(*kind).count();
        check(
            *count >= 1 && *count <= available,
            || {
                format!(
                    "{} species per run exceeds the {available} defined",
                    kind.key()
                )
            },
            errors,
        );
    }
    for tech in data.techs.iter().filter(|t| t.is_inspired()) {
        check(
            !tech.always_known && !tech.relic_only && tech.capstone.is_none(),
            || format!("tech '{}' cannot be both inspired and fixed", tech.id),
            errors,
        );
        for id in &tech.inspired_by {
            check(
                data.find(id).is_some(),
                || format!("tech '{}' is inspired by unknown find '{id}'", tech.id),
                errors,
            );
        }
        let required = data
            .techs
            .iter()
            .any(|t| t.fixed_prereqs.contains(&tech.id));
        check(
            !required,
            || format!("inspired tech '{}' cannot be a fixed prerequisite", tech.id),
            errors,
        );
        if data.tech_unlocks_content(&tech.id) {
            let guaranteed = data.species_per_run.iter().any(|(kind, drawn)| {
                let total = data.finds_of(*kind).count();
                let listed = data
                    .finds_of(*kind)
                    .filter(|f| tech.inspired_by.contains(&f.id))
                    .count();
                listed + drawn > total
            });
            check(
                guaranteed,
                || {
                    format!(
                        "tech '{}' unlocks content but may never be inspired",
                        tech.id
                    )
                },
                errors,
            );
        }
    }
}

fn validate_people(data: &GameData, errors: &mut Vec<String>) {
    unique_ids("trait", data.traits.iter().map(|t| t.id.as_str()), errors);
    for trait_def in &data.traits {
        for other in trait_def
            .excludes
            .iter()
            .chain(&trait_def.clashes_with)
            .chain(&trait_def.bonds_with)
        {
            check(
                data.trait_def(other).is_some(),
                || {
                    format!(
                        "trait '{}' references unknown trait '{other}'",
                        trait_def.id
                    )
                },
                errors,
            );
        }
    }
    check(
        !data.ambitions.is_empty(),
        || "no ambitions defined".into(),
        errors,
    );
    check(
        data.names.given.len() >= 20 && data.names.family.len() >= 20,
        || "name lists are too short for generated crews".into(),
        errors,
    );
}

fn validate_events(data: &GameData, errors: &mut Vec<String>) {
    unique_ids("event", data.events.iter().map(|e| e.id.as_str()), errors);
    for event in &data.events {
        check(
            event.weight >= 0.0,
            || format!("event '{}' has negative weight", event.id),
            errors,
        );
        for site in &event.sites {
            check(
                data.site(site).is_some(),
                || format!("event '{}' names unknown site '{site}'", event.id),
                errors,
            );
        }
    }
    unique_ids(
        "expedition site",
        data.expedition_sites.iter().map(|s| s.id.as_str()),
        errors,
    );
}

fn validate_campaign(data: &GameData, errors: &mut Vec<String>) {
    let campaign = &data.campaign;
    check(
        campaign.acts.len() == 5,
        || "the campaign needs five acts".into(),
        errors,
    );
    check(
        campaign.endings.len() == 3,
        || "the campaign needs three endings".into(),
        errors,
    );
    check(
        !campaign.sites.is_empty(),
        || "no landing sites".into(),
        errors,
    );
    check(
        !campaign.difficulties.is_empty(),
        || "no difficulties".into(),
        errors,
    );
    for act in &campaign.acts {
        for event in act.opening_event.iter().chain(&act.climax_event) {
            check(
                data.event(event).is_some(),
                || format!("act {} names unknown event '{event}'", act.number),
                errors,
            );
        }
        for objective in &act.objectives {
            if let ObjectiveKind::Structure { building, .. } = &objective.kind {
                check(
                    data.building(building).is_some(),
                    || format!("objective '{}' names unknown building", objective.id),
                    errors,
                );
            }
        }
    }
    for ending in &campaign.endings {
        check(
            data.tech(&ending.capstone_tech).is_some(),
            || format!("ending '{}' has unknown capstone tech", ending.id),
            errors,
        );
        check(
            data.building(&ending.capstone_building).is_some(),
            || format!("ending '{}' has unknown capstone building", ending.id),
            errors,
        );
        check(
            data.event(&ending.final_crisis_event).is_some(),
            || format!("ending '{}' has unknown crisis event", ending.id),
            errors,
        );
    }
}

fn validate_society(data: &GameData, errors: &mut Vec<String>) {
    unique_ids(
        "thought",
        data.society.thoughts.iter().map(|t| t.id.as_str()),
        errors,
    );
    for required in REQUIRED_THOUGHTS {
        check(
            data.thought(required).is_some(),
            || format!("required thought '{required}' is missing"),
            errors,
        );
    }
    for policy in &data.society.policies {
        check(
            policy.options.iter().any(|o| o.id == policy.default_option),
            || format!("policy '{}' default option is missing", policy.id),
            errors,
        );
        if let Some(tech) = &policy.tech {
            check(
                data.tech(tech).is_some(),
                || format!("policy '{}' needs unknown tech '{tech}'", policy.id),
                errors,
            );
        }
    }
}
