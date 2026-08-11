use super::*;

#[test]
fn test_simulation_rng_is_seeded_and_bounded() {
    let mut first = SimulationRng::with_seed(42);
    let mut second = SimulationRng::with_seed(42);

    assert_eq!(first.range_i32(-1, 2), second.range_i32(-1, 2));
    let value = first.range_f32(0.0, 100.0);
    assert!((0.0..100.0).contains(&value));
}
