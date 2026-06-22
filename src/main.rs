mod caverna;
mod mcts;

use crate::{
    caverna::State,
    mcts::{random_move, search, GameState},
};

fn main() {
    let mut rng = rand::rng();
    let mut state = State::new(2);
    let names = ["Samuel", "Maria"];
    //state.deal(&mut rng);
    while state.winner().is_none() {
        println!("round {}", names[state.current_player()]);
        if state.current_player() == 0 {
            state = search(&state, &mut rng, 1000).unwrap();
        } else {
            state = random_move(&state, &mut rng);
        }
    }
    for (index, player) in state.players.iter().enumerate() {
        println!("player {}, {}", names[index], player.points());
    }
}
