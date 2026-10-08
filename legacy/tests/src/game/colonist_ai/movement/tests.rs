use super::*;

#[test]
fn test_movement_uses_grid_pathfinding_around_blocked_cells() {
    let mut grid = Grid::new(4, 3);
    grid.set_cell_type(1, 0, CellType::Wall);

    let mut colonist = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(0, 0),
        Trait::FastWalker,
        JobPreference::Builder,
    );
    let occupied = HashMap::new();
    let colonist_names = HashMap::new();
    let mut pending_logs = Vec::new();

    let next = get_next_move_position(
        &mut colonist,
        Position::new(2, 0),
        &occupied,
        &grid,
        &colonist_names,
        0,
        &mut pending_logs,
    );

    assert_ne!(next, Position::new(1, 0));
    assert_ne!(next, colonist.position);
    assert!(is_step_open(next, &occupied, &grid));
}
