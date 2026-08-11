use super::*;

#[test]
fn test_crash_scorch_intensity_marks_wreck_core() {
    assert!(crash_scorch_intensity(8, 5) > 0.9);
    assert!(crash_scorch_intensity(0, 0) < 0.1);
}

#[test]
fn test_flicker_alpha_stays_visible_and_bounded() {
    for tick in [0, 9, 18, 27, 36, 90, 144] {
        let alpha = flicker_alpha(tick, 13);
        assert!((0.42..=0.9).contains(&alpha));
    }
}
