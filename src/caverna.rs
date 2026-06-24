use crate::mcts::GameState;

#[derive(Clone, Copy, PartialEq)]
#[repr(usize)]
pub enum ActionSpace {
    Logging = 0,
    WoodGathering = 1,
    Supplies = 2,
    StartingPlayer = 3,
    Clearing = 4,
    Sustenance = 5,
    SlashAndBurn = 6,
    DriftMining = 7,
    Excavation = 8,
    SheepFarming = 9,
    DonkeyFarming = 10,
    OreMineConstruction = 11,
    RubyMineConstruction = 12,
}
impl ActionSpace {
    const COUNT: usize = 13;
    const ALL: [ActionSpace; Self::COUNT] = [
        ActionSpace::Logging,
        ActionSpace::WoodGathering,
        ActionSpace::Supplies,
        ActionSpace::StartingPlayer,
        ActionSpace::Clearing,
        ActionSpace::Sustenance,
        ActionSpace::SlashAndBurn,
        ActionSpace::DriftMining,
        ActionSpace::Excavation,
        ActionSpace::SheepFarming,
        ActionSpace::DonkeyFarming,
        ActionSpace::OreMineConstruction,
        ActionSpace::RubyMineConstruction,
    ];

    fn gain_resources(self, rounds: u32, resources: &mut Resources) {
        let r = rounds as usize;
        match self {
            ActionSpace::Logging        => resources.wood += 3 + r,
            ActionSpace::WoodGathering  => resources.wood += 1 + r,
            ActionSpace::Supplies       => {
                resources.wood += 1; resources.stone += 1; resources.coal += 1;
                resources.food += 1; resources.points += 2;
            }
            ActionSpace::StartingPlayer => { resources.coal += 2; resources.food += 1 + r; }
            ActionSpace::Clearing       => resources.wood += 1 + r,
            ActionSpace::Sustenance     => { resources.wheat += 1; resources.food += 1 + r; }
            ActionSpace::SlashAndBurn   => {}
            ActionSpace::DriftMining    => resources.stone += 1 + r,
            ActionSpace::Excavation     => resources.stone += 1 + r,
            ActionSpace::SheepFarming        => {}
            ActionSpace::DonkeyFarming       => {}
            ActionSpace::OreMineConstruction | ActionSpace::RubyMineConstruction => {}
        }
    }
    fn gain_animals(self, accumulated: u32, animals: &mut Animals) {
        let r = accumulated as usize;
        match self {
            ActionSpace::SheepFarming  => animals[AnimalType::Sheep  as usize] += 1 + r,
            ActionSpace::DonkeyFarming => animals[AnimalType::Donkey as usize] += 1 + r,
            _ => {}
        }
    }
    fn place_tile(self) -> Vec<TileToPlace> {
        match self {
            ActionSpace::Clearing | ActionSpace::Sustenance | ActionSpace::SlashAndBurn =>
                vec![TileToPlace::Twin((Tile::Meadow, Tile::Field((0, 0))))],
            ActionSpace::DriftMining =>
                vec![TileToPlace::Twin((Tile::Tunnel, Tile::Cave))],
            ActionSpace::Excavation =>
                vec![TileToPlace::Twin((Tile::Tunnel, Tile::Cave)), TileToPlace::Twin((Tile::Cave, Tile::Cave))],
            ActionSpace::OreMineConstruction =>
                vec![TileToPlace::Twin((Tile::OreTunnel, Tile::OreMine))],
            ActionSpace::RubyMineConstruction =>
                vec![TileToPlace::Single(Tile::RubyMine)],
            ActionSpace::SheepFarming | ActionSpace::DonkeyFarming => vec![],
            _ => vec![],
        }
    }
}

const BOARD_WIDTH: usize = 6;
const BOARD_HEIGHT: usize = 4;

fn adjacents(x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> {
    [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)]
        .into_iter()
        .filter(|&(nx, ny)| nx < BOARD_WIDTH && ny < BOARD_HEIGHT)
}

