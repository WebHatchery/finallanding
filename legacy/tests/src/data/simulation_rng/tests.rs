use super::*;

#[test]
fn legacy_sequence_survives_toolkit_migration() {
    let mut rng = SimulationRng::with_seed(42);
    assert_eq!(rng.range_i32(-10, 11), 9);
    let expected = -5.0 + 10.0 * (2_013_331_137_u32 as f32 / u32::MAX as f32);
    assert_eq!(rng.range_f32(-5.0, 5.0).to_bits(), expected.to_bits());
    let mut zero = SimulationRng::with_seed(0);
    assert_eq!(zero.next_u32(), 1_481_765_933);
}

#[test]
fn test_simulation_rng_is_seeded_and_bounded() {
    let mut first = SimulationRng::with_seed(42);
    let mut second = SimulationRng::with_seed(42);

    assert_eq!(first.range_i32(-1, 2), second.range_i32(-1, 2));
    let value = first.range_f32(0.0, 100.0);
    assert!((0.0..100.0).contains(&value));
}
