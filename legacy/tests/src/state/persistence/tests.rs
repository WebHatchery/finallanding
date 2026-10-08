use finallanding::data::building::BuildingType;
use finallanding::data::types::Position;
use finallanding::state::persistence::{SaveDto, SAVE_SCHEMA_VERSION};
use finallanding::state::runtime_state::GameState;

#[test]
fn save_dto_round_trip_preserves_runtime_colony_state() {
    let mut game = GameState::new();
    game.tick = 912;
    game.rng.next_u32();
    game.building_system
        .try_place_building(&mut game.data.grid, BuildingType::Habitat, Position::new(2, 3));

    let encoded = serde_json::to_string(&SaveDto::new(&game)).unwrap();
    let decoded: SaveDto = serde_json::from_str(&encoded).unwrap();
    let restored = decoded.validate().unwrap();

    assert_eq!(restored.tick, 912);
    assert_eq!(restored.building_system.building_count(), 1);
    assert_eq!(restored.rng.state(), game.rng.state());
}

#[test]
fn save_dto_rejects_unknown_schema_versions() {
    let mut dto = SaveDto::new(&GameState::new());
    dto.schema_version = SAVE_SCHEMA_VERSION + 1;

    let error = dto.validate().unwrap_err();

    assert!(error.contains("not supported"));
}