#[derive(Clone, Copy, PartialEq)]
enum Tile {
    // Outdoor
    Forest,
    ForestStable,                     // stable on uncleared forest — 1 boar
    Meadow,                           // unfenced
    MeadowStable,                     // unfenced + stable — 1 animal of any type
    Pasture,                          // fenced — 2 animals/cell
    PastureStable,                    // fenced + stable — doubles capacity
    Field((u8, u8)),                  // (wheat, vegetables) — only one non-zero at a time
    // Indoor
    Mountain,
    Tunnel,
    OreTunnel,
    OreMine,
    RubyMine,
    Cave,
    Dwelling,
}

impl Tile {
    fn base(self) -> Tile {
        match self {
            Tile::Meadow | Tile::MeadowStable
            | Tile::Pasture | Tile::PastureStable
            | Tile::Field(_) => Tile::Forest,
            Tile::Tunnel | Tile::OreTunnel | Tile::OreMine | Tile::RubyMine | Tile::Cave | Tile::Dwelling => Tile::Mountain,
            Tile::Forest | Tile::ForestStable | Tile::Mountain => self,
        }
    }

    fn is_undeveloped(self) -> bool {
        self == self.base()
    }

    fn is_fenceable(self) -> bool {
        matches!(self, Tile::Meadow | Tile::MeadowStable)
    }

    fn maybe_add_stable(self) -> Option<Tile> {
        match self {
            Tile::Forest  => Some(Tile::ForestStable),
            Tile::Meadow  => Some(Tile::MeadowStable),
            Tile::Pasture => Some(Tile::PastureStable),
            _ => None,
        }
    }
}

enum TileToPlace {
    Single(Tile),
    Twin((Tile, Tile)),
}

#[derive(Clone)]
pub struct Dwarf {
    pub weapon: u8,
    pub placed_on: Option<ActionSpace>,
}

#[derive(Clone, Copy)]
pub struct Resources {
    points: usize,
    begging: usize,

    wood: usize,
    stone: usize,
    coal: usize,
    rubies: usize,
    food: usize,
    
    wheat: usize,
    vegetables: usize,
}
impl Resources {
    fn zero() -> Resources {
        Resources { points: 0, begging: 0, wood: 0, stone: 0, coal: 0, rubies: 0, food: 0, wheat: 0, vegetables: 0 }
    }

}
impl std::ops::AddAssign for Resources {
    fn add_assign(&mut self, rhs: Resources) {
        self.points += rhs.points;
        self.begging += rhs.begging;
        self.wood += rhs.wood;
        self.stone += rhs.stone;
        self.coal += rhs.coal;
        self.rubies += rhs.rubies;
        self.food += rhs.food;
        self.wheat += rhs.wheat;
        self.vegetables += rhs.vegetables;
    }
}

#[derive(Clone, Copy, PartialEq)]
#[repr(usize)]
enum AnimalType { Cow = 0, Boar = 1, Donkey = 2, Sheep = 3 }

type Animals = [usize; 4];

#[derive(Clone)]
struct Pasture {
    cells: Vec<(usize, usize)>,
    animals: Option<(AnimalType, u8)>,
}

#[derive(Clone)]
pub struct Player {
    pub dwarfs: Vec<Dwarf>,
    tiles: [[Tile; BOARD_WIDTH]; BOARD_HEIGHT],
    resources: Resources,
    dogs: usize,
    animals: Animals, // indexed by AnimalType
    pastures: Vec<Pasture>,
}
impl Player {
    fn new(food: usize) -> Self {
        let mut tiles = [[Tile::Mountain; BOARD_WIDTH]; BOARD_HEIGHT];
        for row in tiles.iter_mut() {
            for x in 0..3 {
                row[x] = Tile::Forest;
            }
        }
        tiles[3][3] = Tile::Dwelling;
        tiles[2][3] = Tile::Cave;
        let mut player = Player {
            dwarfs: vec![Dwarf { weapon: 0, placed_on: None }, Dwarf { weapon: 0, placed_on: None }],
            tiles,
            resources: Resources::zero(),
            dogs: 0,
            animals: [0; 4],
            pastures: vec![],
        };
        player.resources.food = food;
        player
    }

