mod caverna;
mod mcts;
#[cfg(test)]
mod test_caverna;

use crate::{
    caverna::{ActionSpace, SubAction, Player, Resources, State, Tile, BOARD_HEIGHT},
    mcts::{random_move, search, GameState},
};
use rand::SeedableRng;
use rand::rngs::StdRng;

impl std::fmt::Display for ActionSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = match self {
            ActionSpace::DriftMining          => "Drift Mining",
            ActionSpace::Logging              => "Logging",
            ActionSpace::WoodGathering        => "Wood Gathering",
            ActionSpace::Excavation           => "Excavation",
            ActionSpace::Supplies             => "Supplies",
            ActionSpace::Clearing             => "Clearing",
            ActionSpace::StartingPlayer       => "Starting Player",
            ActionSpace::OreMining            => "Ore Mining",
            ActionSpace::Sustenance           => "Sustenance",
            ActionSpace::RubyMining           => "Ruby Mining",
            ActionSpace::Housework            => "Housework",
            ActionSpace::SlashAndBurn         => "Slash and Burn",
            ActionSpace::SheepFarming         => "Sheep Farming",
            ActionSpace::OreMineConstruction  => "Ore Mine Construction",
            ActionSpace::Blacksmithing        => "Blacksmithing",
            ActionSpace::WishForChildren      => "Wish for Children",
            ActionSpace::RubyMineConstruction => "Ruby Mine Construction",
            ActionSpace::DonkeyFarming        => "Donkey Farming",
            ActionSpace::FamilyLife           => "Family Life",
            ActionSpace::OreDelivery          => "Ore Delivery",
            ActionSpace::RubyDelivery         => "Ruby Delivery",
            ActionSpace::Adventure            => "Adventure",
        };
        write!(f, "{}", name)
    }
}

fn resource_delta_parts(old: &Resources, new: &Resources) -> Vec<String> {
    let mut parts = vec![];
    macro_rules! diff {
        ($field:ident, $name:literal) => {
            match new.$field as i64 - old.$field as i64 {
                d if d > 0 => parts.push(format!("+{} {}", d, $name)),
                d if d < 0 => parts.push(format!("{} {}", d, $name)),
                _ => {}
            }
        };
    }
    diff!(wood, "wood");
    diff!(stone, "stone");
    diff!(coal, "coal");
    diff!(rubies, "rubies");
    diff!(food, "food");
    diff!(wheat, "wheat");
    diff!(vegetables, "veg");
    diff!(gold, "gold");
    if new.begging > old.begging {
        parts.push(format!("+{} begging", new.begging - old.begging));
    }
    parts
}

fn animal_delta_parts(old: &[usize; 4], new: &[usize; 4], old_dogs: usize, new_dogs: usize) -> Vec<String> {
    let names = ["cow", "boar", "donkey", "sheep"];
    let mut parts = vec![];
    for i in 0..4 {
        match new[i] as i64 - old[i] as i64 {
            d if d > 0 => parts.push(format!("+{} {}", d, names[i])),
            d if d < 0 => parts.push(format!("{} {}", d, names[i])),
            _ => {}
        }
    }
    match new_dogs as i64 - old_dogs as i64 {
        d if d > 0 => parts.push(format!("+{} dog", d)),
        d if d < 0 => parts.push(format!("{} dog", d)),
        _ => {}
    }
    parts
}

fn describe_move(state: &State, next: &State) -> String {
    let current = state.current_player;
    let p_old = &state.players[current];
    let p_new = &next.players[current];

    let mut parts = resource_delta_parts(&p_old.resources, &p_new.resources);
    parts.extend(animal_delta_parts(&p_old.animals, &p_new.animals, p_old.dogs, p_new.dogs));

    for (i, (nd, od)) in p_new.dwarfs.iter().zip(p_old.dwarfs.iter()).enumerate() {
        if nd.weapon > od.weapon {
            parts.push(format!("dwarf {} forged weapon {}", i + 1, nd.weapon));
        }
    }
    if p_new.children > p_old.children {
        parts.push(format!("+{} child", p_new.children - p_old.children));
    }
    if p_new.dwarfs.len() > p_old.dwarfs.len() {
        parts.push(format!("+{} dwarf", p_new.dwarfs.len() - p_old.dwarfs.len()));
    }

    let gains = if parts.is_empty() { String::new() } else { format!(" ({})", parts.join(", ")) };

    match state.pending.last() {
        Some(SubAction::SelectActionSpace) => {
            let space = p_new.dwarfs.iter().zip(p_old.dwarfs.iter())
                .find_map(|(nd, od)| {
                    if nd.placed_on.is_some() && od.placed_on.is_none() { nd.placed_on } else { None }
                });
            match space {
                Some(s) => format!("{}{}", s, gains),
                None => unreachable!("advance round is handled in the main loop"),
            }
        }
        Some(SubAction::ExpeditionPick { .. }) => {
            if parts.is_empty() { "expedition pass".to_string() }
            else { format!("expedition pick{}", gains) }
        }
        Some(SubAction::PlaceTile { .. }) => format!("place tile{}", gains),
        Some(SubAction::Pasture)           => format!("pasture{}", gains),
        Some(SubAction::Stable)            => format!("stable{}", gains),
        Some(SubAction::Furnish)           => format!("furnish{}", gains),
        Some(SubAction::Sow)                    => format!("sow{}", gains),
        Some(SubAction::ChooseTile(_))          => format!("choose tile{}", gains),
        Some(SubAction::TradeFood)              => format!("trade food{}", gains),
        Some(SubAction::FinishRound)            => unreachable!("finish round is handled in the main loop"),
        None => "?".to_string(),
    }
}

