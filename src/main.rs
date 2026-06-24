mod caverna;
mod mcts;
#[cfg(test)]
mod test_caverna;

use crate::{
    caverna::State,
    mcts::{random_move, search, GameState},
};
use rand::SeedableRng;
use rand::rngs::StdRng;

fn main() {
    let mut seed: Option<u64> = None;
    let mut mcts_iter: usize = 1000;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => {
                seed = Some(args.next().expect("--seed requires a value").parse().expect("--seed must be a number"));
            }
            "--mcts-iter" => {
                mcts_iter = args.next().expect("--mcts-iter requires a value").parse().expect("--mcts-iter must be a number");
            }
            other => eprintln!("unknown argument: {}", other),
        }
    }

    let mut rng: StdRng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_rng(&mut rand::rng()),
    };

    let mut state = State::new(2);
    let names = ["Samuel", "Maria"];
    let mut round = u32::MAX;
    while state.winner().is_none() {
        if state.round != round {
            round = state.round;
            println!("=== Round {} ===", round + 1);
        }
        if state.is_placement() {
            println!("{}", names[state.current_player()]);
        }
        if state.current_player() == 0 {
            state = search(&state, &mut rng, mcts_iter).unwrap();
        } else {
            state = random_move(&state, &mut rng);
        }
    }
    for (index, player) in state.players.iter().enumerate() {
        println!("player {}, {}", names[index], player.points());
    }
}