    fn adjacent_to_developed(&self, x: usize, y: usize, base: Tile) -> bool {
        adjacents(x, y).any(|(nx, ny)| {
            let t = self.tiles[ny][nx];
            !t.is_undeveloped() && t.base() == base
        })
    }

    fn tile_placements(&self, tile: TileToPlace) -> Vec<[[Tile; BOARD_WIDTH]; BOARD_HEIGHT]> {
        let mut result = vec![];
        match tile {
            TileToPlace::Single(t) => {
                let base = t.base();
                for y in 0..BOARD_HEIGHT {
                    for x in 0..BOARD_WIDTH {
                        if self.tiles[y][x] != base { continue; }
                        if self.adjacent_to_developed(x, y, base) {
                            let mut board = self.tiles;
                            board[y][x] = t;
                            result.push(board);
                        }
                    }
                }
            }
            TileToPlace::Twin((t1, t2)) => {
                let base = t1.base();
                let outdoor_first = base == Tile::Forest
                    && self.tiles.iter().flatten().all(|t| t.is_undeveloped());
                for y in 0..BOARD_HEIGHT {
                    for x in 0..BOARD_WIDTH {
                        for (x2, y2) in [(x + 1, y), (x, y + 1)] {
                            if x2 >= BOARD_WIDTH || y2 >= BOARD_HEIGHT { continue; }
                            if self.tiles[y][x] != base || self.tiles[y2][x2] != base { continue; }
                            let valid = if outdoor_first {
                                (x == 2 && y == 3) || (x2 == 2 && y2 == 3)
                            } else {
                                self.adjacent_to_developed(x, y, base) || self.adjacent_to_developed(x2, y2, base)
                            };
                            if valid {
                                let mut board = self.tiles;
                                board[y][x] = t1; board[y2][x2] = t2;
                                result.push(board);
                                if t1 != t2 {
                                    let mut board = self.tiles;
                                    board[y][x] = t2; board[y2][x2] = t1;
                                    result.push(board);
                                }
                            }
                        }
                    }
                }
            }
        }
        result
    }
    pub fn points(&self) -> i32 {
        self.dwarfs.len() as i32 +
        self.resources.points as i32 +
        self.resources.rubies as i32 +
        self.resources.wheat as i32 / 2 +
        self.resources.vegetables as i32 +
        self.dogs as i32 +
        self.animals.iter().sum::<usize>() as i32 -
        self.resources.begging as i32 * 3 -
        self.tiles.iter().flatten().filter(|&&t| t.is_undeveloped()).count() as i32 -
        self.animals.iter().filter(|&&n| n == 0).count() as i32
    }
    fn trim_animals(&self, animals: Animals) -> Animals {
        // Boars can live in forest stables (1 per stable)
        let boar_fixed = self.tiles.iter().flatten()
            .filter(|&&t| t == Tile::ForestStable)
            .count();
        // Sheep can live on unfenced meadows, guarded by dogs (dogs+1 total if any meadow exists)
        let sheep_meadow = if self.tiles.iter().flatten().any(|&t| t == Tile::Meadow) {
            self.dogs + 1
        } else {
            0
        };
        // Flexible slots: each MeadowStable holds 1 farm animal; dwelling holds 2
        let flex = self.tiles.iter().flatten()
            .filter(|&&t| t == Tile::MeadowStable)
            .count() + 2;

        // Capacity per pasture: cells * 2, doubled for each stable in the pasture
        let pasture_caps: Vec<usize> = self.pastures.iter().map(|p| {
            let stables = p.cells.iter()
                .filter(|&&(x, y)| self.tiles[y][x] == Tile::PastureStable)
                .count();
            p.cells.len() * 2 * (1 << stables)
        }).collect();

        let fixed_boar  = animals[AnimalType::Boar  as usize].min(boar_fixed);
        let fixed_sheep = animals[AnimalType::Sheep as usize].min(sheep_meadow);
        // Remaining to assign to pastures/flex, in priority order: cattle, boar, donkey, sheep
        let need = [
            animals[AnimalType::Cow    as usize],
            animals[AnimalType::Boar   as usize] - fixed_boar,
            animals[AnimalType::Donkey as usize],
            animals[AnimalType::Sheep  as usize] - fixed_sheep,
        ];

        // Try all type assignments per pasture (4^n, typically ≤256)
        let n = pasture_caps.len();
        let mut best = [0usize; 4];
        let mut best_score = 0usize;
        for combo in 0..4_usize.pow(n as u32) {
            let mut remaining = need;
            let mut kept = [0usize; 4];
            let mut code = combo;
            for &cap in &pasture_caps {
                let t = code % 4;
                code /= 4;
                let took = remaining[t].min(cap);
                kept[t] += took;
                remaining[t] -= took;
            }
            // Fill flex slots greedily in priority order
            let mut flex_left = flex;
            for t in 0..4 {
                let took = remaining[t].min(flex_left);
                kept[t] += took;
                flex_left -= took;
            }
            let score: usize = kept.iter().sum();
            if score > best_score {
                best_score = score;
                best = kept;
            }
        }

        [best[AnimalType::Cow as usize], fixed_boar + best[AnimalType::Boar as usize], best[AnimalType::Donkey as usize], fixed_sheep + best[AnimalType::Sheep as usize]]
    }

