use super::*;

#[test]
fn test_strongest_relationship_value_uses_absolute_pressure() {
    let mut colonist = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    );
    colonist.relationships.insert(2, 26);
    colonist.relationships.insert(3, -31);

    assert_eq!(strongest_relationship_value(&colonist), Some(-31));
}

#[test]
fn test_sprite_pose_tracks_colonist_state() {
    assert_eq!(sprite_pose_for_state(ColonistState::Idle), SpritePose::Idle);
    assert_eq!(
        sprite_pose_for_state(ColonistState::Moving {
            target: Position::new(1, 1)
        }),
        SpritePose::Moving
    );
    assert_eq!(
        sprite_pose_for_state(ColonistState::Working),
        SpritePose::Working
    );
    assert_eq!(
        sprite_pose_for_state(ColonistState::Eating),
        SpritePose::Eating
    );
    assert_eq!(
        sprite_pose_for_state(ColonistState::Sleeping),
        SpritePose::Sleeping
    );
}

#[test]
fn test_social_body_language_overrides_idle_pose() {
    let colonist = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    );

    assert_eq!(
        sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Tense(-24)), 0),
        SpritePose::Tense
    );
    assert_eq!(
        sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Supported(28)), 0),
        SpritePose::Supported
    );
}

#[test]
fn test_social_body_language_cycles_alternate_pose_frames() {
    let colonist = Colonist::new(
        1,
        "Alice".to_string(),
        Position::new(0, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    );

    assert_eq!(
        sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Supported(28)), 45),
        SpritePose::SupportedReach
    );
    assert_eq!(
        sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Tense(-24)), 45),
        SpritePose::TenseGuarded
    );
    assert_eq!(
        sprite_pose_for_colonist_frame(&colonist, Some(SocialBodyLanguage::Tense(-24)), 90),
        SpritePose::Tense
    );
}
