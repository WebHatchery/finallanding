use super::*;
use crate::data::colonist::{JobPreference, Trait};
use crate::data::types::Position;

fn test_colonist(id: u32, name: &str) -> Colonist {
    Colonist::new(
        id,
        name.to_string(),
        Position::new(id as i32, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    )
}

#[test]
fn test_toggle_pair_directive_pairs_supportive_colonists() {
    let mut colonists = vec![test_colonist(1, "Alice"), test_colonist(2, "Bob")];
    colonists[0].relationships.insert(2, 22);
    colonists[1].relationships.insert(1, 18);

    let change = RelationshipDirectiveSystem::toggle_pair_directive(&mut colonists, 1, 2).unwrap();

    assert_eq!(change, DirectiveChange::Set(PairDirective::Pair));
    assert_eq!(colonists[0].preferred_partner_id, Some(2));
    assert_eq!(colonists[1].preferred_partner_id, Some(1));
    assert_eq!(
        RelationshipDirectiveSystem::directive_for_pair(&colonists, 1, 2),
        Some(PairDirective::Pair)
    );
}

#[test]
fn test_toggle_pair_directive_separates_tense_colonists() {
    let mut colonists = vec![test_colonist(1, "Alice"), test_colonist(2, "Bob")];
    colonists[0].relationships.insert(2, -24);
    colonists[1].relationships.insert(1, -18);

    let change = RelationshipDirectiveSystem::toggle_pair_directive(&mut colonists, 1, 2).unwrap();

    assert_eq!(change, DirectiveChange::Set(PairDirective::Separate));
    assert_eq!(colonists[0].avoided_partner_id, Some(2));
    assert_eq!(colonists[1].avoided_partner_id, Some(1));
}

#[test]
fn test_toggle_pair_directive_clears_existing_directive() {
    let mut colonists = vec![test_colonist(1, "Alice"), test_colonist(2, "Bob")];
    colonists[0].relationships.insert(2, 12);
    colonists[1].relationships.insert(1, 16);

    RelationshipDirectiveSystem::toggle_pair_directive(&mut colonists, 1, 2).unwrap();
    let change = RelationshipDirectiveSystem::toggle_pair_directive(&mut colonists, 1, 2).unwrap();

    assert_eq!(change, DirectiveChange::Cleared(PairDirective::Pair));
    assert_eq!(colonists[0].preferred_partner_id, None);
    assert_eq!(colonists[1].preferred_partner_id, None);
}
