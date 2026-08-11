use super::*;

#[test]
fn test_toolbar_mode_from_name_accepts_capture_modes() {
    assert_eq!(toolbar_mode_from_name("assign"), Some(ToolbarMode::Assign));
    assert_eq!(
        toolbar_mode_from_name(" Research "),
        Some(ToolbarMode::Research)
    );
    assert_eq!(toolbar_mode_from_name("missing"), None);
}

#[test]
fn test_building_type_from_name_accepts_capture_names() {
    assert_eq!(
        building_type_from_name("mess hall"),
        Some(BuildingType::MessHall)
    );
    assert_eq!(
        building_type_from_name("exploration-gate"),
        Some(BuildingType::ExplorationGate)
    );
    assert_eq!(building_type_from_name("missing"), None);
}