    fn stable_count(&self) -> usize {
        self.tiles.iter().flatten()
            .filter(|&&t| matches!(t, Tile::ForestStable | Tile::MeadowStable | Tile::PastureStable))
            .count()
    }

    fn feed(&mut self) {
        let needed = self.dwarfs.len();
        if self.resources.food >= needed {
            self.resources.food -= needed;
        } else {
            self.resources.begging += needed - self.resources.food;
            self.resources.food = 0;
        }
    }
    fn harvest(&mut self) {
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                match self.tiles[y][x] {
                    Tile::Field((w, _)) if w > 0 => {
                        self.resources.wheat += 1;
                        self.tiles[y][x] = Tile::Field((w - 1, 0));
                    }
                    Tile::Field((_, v)) if v > 0 => {
                        self.resources.vegetables += 1;
                        self.tiles[y][x] = Tile::Field((0, v - 1));
                    }
                    _ => {}
                }
            }
        }
    }

    fn breed(&mut self) {
        let bred = self.animals.map(|n| n + if n >= 2 { 1 } else { 0 });
        self.animals = self.trim_animals(bred);
    }

    fn verify_pastures(&self) {
        let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
        for (i, pasture) in self.pastures.iter().enumerate() {
            assert!(!pasture.cells.is_empty(), "pasture {i} has no cells");

            for &(x, y) in &pasture.cells {
                assert!(
                    matches!(self.tiles[y][x], Tile::Pasture | Tile::PastureStable),
                    "pasture {i} cell ({x},{y}) is not a pasture tile"
                );
                assert!(seen.insert((x, y)), "cell ({x},{y}) appears in multiple pastures");
            }

            // All cells must form a single connected component
            let cell_set: std::collections::HashSet<_> = pasture.cells.iter().copied().collect();
            let mut visited: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
            let mut stack = vec![pasture.cells[0]];
            while let Some((x, y)) = stack.pop() {
                if !visited.insert((x, y)) { continue; }
                for (nx, ny) in adjacents(x, y) {
                    if cell_set.contains(&(nx, ny)) {
                        stack.push((nx, ny));
                    }
                }
            }
            assert_eq!(visited.len(), pasture.cells.len(), "pasture {i} cells are not all connected");
        }
    }
}

