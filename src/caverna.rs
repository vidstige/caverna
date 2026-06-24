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
    Blacksmithing = 13,
    OreMining = 14,
    RubyMining = 15,
    OreDelivery = 16,
    RubyDelivery = 17,
    Adventure = 18,
}
impl ActionSpace {
    const COUNT: usize = 19;
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
        ActionSpace::Blacksmithing,
        ActionSpace::OreMining,
        ActionSpace::RubyMining,
        ActionSpace::OreDelivery,
        ActionSpace::RubyDelivery,
        ActionSpace::Adventure,
    ];

    fn picks_per_adventure(self) -> usize {
        match self {
            ActionSpace::Logging => 1,
            ActionSpace::OreMineConstruction => 2,
            ActionSpace::Blacksmithing => 3,
            ActionSpace::Adventure => 1,
            _ => 0,
        }
    }

    fn adventure_count(self) -> usize {
        match self {
            ActionSpace::Logging | ActionSpace::OreMineConstruction | ActionSpace::Blacksmithing => 1,
            ActionSpace::Adventure => 2,
            _ => 0,
        }
    }

    fn gain_resources(self, rounds: u32, resources: &mut Resources) {
        let r = rounds as usize;
        match self {
            ActionSpace::Logging        => resources.wood += 3 + r,
            ActionSpace::WoodGathering  => resources.wood += 1 + r,
            ActionSpace::Supplies       => {
                resources.wood += 1; resources.stone += 1; resources.coal += 1;
                resources.food += 1; resources.gold += 2;
            }
            ActionSpace::StartingPlayer => { resources.coal += 2; resources.food += 1 + r; }
            ActionSpace::Clearing       => resources.wood += 1 + r,
            ActionSpace::Sustenance     => { resources.wheat += 1; resources.food += 1 + r; }
            ActionSpace::SlashAndBurn   => {}
            ActionSpace::DriftMining    => resources.stone += 1 + r,
            ActionSpace::Excavation     => resources.stone += 1 + r,
            ActionSpace::SheepFarming        => {}
            ActionSpace::DonkeyFarming       => {}
            ActionSpace::OreMining   => resources.coal   += 2 + r,
            ActionSpace::RubyMining  => resources.rubies += 1 + r,
            ActionSpace::OreDelivery => { resources.stone += 1 + r; resources.coal += 1 + r; }
            ActionSpace::RubyDelivery => resources.rubies += 2 + r,
            ActionSpace::OreMineConstruction | ActionSpace::RubyMineConstruction
            | ActionSpace::Blacksmithing | ActionSpace::Adventure => {}
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
    fn gain_placement_resources(self, replaced: TileGroup, resources: &mut Resources) {
        match self {
            ActionSpace::OreMineConstruction => resources.coal += 3,
            ActionSpace::RubyMineConstruction => {
                if matches!(replaced, TileGroup::Single(Tile::OreTunnel)) {
                    resources.rubies += 1;
                }
            }
            _ => {}
        }
    }
    fn place_tile(self) -> Vec<TileGroup> {
        match self {
            ActionSpace::Clearing | ActionSpace::Sustenance | ActionSpace::SlashAndBurn =>
                vec![TileGroup::Twin((Tile::Meadow, Tile::Field((0, 0))))],
            ActionSpace::DriftMining =>
                vec![TileGroup::Twin((Tile::Tunnel, Tile::Cave))],
            ActionSpace::Excavation =>
                vec![TileGroup::Twin((Tile::Tunnel, Tile::Cave)), TileGroup::Twin((Tile::Cave, Tile::Cave))],
            ActionSpace::OreMineConstruction =>
                vec![TileGroup::Twin((Tile::OreTunnel, Tile::OreMine))],
            ActionSpace::RubyMineConstruction =>
                vec![TileGroup::Single(Tile::RubyMine)],
            ActionSpace::SheepFarming | ActionSpace::DonkeyFarming
            | ActionSpace::Blacksmithing | ActionSpace::Adventure
            | ActionSpace::OreMining | ActionSpace::RubyMining
            | ActionSpace::OreDelivery | ActionSpace::RubyDelivery => vec![],
            _ => vec![],
        }
    }
}

const BOARD_WIDTH: usize = 6;
const BOARD_HEIGHT: usize = 4;

fn changed_cells(old: &[[Tile; BOARD_WIDTH]; BOARD_HEIGHT], new: &[[Tile; BOARD_WIDTH]; BOARD_HEIGHT]) -> Vec<(usize, usize)> {
    (0..BOARD_HEIGHT).flat_map(|y| (0..BOARD_WIDTH).map(move |x| (x, y)))
        .filter(|&(x, y)| old[y][x] != new[y][x])
        .collect()
}

fn board_delta(old: &[[Tile; BOARD_WIDTH]; BOARD_HEIGHT], new: &[[Tile; BOARD_WIDTH]; BOARD_HEIGHT]) -> TileGroup {
    let replaced: Vec<Tile> = changed_cells(old, new).into_iter()
        .map(|(x, y)| old[y][x])
        .collect();
    match replaced.as_slice() {
        &[t]      => TileGroup::Single(t),
        &[t1, t2] => TileGroup::Twin((t1, t2)),
        _         => panic!("unexpected board delta: {} tiles changed", replaced.len()),
    }
}

fn apply_location_bonus((x, y): (usize, usize), resources: &mut Resources, animals: &mut Animals) {
    match (x, y) {
        (0, 2) | (2, 0) => animals[AnimalType::Boar as usize] += 1,
        (1, 3)          => resources.food += 1,
        (5, 0)          => resources.food += 2,
        (4, 3)          => resources.food += 1,
        _               => {}
    }
}

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

    fn points(self) -> i32 {
        match self {
            Tile::Pasture | Tile::PastureStable => 2,
            Tile::OreMine  => 3,
            Tile::RubyMine => 4,
            Tile::Dwelling => 0, // furnished dwellings score more — handled separately later
            _ => 0,
        }
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

enum TileGroup {
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
    gold: usize,
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
        Resources { gold: 0, begging: 0, wood: 0, stone: 0, coal: 0, rubies: 0, food: 0, wheat: 0, vegetables: 0 }
    }

}
impl std::ops::AddAssign for Resources {
    fn add_assign(&mut self, rhs: Resources) {
        self.gold += rhs.gold;
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

    fn tile_placements(&self, tile: TileGroup) -> Vec<[[Tile; BOARD_WIDTH]; BOARD_HEIGHT]> {
        let mut result = vec![];
        match tile {
            TileGroup::Single(t) => {
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
            TileGroup::Twin((t1, t2)) => {
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
        self.resources.gold as i32 +
        self.resources.rubies as i32 +
        {
            let field_wheat: usize = self.tiles.iter().flatten()
                .filter_map(|&t| if let Tile::Field((w, _)) = t { Some(w as usize) } else { None })
                .sum();
            let field_veg: usize = self.tiles.iter().flatten()
                .filter_map(|&t| if let Tile::Field((_, v)) = t { Some(v as usize) } else { None })
                .sum();
            let wheat = self.resources.wheat + field_wheat;
            let veg   = self.resources.vegetables + field_veg;
            ((wheat + 1) / 2 + veg) as i32
        } +
        self.dogs as i32 +
        self.animals.iter().sum::<usize>() as i32 -
        self.tiles.iter().flatten().map(|&t| t.points()).sum::<i32>() -
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

        // Donkeys can live in mines (1 per ore mine or ruby mine)
        let donkey_fixed_cap = self.tiles.iter().flatten()
            .filter(|&&t| matches!(t, Tile::OreMine | Tile::RubyMine))
            .count();

        let fixed_boar   = animals[AnimalType::Boar   as usize].min(boar_fixed);
        let fixed_sheep  = animals[AnimalType::Sheep  as usize].min(sheep_meadow);
        let fixed_donkey = animals[AnimalType::Donkey as usize].min(donkey_fixed_cap);
        // Remaining to assign to pastures/flex, in priority order: cattle, boar, donkey, sheep
        let need = [
            animals[AnimalType::Cow    as usize],
            animals[AnimalType::Boar   as usize] - fixed_boar,
            animals[AnimalType::Donkey as usize] - fixed_donkey,
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

        [best[AnimalType::Cow as usize], fixed_boar + best[AnimalType::Boar as usize], fixed_donkey + best[AnimalType::Donkey as usize], fixed_sheep + best[AnimalType::Sheep as usize]]
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

#[derive(Clone, PartialEq)]
enum Phase {
    Placement,
    Adventuring { space: ActionSpace, remaining_picks: usize, remaining_adventures: usize, used_items: u16 },
    Trading,
}

#[derive(Clone)]
pub struct State {
    pub players: Vec<Player>,
    pub round: u32,
    pub starting_player: u8,
    pub accumulated: [u32; ActionSpace::COUNT],
    current_player: usize,
    phase: Phase,
}
impl State {
    pub fn new(count: u32) -> Self {
        let mut players = Vec::new();
        for _ in 0..count {
            players.push(Player::new(2));
        }
        State { players, round: 0, starting_player: 0, accumulated: [0u32; ActionSpace::COUNT], current_player: 0, phase: Phase::Placement }
    }
    pub fn is_placement(&self) -> bool {
        self.phase == Phase::Placement
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

    fn next_placement_player(&self) -> Option<usize> {
        let n = self.players.len();
        for i in 1..=n {
            let next = (self.current_player + i) % n;
            if self.players[next].dwarfs.iter().any(|d| d.placed_on.is_none()) {
                return Some(next);
            }
        }
        None
    }

    // Used only by verify() — re-derives current_player from dwarf state.
    fn derive_current_player(&self) -> usize {
        let n = self.players.len();
        let placed: Vec<usize> = self.players.iter()
            .map(|p| p.dwarfs.iter().filter(|d| d.placed_on.is_some()).count())
            .collect();
        let total_placed: usize = placed.iter().sum();
        let total_dwarves: Vec<usize> = self.players.iter().map(|p| p.dwarfs.len()).collect();
        let mut turns = 0;
        let mut seat = self.starting_player as usize;
        let max_iter = total_dwarves.iter().sum::<usize>() * n + 1;
        for _ in 0..max_iter {
            if placed[seat] < total_dwarves[seat] {
                if turns == total_placed { return seat; }
                turns += 1;
            }
            seat = (seat + 1) % n;
        }
        self.starting_player as usize
    }

    pub fn verify(&self) {
        match self.phase {
            Phase::Placement => {
                let derived = self.derive_current_player();
                assert_eq!(
                    self.current_player, derived,
                    "current_player mismatch: explicit={}, derived={}",
                    self.current_player, derived
                );
            }
            Phase::Adventuring { space, remaining_picks, .. } => {
                assert!(remaining_picks >= 1, "Adventuring remaining_picks must be >= 1");
                assert!(
                    self.players[self.current_player].dwarfs.iter().any(|d| d.placed_on == Some(space)),
                    "current player has no dwarf on {:?} during Adventuring", space as usize
                );
            }
            Phase::Trading => {
                assert!(
                    self.players.iter().all(|p| p.dwarfs.iter().all(|d| d.placed_on.is_some())),
                    "not all dwarves placed during Trading phase"
                );
                assert!(
                    self.current_player < self.players.len(),
                    "current_player {} out of range", self.current_player
                );
            }
        }
        for player in &self.players {
            player.verify_pastures();
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

    fn adventure_options(&self, player_idx: usize, weapon: u8, used_items: u16) -> Vec<(Self, u16)> {
        if weapon == 0 {
            return vec![(self.clone(), 0)];
        }
        let min_weapon = |i: usize| -> u8 {
            match i { 0..=7 => (i / 2 + 1) as u8, 8 => 5, _ => 6 }
        };
        let avail = |i: usize| weapon >= min_weapon(i) && used_items & (1 << i) == 0;
        let mut results = vec![];
        if avail(0) { let mut c = self.clone(); c.players[player_idx].resources.wood += 1; results.push((c, 1u16 << 0)); }
        if avail(1) { let mut c = self.clone(); c.players[player_idx].dogs += 1; results.push((c, 1u16 << 1)); }
        if avail(2) { let mut c = self.clone(); c.players[player_idx].resources.wheat += 1; results.push((c, 1u16 << 2)); }
        if avail(3) { let mut c = self.clone(); c.players[player_idx].animals[AnimalType::Sheep as usize] += 1; results.push((c, 1u16 << 3)); }
        if avail(4) { let mut c = self.clone(); c.players[player_idx].resources.stone += 1; results.push((c, 1u16 << 4)); }
        if avail(5) { let mut c = self.clone(); c.players[player_idx].animals[AnimalType::Donkey as usize] += 1; results.push((c, 1u16 << 5)); }
        if avail(6) { let mut c = self.clone(); c.players[player_idx].resources.vegetables += 1; results.push((c, 1u16 << 6)); }
        if avail(7) { let mut c = self.clone(); c.players[player_idx].resources.coal += 2; results.push((c, 1u16 << 7)); }
        if avail(8) { let mut c = self.clone(); c.players[player_idx].animals[AnimalType::Boar as usize] += 1; results.push((c, 1u16 << 8)); }
        if avail(9) { let mut c = self.clone(); c.players[player_idx].resources.gold += 2; results.push((c, 1u16 << 9)); }
        // Weapon > 0 but all reachable items already picked — pass through with no reward
        if results.is_empty() { results.push((self.clone(), 0)); }
        results
    }

    fn forge_options(&self, player_idx: usize, space: ActionSpace) -> Vec<Self> {
        let dwarf = self.players[player_idx].dwarfs.iter()
            .find(|d| d.placed_on == Some(space))
            .expect("no dwarf on space");
        if dwarf.weapon != 0 {
            return vec![];
        }
        let coal = self.players[player_idx].resources.coal;
        (1..=coal.min(8)).map(|strength| {
            let mut c = self.clone();
            c.players[player_idx].resources.coal -= strength;
            c.players[player_idx].dwarfs.iter_mut()
                .find(|d| d.placed_on == Some(space))
                .expect("no dwarf on space in forge_options")
                .weapon = strength as u8;
            c
        }).collect()
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
        self.current_player
    }

    fn num_players(&self) -> usize {
        self.players.len()
    }

    fn children<R: rand::prelude::Rng>(&self, _rng: &mut R) -> Vec<Self> {
        if self.done() {
            return vec![];
        }

        let current = self.current_player;

        match self.phase {
            Phase::Placement => {
                let mut children = vec![];
                for &space in &ActionSpace::ALL {
                    let occupied = self.players.iter()
                        .any(|p| p.dwarfs.iter().any(|d| d.placed_on == Some(space)));
                    if occupied { continue; }

                    let mut child = self.clone();
                    child.players[current].dwarfs.iter_mut()
                        .find(|d| d.placed_on.is_none())
                        .expect("current player has no unplaced dwarf")
                        .placed_on = Some(space);
                    space.gain_resources(child.accumulated[space as usize], &mut child.players[current].resources);
                    space.gain_animals(child.accumulated[space as usize], &mut child.players[current].animals);
                    if space == ActionSpace::StartingPlayer {
                        child.starting_player = current as u8;
                    }
                    if space == ActionSpace::OreMining || space == ActionSpace::OreDelivery {
                        let mines = child.players[current].tiles.iter().flatten()
                            .filter(|&&t| t == Tile::OreMine).count();
                        child.players[current].resources.coal += mines * 2;
                    }
                    if space == ActionSpace::RubyMining {
                        let has_mine = child.players[current].tiles.iter().flatten()
                            .any(|&t| t == Tile::RubyMine);
                        if has_mine { child.players[current].resources.rubies += 1; }
                    }
                    if space == ActionSpace::RubyDelivery {
                        let mines = child.players[current].tiles.iter().flatten()
                            .filter(|&&t| t == Tile::RubyMine).count();
                        if mines >= 2 { child.players[current].resources.rubies += 1; }
                    }

                    let next = child.next_placement_player();

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
                                    let replaced = board_delta(&child.players[current].tiles, &board);
                                    space.gain_placement_resources(replaced, &mut c.players[current].resources);
                                    for pos in changed_cells(&child.players[current].tiles, &board) {
                                        let p = &mut c.players[current];
                                        apply_location_bonus(pos, &mut p.resources, &mut p.animals);
                                    }
                                    c.players[current].tiles = board;
                                    c
                                }).collect()
                            }
                        }).collect()
                    };

                    if space == ActionSpace::Blacksmithing {
                        candidates = candidates.into_iter()
                            .flat_map(|c| c.forge_options(current, space))
                            .collect();
                    }
                    if space == ActionSpace::Adventure {
                        candidates = candidates.into_iter()
                            .flat_map(|c| {
                                let armed = c.players[current].dwarfs.iter()
                                    .find(|d| d.placed_on == Some(space))
                                    .map_or(false, |d| d.weapon > 0);
                                if armed {
                                    vec![c]
                                } else {
                                    let mut opts = vec![c.clone()];
                                    opts.extend(c.forge_options(current, space));
                                    opts
                                }
                            })
                            .collect();
                    }

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

                    let adventures = space.adventure_count();

                    for c in &mut candidates {
                        if adventures > 0 {
                            c.phase = Phase::Adventuring { space, remaining_picks: space.picks_per_adventure(), remaining_adventures: adventures, used_items: 0 };
                        } else {
                            match next {
                                Some(p) => c.current_player = p,
                                None => { c.phase = Phase::Trading; c.current_player = 0; }
                            }
                        }
                    }

                    children.extend(candidates);
                }
                children
            }

            Phase::Adventuring { space, remaining_picks, remaining_adventures, used_items } => {
                let weapon = self.players[current].dwarfs.iter()
                    .find(|d| d.placed_on == Some(space))
                    .map(|d| d.weapon)
                    .unwrap_or(0);
                self.adventure_options(current, weapon, used_items)
                    .into_iter()
                    .map(|(mut c, item_bit)| {
                        let new_used = used_items | item_bit;
                        if remaining_picks == 1 {
                            if remaining_adventures == 1 {
                                if let Some(d) = c.players[current].dwarfs.iter_mut()
                                    .find(|d| d.placed_on == Some(space))
                                {
                                    if d.weapon > 0 { d.weapon = d.weapon.saturating_add(1); }
                                }
                                match c.next_placement_player() {
                                    Some(p) => { c.phase = Phase::Placement; c.current_player = p; }
                                    None    => { c.phase = Phase::Trading;   c.current_player = 0; }
                                }
                            } else {
                                c.phase = Phase::Adventuring { space, remaining_picks: space.picks_per_adventure(), remaining_adventures: remaining_adventures - 1, used_items: 0 };
                            }
                        } else {
                            c.phase = Phase::Adventuring { space, remaining_picks: remaining_picks - 1, remaining_adventures, used_items: new_used };
                        }
                        c
                    })
                    .collect()
            }

            Phase::Trading => {
                let food_needed = self.players[current].dwarfs.len();
                let mut children = vec![];

                // "Done trading" — advance to next player or execute harvest
                let mut done = self.clone();
                if current + 1 < self.players.len() {
                    done.current_player = current + 1;
                } else {
                    done.replenish();
                    done.return_dwarfs();
                    done.harvest();
                    done.round += 1;
                    done.current_player = done.starting_player as usize;
                    done.phase = Phase::Placement;
                }
                children.push(done);

                // Trade actions — only offered when more food is needed.
                // For each resource, offer 1..=n units where n covers the gap
                // (ceiling division, so multi-food trades may overshoot by a little).
                let p = &self.players[current];
                let food_gap = food_needed.saturating_sub(p.resources.food);

                if food_gap > 0 {
                    let sheep  = p.animals[AnimalType::Sheep  as usize];
                    let boar   = p.animals[AnimalType::Boar   as usize];
                    let cow    = p.animals[AnimalType::Cow    as usize];
                    let donkey = p.animals[AnimalType::Donkey as usize];
                    let wheat  = p.resources.wheat;
                    let veg    = p.resources.vegetables;
                    let rubies = p.resources.rubies;

                    // 1 sheep → 1 food
                    for n in 1..=sheep.min(food_gap) {
                        let mut c = self.clone();
                        c.players[current].animals[AnimalType::Sheep as usize] -= n;
                        c.players[current].resources.food += n;
                        children.push(c);
                    }

                    // 1 boar → 2 food
                    for n in 1..=boar.min((food_gap + 1) / 2) {
                        let mut c = self.clone();
                        c.players[current].animals[AnimalType::Boar as usize] -= n;
                        c.players[current].resources.food += n * 2;
                        children.push(c);
                    }

                    // 1 cow → 3 food
                    for n in 1..=cow.min((food_gap + 2) / 3) {
                        let mut c = self.clone();
                        c.players[current].animals[AnimalType::Cow as usize] -= n;
                        c.players[current].resources.food += n * 3;
                        children.push(c);
                    }

                    // 1 donkey → 1 food (only when no pair trade is possible)
                    if donkey == 1 {
                        let mut c = self.clone();
                        c.players[current].animals[AnimalType::Donkey as usize] -= 1;
                        c.players[current].resources.food += 1;
                        children.push(c);
                    }

                    // 2 donkeys → 3 food (bulk rate)
                    for n in 1..=(donkey / 2).min((food_gap + 2) / 3) {
                        let mut c = self.clone();
                        c.players[current].animals[AnimalType::Donkey as usize] -= n * 2;
                        c.players[current].resources.food += n * 3;
                        children.push(c);
                    }

                    // 1 wheat → 1 food
                    for n in 1..=wheat.min(food_gap) {
                        let mut c = self.clone();
                        c.players[current].resources.wheat -= n;
                        c.players[current].resources.food += n;
                        children.push(c);
                    }

                    // 1 vegetable → 2 food
                    for n in 1..=veg.min((food_gap + 1) / 2) {
                        let mut c = self.clone();
                        c.players[current].resources.vegetables -= n;
                        c.players[current].resources.food += n * 2;
                        children.push(c);
                    }

                    // 1 ruby → 2 food
                    for n in 1..=rubies.min((food_gap + 1) / 2) {
                        let mut c = self.clone();
                        c.players[current].resources.rubies -= n;
                        c.players[current].resources.food += n * 2;
                        children.push(c);
                    }
                }

                children
            }
        }
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