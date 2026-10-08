use super::*;

#[test]
fn test_sprite_generation_preserves_transparent_edges() {
    let image = generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::Idle);

    assert_eq!(image.width, SPRITE_WIDTH);
    assert_eq!(image.height, SPRITE_HEIGHT);
    assert_eq!(image.get_pixel(0, 0).a, 0.0);
    assert!(image.get_pixel(16, 36).a >= 0.89);
}

#[test]
fn test_sprite_pose_generation_changes_body_language() {
    let idle = generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::Idle);
    let working = generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::Working);
    let sleeping = generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::Sleeping);
    let supported = generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::Supported);
    let supported_reach = generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::SupportedReach);
    let tense = generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::Tense);
    let tense_guarded = generate_sprite(SURVIVOR_ART_PROFILES[0], 0, SpritePose::TenseGuarded);

    assert_ne!(idle.get_pixel(29, 45), working.get_pixel(29, 45));
    assert!(sleeping.get_pixel(17, 43).a > 0.7);
    assert_eq!(sleeping.get_pixel(16, 19).a, 0.0);
    assert_ne!(idle.get_pixel(5, 35), supported.get_pixel(5, 35));
    assert_ne!(
        supported.get_pixel(23, 23),
        supported_reach.get_pixel(23, 23)
    );
    assert_ne!(idle.get_pixel(12, 36), tense.get_pixel(12, 36));
    assert_ne!(tense.get_pixel(12, 40), tense_guarded.get_pixel(12, 40));
}