#[derive(Clone)]
pub struct State {
    pub players: Vec<Player>,
    pub round: u32,
    pub starting_player: u8,
    pub accumulated: [u32; ActionSpace::COUNT],
}
impl State {
    pub fn new(count: u32) -> Self {
        let mut players = Vec::new();
        for _ in 0..count {
            players.push(Player::new(2));
        }
        State { players, round: 0, starting_player: 0, accumulated: [0u32; ActionSpace::COUNT] }
    }
    fn rounds(&self) -> u32 {
        match self.players.len() {
            2 => 11,
            _ => 12,
        }
    }
    fn done(&self) -> bool {
        self.round >= self.rounds()
    }
    fn return_dwarfs(&mut self) {
        for player in &mut self.players {
            for dwarf in &mut player.dwarfs {
                dwarf.placed_on = None;
            }
        }
    }
    fn replenish(&mut self) {
        for i in 0..ActionSpace::COUNT {
            let taken = self.players.iter()
                .any(|p| p.dwarfs.iter().any(|d| d.placed_on.map(|s| s as usize) == Some(i)));
            if taken {
                self.accumulated[i] = 0;
            } else {
                self.accumulated[i] += 1;
            }
        }
    }
    fn harvest(&mut self) {
        for player in &mut self.players {
            player.harvest();
            player.feed();
            player.breed();
        }
    }
    fn pasture_options(&self, player_idx: usize) -> Vec<Self> {
        let player = &self.players[player_idx];
        let mut results = vec![self.clone()];

        let fence = |t: Tile| match t {
            Tile::Meadow       => Tile::Pasture,
            Tile::MeadowStable => Tile::PastureStable,
            _ => unreachable!(),
        };

        let meadows: Vec<(usize, usize)> = (0..BOARD_HEIGHT)
            .flat_map(|y| (0..BOARD_WIDTH).map(move |x| (x, y)))
            .filter(|&(x, y)| player.tiles[y][x].is_fenceable())
            .collect();

        if player.resources.wood >= 2 {
            for &(x, y) in &meadows {
                let mut child = self.clone();
                child.players[player_idx].tiles[y][x] = fence(player.tiles[y][x]);
                child.players[player_idx].resources.wood -= 2;
                child.players[player_idx].pastures.push(Pasture { cells: vec![(x, y)], animals: None });
                results.push(child);
            }
        }

        if player.resources.wood >= 4 {
            for &(x, y) in &meadows {
                for (x2, y2) in [(x + 1, y), (x, y + 1)] {
                    if x2 >= BOARD_WIDTH || y2 >= BOARD_HEIGHT { continue; }
                    if !player.tiles[y2][x2].is_fenceable() { continue; }
                    let mut child = self.clone();
                    child.players[player_idx].tiles[y][x] = fence(player.tiles[y][x]);
                    child.players[player_idx].tiles[y2][x2] = fence(player.tiles[y2][x2]);
                    child.players[player_idx].resources.wood -= 4;
                    child.players[player_idx].pastures.push(Pasture {
                        cells: vec![(x, y), (x2, y2)],
                        animals: None,
                    });
                    results.push(child);
                }
            }
        }

        results
    }

    fn stable_options(&self, player_idx: usize) -> Vec<Self> {
        let player = &self.players[player_idx];
        let mut results = vec![self.clone()];

        if player.resources.stone < 1 || player.stable_count() >= 3 {
            return results;
        }

        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                if let Some(t) = player.tiles[y][x].maybe_add_stable() {
                    let mut child = self.clone();
                    child.players[player_idx].tiles[y][x] = t;
                    child.players[player_idx].resources.stone -= 1;
                    results.push(child);
                }
            }
        }

        results
    }

    fn sow_options(&self, player_idx: usize) -> Vec<Self> {
        let player = &self.players[player_idx];
        let empty: Vec<(usize, usize)> = (0..BOARD_HEIGHT)
            .flat_map(|y| (0..BOARD_WIDTH).map(move |x| (x, y)))
            .filter(|&(x, y)| player.tiles[y][x] == Tile::Field((0, 0)))
            .collect();
        let max_wheat = player.resources.wheat.min(2);
        let max_veg = player.resources.vegetables.min(2);
        let mut results = vec![];
        for wheat in subsets_up_to_2(&empty) {
            if wheat.len() > max_wheat { continue; }
            let remaining: Vec<_> = empty.iter().copied()
                .filter(|s| !wheat.contains(s))
                .collect();
            for veg in subsets_up_to_2(&remaining) {
                if veg.len() > max_veg { continue; }
                let mut child = self.clone();
                child.players[player_idx].resources.wheat -= wheat.len();
                child.players[player_idx].resources.vegetables -= veg.len();
                for &(x, y) in &wheat {
                    child.players[player_idx].tiles[y][x] = Tile::Field((3, 0));
                }
                for &(x, y) in &veg {
                    child.players[player_idx].tiles[y][x] = Tile::Field((0, 2));
                }
                results.push(child);
            }
        }
        results
    }
}

