use super::*;

fn create_test_grid() -> Grid {
    let mut grid = Grid::new(20, 20);
    // Make all cells Floor (walkable)
    for y in 0..20 {
        for x in 0..20 {
            grid.set_cell_type(x, y, CellType::Floor);
        }
    }
    grid
}

#[test]
fn test_place_building() {
    let mut grid = create_test_grid();
    let mut system = BuildingSystem::new();

    let result = system.try_place_building(&mut grid, BuildingType::Habitat, Position::new(5, 5));

    assert!(matches!(result, PlacementResult::Success(1)));
    assert_eq!(system.building_count(), 1);
}

#[test]
fn test_placement_overlap() {
    let mut grid = create_test_grid();
    let mut system = BuildingSystem::new();

    // Place first building
    system.try_place_building(&mut grid, BuildingType::Habitat, Position::new(5, 5));

    // Try to place overlapping building
    let result = system.try_place_building(&mut grid, BuildingType::Storage, Position::new(6, 6));

    assert_eq!(result, PlacementResult::AreaOccupied);
    assert_eq!(system.building_count(), 1);
}

#[test]
fn test_placement_out_of_bounds() {
    let mut grid = create_test_grid();
    let mut system = BuildingSystem::new();

    let result = system.try_place_building(
        &mut grid,
        BuildingType::Habitat,
        Position::new(19, 19), // 2x2 would be out of bounds
    );

    assert_eq!(result, PlacementResult::OutOfBounds);
}

#[test]
fn test_undo_placement() {
    let mut grid = create_test_grid();
    let mut system = BuildingSystem::new();

    system.try_place_building(&mut grid, BuildingType::Habitat, Position::new(5, 5));
    assert_eq!(system.building_count(), 1);
    assert_eq!(
        system.last_placed_building().map(|b| b.building_type),
        Some(BuildingType::Habitat)
    );

    let undone_id = system.undo_last_placement(&mut grid);
    assert_eq!(undone_id, Some(1));
    assert_eq!(system.building_count(), 0);

    // Area should be free again
    assert!(system.can_place_building(&grid, BuildingType::Habitat, Position::new(5, 5)));
}

#[test]
fn test_can_place_preview() {
    let mut grid = create_test_grid();
    let mut system = BuildingSystem::new();

    assert!(system.can_place_building(&grid, BuildingType::MessHall, Position::new(5, 5)));

    system.try_place_building(&mut grid, BuildingType::MessHall, Position::new(5, 5));

    assert!(!system.can_place_building(&grid, BuildingType::Habitat, Position::new(6, 5)));
}