fn tile_symbol(tile: Tile) -> &'static str {
    match tile {
        Tile::Forest        => "..",
        Tile::ForestStable  => ".s",
        Tile::Meadow        => "Me",
        Tile::MeadowStable  => "Ms",
        Tile::Pasture       => "Pa",
        Tile::PastureStable => "Ps",
        Tile::Field((0, 0)) => "Fd",
        Tile::Field((w, _)) if w > 0 => "Fw",
        Tile::Field(_)      => "Fv",
        Tile::Mountain  => "~~",
        Tile::Tunnel    => "Tu",
        Tile::DeepTunnel => "Dt",
        Tile::OreMine   => "Om",
        Tile::RubyMine  => "Rm",
        Tile::Cave      => "Cv",
        Tile::Dwelling  => "Dw",
    }
}

fn print_player_state(player: &Player) {
    for y in 0..BOARD_HEIGHT {
        let out: String = player.outdoor[y].iter().map(|&t| tile_symbol(t)).collect::<Vec<_>>().join(" ");
        let ind: String = player.indoor[y].iter().map(|&t| tile_symbol(t)).collect::<Vec<_>>().join(" ");
        println!("    {} | {}", out, ind);
    }
    let r = &player.resources;
    let mut parts: Vec<String> = [
        (r.wood, "wood"), (r.stone, "stone"), (r.coal, "coal"), (r.rubies, "ruby"),
        (r.food, "food"), (r.wheat, "wheat"), (r.vegetables, "veg"), (r.gold, "gold"),
    ].iter()
     .filter(|(n, _)| *n > 0)
     .map(|(n, name)| format!("{} {}", n, name))
     .collect();
    player.animals.iter()
        .zip(["cow", "boar", "donkey", "sheep"])
        .filter(|(&n, _)| n > 0)
        .for_each(|(n, name)| parts.push(format!("{} {}", n, name)));
    if player.dogs > 0 { parts.push(format!("{} dog", player.dogs)); }
    if !parts.is_empty() {
        println!("    {}", parts.join(", "));
    }
}

fn print_options(state: &State, chosen: &State, rng: &mut StdRng) {
    use std::collections::BTreeMap;
    let chosen_desc = describe_move(state, chosen);
    let children = state.children(rng);
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for child in &children {
        *counts.entry(describe_move(state, child)).or_insert(0) += 1;
    }
    println!("    {} children:", children.len());
    for (desc, count) in &counts {
        let marker = if *desc == chosen_desc { '>' } else { ' ' };
        println!("    {} {:>3}x {}", marker, count, desc);
    }
}

fn main() {
    let mut seed: Option<u64> = None;
    let mut mcts_iter: usize = 1000;
    let mut verbose = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" => {
                seed = Some(args.next().expect("--seed requires a value").parse().expect("--seed must be a number"));
            }
            "--mcts-iter" => {
                mcts_iter = args.next().expect("--mcts-iter requires a value").parse().expect("--mcts-iter must be a number");
            }
            "-v" | "--verbose" => verbose = true,
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
        let current = state.current_player();
        let next = if current == 0 {
            search(&state, &mut rng, mcts_iter).unwrap()
        } else {
            random_move(&state, &mut rng)
        };
        let is_advance_round = matches!(state.pending.last(), Some(SubAction::SelectActionSpace))
            && state.players[current].dwarfs.iter().all(|d| d.placed_on.is_some());
        let is_finish_round = matches!(state.pending.last(), Some(SubAction::FinishRound));
        if is_advance_round {
            let parts: Vec<String> = names.iter().zip(state.players.iter()).zip(next.players.iter())
                .map(|((name, old), new)| {
                    let wheat = new.resources.wheat as i64 - old.resources.wheat as i64;
                    let veg = new.resources.vegetables as i64 - old.resources.vegetables as i64;
                    let mut crops = vec![];
                    if wheat > 0 { crops.push(format!("+{} wheat", wheat)); }
                    if veg > 0 { crops.push(format!("+{} veg", veg)); }
                    if crops.is_empty() { format!("{} +0", name) } else { format!("{} {}", name, crops.join(", ")) }
                })
                .collect();
            println!("  harvest: {}", parts.join(", "));
        } else if is_finish_round {
            let parts: Vec<String> = names.iter().zip(state.players.iter()).zip(next.players.iter())
                .map(|((name, old), new)| {
                    let food = new.resources.food as i64 - old.resources.food as i64;
                    let begging = new.resources.begging as i64 - old.resources.begging as i64;
                    let mut p = vec![];
                    if food != 0 { p.push(format!("{} food", food)); }
                    if begging > 0 { p.push(format!("+{} begging", begging)); }
                    if p.is_empty() { format!("{} -0", name) } else { format!("{} {}", name, p.join(", ")) }
                })
                .collect();
            println!("  feed: {}", parts.join(", "));
        } else {
            println!("  {}: {}", names[current], describe_move(&state, &next));
            print_player_state(&next.players[current]);
            if verbose {
                print_options(&state, &next, &mut rng);
            }
        }
        state = next;
    }
    for (index, player) in state.players.iter().enumerate() {
        println!("player {}, {}", names[index], player.points());
    }
}