fn subsets_up_to_2(items: &[(usize, usize)]) -> Vec<Vec<(usize, usize)>> {
    let mut result = vec![vec![]];
    for i in 0..items.len() {
        result.push(vec![items[i]]);
        for j in (i + 1)..items.len() {
            result.push(vec![items[i], items[j]]);
        }
    }
    result
}

impl GameState for State {
    fn current_player(&self) -> usize {
        let n = self.players.len();
        let placed: Vec<usize> = self.players.iter()
            .map(|p| p.dwarfs.iter().filter(|d| d.placed_on.is_some()).count())
            .collect();
        let total_placed: usize = placed.iter().sum();
        let total_dwarves: Vec<usize> = self.players.iter().map(|p| p.dwarfs.len()).collect();

        // Replay the clockwise turn sequence to find who goes next.
        // Each step advances past a player who still has dwarves to place.
        let mut turns = 0;
        let mut seat = self.starting_player as usize;
        let max_iter = total_dwarves.iter().sum::<usize>() * n + 1;
        for _ in 0..max_iter {
            if placed[seat] < total_dwarves[seat] {
                if turns == total_placed {
                    return seat;
                }
                turns += 1;
            }
            seat = (seat + 1) % n;
        }
        self.starting_player as usize
    }

    fn num_players(&self) -> usize {
        self.players.len()
    }

    fn children<R: rand::prelude::Rng>(&self, _rng: &mut R) -> Vec<Self> {
        if self.done() {
            return vec![];
        }

        let current = self.current_player();
        let mut children = vec![];
        for &space in &ActionSpace::ALL {
            let occupied = self.players.iter()
                .any(|p| p.dwarfs.iter().any(|d| d.placed_on == Some(space)));
            if occupied {
                continue;
            }
            let mut child = self.clone();
            child.players[current].dwarfs.iter_mut()
                .find(|d| d.placed_on.is_none())
                .expect("current player has no unplaced dwarf")
                .placed_on = Some(space);
            space.gain_resources(child.accumulated[space as usize], &mut child.players[current].resources);
            space.gain_animals(child.accumulated[space as usize], &mut child.players[current].animals);

            let tile_choices = space.place_tile();
            let mut candidates = if tile_choices.is_empty() {
                vec![child]
            } else {
                tile_choices.into_iter().flat_map(|tile| {
                    let boards = child.players[current].tile_placements(tile);
                    if boards.is_empty() {
                        vec![child.clone()]
                    } else {
                        boards.into_iter().map(|board| {
                            let mut c = child.clone();
                            c.players[current].tiles = board;
                            c
                        }).collect()
                    }
                }).collect()
            };

            if space == ActionSpace::SlashAndBurn {
                candidates = candidates.into_iter()
                    .flat_map(|c| c.sow_options(current))
                    .collect();
            }

            if matches!(space, ActionSpace::SheepFarming | ActionSpace::DonkeyFarming) {
                candidates = candidates.into_iter()
                    .flat_map(|c| c.pasture_options(current))
                    .flat_map(|c| c.stable_options(current))
                    .collect();
            }

            let all_placed = candidates[0].players.iter()
                .all(|p| p.dwarfs.iter().all(|d| d.placed_on.is_some()));
            if all_placed {
                for c in &mut candidates {
                    c.replenish();
                    c.return_dwarfs();
                    c.harvest();
                    c.round += 1;
                }
            }

            children.extend(candidates);
        }
        children
    }

    fn winner(&self) -> Option<usize> {
        if !self.done() {
            return None;
        }
        self.players.iter()
            .enumerate()
            .max_by_key(|(_, p)| p.points())
            .map(|(i, _)| i)
    }
}