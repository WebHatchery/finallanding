use super::*;

#[test]
fn test_building_sizes() {
    assert_eq!(BuildingType::Habitat.size(), (2, 2));
    assert_eq!(BuildingType::MessHall.size(), (3, 2));
    assert_eq!(BuildingType::Workshop.size(), (2, 3));
    assert_eq!(BuildingType::Storage.size(), (2, 2));
    assert_eq!(BuildingType::ExplorationGate.size(), (2, 2));
}

#[test]
fn test_building_salvage_costs() {
    assert_eq!(BuildingType::Habitat.salvage_cost(), 8);
    assert_eq!(BuildingType::MessHall.salvage_cost(), 12);
    assert_eq!(BuildingType::Workshop.salvage_cost(), 10);
    assert_eq!(BuildingType::Storage.salvage_cost(), 6);
    assert_eq!(BuildingType::ExplorationGate.salvage_cost(), 14);
}

#[test]
fn test_building_planning_roles_cover_core_needs() {
    assert_eq!(BuildingType::Habitat.planning_role(), "Recovery");
    assert_eq!(BuildingType::MessHall.planning_role(), "Food");
    assert_eq!(BuildingType::Workshop.planning_role(), "Salvage");
    assert_eq!(BuildingType::Storage.planning_role(), "Storage");
    assert_eq!(BuildingType::ExplorationGate.planning_role(), "Exploration");
}

#[test]
fn test_building_occupies() {
    let building = Building::new(1, BuildingType::Habitat, Position::new(5, 5));

    // Should occupy 2x2 area starting at (5,5)
    assert!(building.occupies(Position::new(5, 5)));
    assert!(building.occupies(Position::new(6, 5)));
    assert!(building.occupies(Position::new(5, 6)));
    assert!(building.occupies(Position::new(6, 6)));

    // Should not occupy outside area
    assert!(!building.occupies(Position::new(4, 5)));
    assert!(!building.occupies(Position::new(7, 5)));
    assert!(!building.occupies(Position::new(5, 7)));
}

#[test]
fn test_occupied_cells() {
    let building = Building::new(1, BuildingType::Habitat, Position::new(0, 0));
    let cells = building.occupied_cells();

    assert_eq!(cells.len(), 4);
    assert!(cells.contains(&Position::new(0, 0)));
    assert!(cells.contains(&Position::new(1, 0)));
    assert!(cells.contains(&Position::new(0, 1)));
    assert!(cells.contains(&Position::new(1, 1)));
}
