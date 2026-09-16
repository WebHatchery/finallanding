use super::*;

#[test]
fn test_coordinate_conversion() {
    let pos = Grid::world_to_grid(50.0, 70.0);
    assert_eq!(pos.x, 1);
    assert_eq!(pos.y, 2);

    let (wx, wy) = Grid::grid_to_world(1, 2);
    assert_eq!(wx, 32.0);
    assert_eq!(wy, 64.0);
}

#[test]
fn test_bounds_checking() {
    let grid = Grid::new(10, 10);
    assert!(grid.is_in_bounds(0, 0));
    assert!(grid.is_in_bounds(9, 9));
    assert!(!grid.is_in_bounds(-1, 0));
    assert!(!grid.is_in_bounds(10, 10));
}

#[test]
fn test_pathfinding_simple() {
    let mut grid = Grid::new(5, 5);
    // Make all cells walkable
    for y in 0..5 {
        for x in 0..5 {
            grid.set_cell_type(x, y, CellType::Floor);
        }
    }

    let path = grid.find_path(Position::new(0, 0), Position::new(4, 4));
    assert!(path.is_some());
    let path = path.unwrap();
    assert_eq!(path.first(), Some(&Position::new(0, 0)));
    assert_eq!(path.last(), Some(&Position::new(4, 4)));
}

#[test]
fn test_pathfinding_blocked() {
    let mut grid = Grid::new(3, 3);
    for y in 0..3 {
        for x in 0..3 {
            grid.set_cell_type(x, y, CellType::Floor);
        }
    }
    // Block the middle row
    grid.set_cell_type(0, 1, CellType::Wall);
    grid.set_cell_type(1, 1, CellType::Wall);
    grid.set_cell_type(2, 1, CellType::Wall);

    let path = grid.find_path(Position::new(1, 0), Position::new(1, 2));
    assert!(path.is_none());
}
