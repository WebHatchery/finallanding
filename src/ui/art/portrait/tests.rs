use super::*;
use crate::ui::art::profiles::SURVIVOR_ART_PROFILES;

#[test]
fn test_portrait_generation_uses_opaque_pixels() {
    let image = generate_portrait(SURVIVOR_ART_PROFILES[0], 0);

    assert_eq!(image.width, PORTRAIT_SIZE);
    assert_eq!(image.height, PORTRAIT_SIZE);
    assert!(image.get_pixel(64, 56).a > 0.9);
    assert_ne!(image.get_pixel(44, 94), image.get_pixel(80, 94));
}
