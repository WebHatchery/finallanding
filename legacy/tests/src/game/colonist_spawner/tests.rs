use super::*;

#[test]
fn test_initial_colonists_start_with_social_context() {
    let mut state = GameState::new();

    spawn_initial_colonists(&mut state);

    let alice = state
        .colonists
        .iter()
        .find(|colonist| colonist.id == 0)
        .unwrap();
    let fiona = state
        .colonists
        .iter()
        .find(|colonist| colonist.id == 5)
        .unwrap();
    let charlie = state
        .colonists
        .iter()
        .find(|colonist| colonist.id == 2)
        .unwrap();
    let evan = state
        .colonists
        .iter()
        .find(|colonist| colonist.id == 4)
        .unwrap();

    assert_eq!(alice.relationships.get(&5), Some(&-24));
    assert_eq!(fiona.relationships.get(&0), Some(&-24));
    assert_eq!(charlie.relationships.get(&4), Some(&28));
    assert_eq!(evan.relationships.get(&2), Some(&28));
}
