//! Game data loads, validates, and covers every key the code asks for.

use finallanding::data::game_data;
use std::fs;
use std::path::Path;

fn source_files(dir: &Path, out: &mut Vec<String>) {
    for entry in fs::read_dir(dir)
        .expect("source directory is readable")
        .flatten()
    {
        let path = entry.path();
        if path.is_dir() {
            source_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(fs::read_to_string(&path).expect("source file is readable"));
        }
    }
}

/// Every `"key"` literal passed to the named function across `src/`.
fn literal_arguments(function: &str) -> Vec<String> {
    let mut sources = Vec::new();
    source_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut sources,
    );
    let needle = format!("{function}(\"");
    let mut keys = Vec::new();
    for source in sources {
        let mut rest = source.as_str();
        while let Some(start) = rest.find(&needle) {
            rest = &rest[start + needle.len()..];
            if let Some(end) = rest.find('"') {
                keys.push(rest[..end].to_owned());
            }
        }
    }
    keys.sort();
    keys.dedup();
    keys
}

#[test]
fn embedded_data_loads_and_validates() {
    let data = game_data();
    assert_eq!(data.campaign.acts.len(), 5);
    assert_eq!(data.campaign.endings.len(), 3);
    assert!(data.techs.len() >= 50, "a deep technology pool");
    assert!(data.buildings.len() >= 30, "a broad building catalogue");
    assert!(data.events.len() >= 30, "a varied event deck");
}

#[test]
fn every_ui_label_the_code_uses_exists() {
    let data = game_data();
    let missing: Vec<String> = literal_arguments("label")
        .into_iter()
        .filter(|key| !data.text.labels.contains_key(key))
        .collect();
    assert!(
        missing.is_empty(),
        "labels missing from text.json: {missing:?}"
    );
}

#[test]
fn every_chronicle_template_the_code_uses_exists() {
    let data = game_data();
    let missing: Vec<String> = literal_arguments("line")
        .into_iter()
        .filter(|key| !data.text.chronicle.contains_key(key))
        .collect();
    assert!(
        missing.is_empty(),
        "chronicle templates missing: {missing:?}"
    );
}

#[test]
fn every_plan_in_the_library_has_a_readable_name() {
    let data = game_data();
    let missing: Vec<String> = literal_arguments("option")
        .into_iter()
        .filter(|plan| !data.text.labels.contains_key(&format!("plan_{plan}")))
        .collect();
    assert!(missing.is_empty(), "plans without a label: {missing:?}");
}

#[test]
fn each_ending_has_a_buildable_capstone_path() {
    let data = game_data();
    for ending in &data.campaign.endings {
        let tech = data
            .tech(&ending.capstone_tech)
            .expect("capstone tech exists");
        assert_eq!(tech.capstone.as_deref(), Some(ending.id.as_str()));
        let building = data
            .building(&ending.capstone_building)
            .expect("capstone building exists");
        assert_eq!(building.tech.as_deref(), Some(tech.id.as_str()));
        assert!(building.is_player_buildable());
    }
}
