use crate::{caverna::State, mcts::GameState};

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
