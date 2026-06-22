use crate::{caverna::State, mcts::GameState};

#[test]
fn test_no_consecutive_same_player() {
    let mut rng = rand::rng();
    let mut state = State::new(2);
    let mut turns = vec![];

    for _ in 0..10 {
        if state.winner().is_some() { break; }
        turns.push(state.current_player());
        let mut children = state.children(&mut rng);
        state = children.remove(0);
    }

    for i in 1..turns.len() {
        assert_ne!(turns[i - 1], turns[i],
            "player {} moved consecutively at turns {} and {}: {:?}",
            turns[i], i - 1, i, turns);
    }
}
