use crate::{caverna::State, mcts::GameState};

#[test]
fn test_initial_children() {
    let mut rng = rand::rng();
    let state = State::new(2);
    let children = state.children(&mut rng);

    // Round 0 opens 13 action spaces (indices 0..=12), each producing one candidate.
    assert_eq!(children.len(), 13);

    // Every child: still round 0, player 0 placed exactly one dwarf
    for child in &children {
        assert_eq!(child.round, 0);
        let placed = child.players[0].dwarfs.iter().filter(|d| d.placed_on.is_some()).count();
        assert_eq!(placed, 1, "player 0 should have exactly one placed dwarf");
    }
}

#[test]
fn test_game_runs() {
    let mut rng = rand::rng();
    let mut state = State::new(2);
    for _ in 0..200 {
        if state.winner().is_some() { break; }
        let mut children = state.children(&mut rng);
        assert!(!children.is_empty(), "no children in non-terminal state");
        state = children.remove(0);
    }
}
