use crate::mcts::GameState;

#[derive(Clone, Copy, PartialEq)]
#[repr(usize)]
pub enum ActionSpace {
    DriftMining = 0,
    Logging = 1,
    WoodGathering = 2,
    Excavation = 3,
    Supplies = 4,
    Clearing = 5,
    StartingPlayer = 6,
    OreMining = 7,
    Sustenance = 8,
    RubyMining = 9,
    Housework = 10,
    SlashAndBurn = 11,
    SheepFarming = 12,
    OreMineConstruction = 13,
    Blacksmithing = 14,
    WishForChildren = 15,
    RubyMineConstruction = 16,
    DonkeyFarming = 17,
    FamilyLife = 18,
    OreDelivery = 19,
    RubyDelivery = 20,
    Adventure = 21,
}
impl ActionSpace {
    const COUNT: usize = 22;
    const ALL: [ActionSpace; Self::COUNT] = [
        ActionSpace::DriftMining,
        ActionSpace::Logging,
        ActionSpace::WoodGathering,
        ActionSpace::Excavation,
        ActionSpace::Supplies,
        ActionSpace::Clearing,
        ActionSpace::StartingPlayer,
        ActionSpace::OreMining,
        ActionSpace::Sustenance,
        ActionSpace::RubyMining,
        ActionSpace::Housework,
        ActionSpace::SlashAndBurn,
        ActionSpace::SheepFarming,
        ActionSpace::OreMineConstruction,
        ActionSpace::Blacksmithing,
        ActionSpace::WishForChildren,
        ActionSpace::RubyMineConstruction,
        ActionSpace::DonkeyFarming,
        ActionSpace::FamilyLife,
        ActionSpace::OreDelivery,
        ActionSpace::RubyDelivery,
        ActionSpace::Adventure,
    ];

    fn picks_per_expedition(self) -> usize {
        match self {
            ActionSpace::Logging => 1,
            ActionSpace::OreMineConstruction => 2,
            ActionSpace::Blacksmithing => 3,
            ActionSpace::Adventure => 1,
            _ => 0,
        }
    }

    fn expedition_count(self) -> usize {
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
            | ActionSpace::Blacksmithing | ActionSpace::Adventure | ActionSpace::Housework
            | ActionSpace::WishForChildren | ActionSpace::FamilyLife => {}
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
                if matches!(replaced, TileGroup::Single(Tile::DeepTunnel)) {
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
                vec![TileGroup::Twin((Tile::DeepTunnel, Tile::OreMine))],
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

pub(crate) const HALF_WIDTH: usize = 3;
pub(crate) const BOARD_HEIGHT: usize = 4;

pub(crate) type Board = [[Tile; HALF_WIDTH]; BOARD_HEIGHT];

#[derive(Clone, Copy)]
enum Side { Outdoor, Indoor }

fn changed_cells(old: &Board, new: &Board) -> Vec<(usize, usize)> {
    (0..BOARD_HEIGHT).flat_map(|y| (0..HALF_WIDTH).map(move |x| (x, y)))
        .filter(|&(x, y)| old[y][x] != new[y][x])
        .collect()
}

fn board_delta(old: &Board, new: &Board) -> TileGroup {
    let replaced: Vec<Tile> = changed_cells(old, new).into_iter()
        .map(|(x, y)| old[y][x])
        .collect();
    match replaced.as_slice() {
        &[t]      => TileGroup::Single(t),
        &[t1, t2] => TileGroup::Twin((t1, t2)),
        _         => panic!("unexpected board delta: {} tiles changed", replaced.len()),
    }
}

fn apply_outdoor_location_bonus((x, y): (usize, usize), resources: &mut Resources, animals: &mut Animals) {
    match (x, y) {
        (0, 2) | (2, 0) => animals[AnimalType::Boar as usize] += 1,
        (1, 3)          => resources.food += 1,
        _               => {}
    }
}

fn apply_indoor_location_bonus((x, y): (usize, usize), resources: &mut Resources, _animals: &mut Animals) {
    match (x, y) {
        (2, 0) => resources.food += 2,
        (1, 3) => resources.food += 1,
        _      => {}
    }
}

fn adjacents(x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> {
    [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)]
        .into_iter()
        .filter(|&(nx, ny)| nx < HALF_WIDTH && ny < BOARD_HEIGHT)
}

fn adjacent_to_developed(board: &Board, x: usize, y: usize) -> bool {
    adjacents(x, y).any(|(nx, ny)| !board[ny][nx].is_undeveloped())
}

fn cells_adjacent_to_developed(board: &Board) -> Vec<(usize, usize)> {
    if board.iter().flatten().all(|t| t.is_undeveloped()) {
        return vec![(2, 3)];
    }
    (0..BOARD_HEIGHT).flat_map(|y| (0..HALF_WIDTH).map(move |x| (x, y)))
        .filter(|&(x, y)| adjacent_to_developed(board, x, y))
        .collect()
}

fn tile_placements_on(board: &Board, tile: TileGroup) -> Vec<Board> {
    let mut result = vec![];
    match tile {
        TileGroup::Single(t) => {
            let anchors: Vec<(usize, usize)> = if t.requires_adjacency() {
                cells_adjacent_to_developed(board).into_iter()
                    .filter(|&(x, y)| t.can_place_on(board[y][x]))
                    .collect()
            } else {
                (0..BOARD_HEIGHT).flat_map(|y| (0..HALF_WIDTH).map(move |x| (x, y)))
                    .filter(|&(x, y)| t.can_place_on(board[y][x]))
                    .collect()
            };
            for (x, y) in anchors {
                let mut b = *board;
                b[y][x] = t;
                result.push(b);
            }
        }
        TileGroup::Twin((t1, t2)) => {
            let requires_adj = t1.requires_adjacency();
            let anchors: std::collections::HashSet<(usize, usize)> = if requires_adj {
                cells_adjacent_to_developed(board).into_iter().collect()
            } else {
                std::collections::HashSet::new()
            };
            for y in 0..BOARD_HEIGHT {
                for x in 0..HALF_WIDTH {
                    for (x2, y2) in [(x + 1, y), (x, y + 1)] {
                        if x2 >= HALF_WIDTH || y2 >= BOARD_HEIGHT { continue; }
                        if !t1.can_place_on(board[y][x]) || !t2.can_place_on(board[y2][x2]) { continue; }
                        let valid = !requires_adj
                            || anchors.contains(&(x, y))
                            || anchors.contains(&(x2, y2));
                        if valid {
                            let mut b = *board;
                            b[y][x] = t1; b[y2][x2] = t2;
                            result.push(b);
                            if t1 != t2 {
                                let mut b = *board;
                                b[y][x] = t2; b[y2][x2] = t1;
                                result.push(b);
                            }
                        }
                    }
                }
            }
        }
    }
    result
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Tile {
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
    DeepTunnel,
    OreMine,
    RubyMine,
    Cave,
    Dwelling,
}

impl Tile {
    fn is_undeveloped(self) -> bool {
        matches!(self, Tile::Forest | Tile::Mountain)
    }

    fn side(self) -> Side {
        if matches!(self, Tile::Forest | Tile::ForestStable | Tile::Meadow | Tile::MeadowStable
            | Tile::Pasture | Tile::PastureStable | Tile::Field(_))
        { Side::Outdoor } else { Side::Indoor }
    }

    fn can_place_on(self, other: Tile) -> bool {
        matches!((self, other),
            (Tile::Meadow | Tile::MeadowStable | Tile::Pasture | Tile::PastureStable | Tile::Field(_), Tile::Forest)
            | (Tile::Tunnel | Tile::Cave, Tile::Mountain)
            | (Tile::DeepTunnel | Tile::OreMine, Tile::Tunnel)
            | (Tile::RubyMine, Tile::Tunnel | Tile::DeepTunnel)
            | (Tile::Dwelling, Tile::Cave)
        )
    }

    fn requires_adjacency(self) -> bool {
        !matches!(self, Tile::DeepTunnel | Tile::OreMine | Tile::RubyMine | Tile::Dwelling)
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

#[derive(Clone, Copy)]
struct Furnishing {
    tile: Tile,
    cost_wood: usize,
    cost_stone: usize,
    max_count: usize,
}

// All cave furnishings (used by Housework and Adventure loot)
const FURNISHINGS: &[Furnishing] = &[
    Furnishing { tile: Tile::Dwelling, cost_wood: 4, cost_stone: 3, max_count: 16 },
];

// Only dwelling-type furnishings (used by WishForChildren, which cannot furnish other cave types)
const DWELLING_FURNISHINGS: &[Furnishing] = &[
    Furnishing { tile: Tile::Dwelling, cost_wood: 4, cost_stone: 3, max_count: 16 },
];

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
    pub(crate) gold: usize,
    pub(crate) begging: usize,

    pub(crate) wood: usize,
    pub(crate) stone: usize,
    pub(crate) coal: usize,
    pub(crate) rubies: usize,
    pub(crate) food: usize,

    pub(crate) wheat: usize,
    pub(crate) vegetables: usize,
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
    pub children: usize,
    pub(crate) outdoor: Board,
    pub(crate) indoor: Board,
    pub(crate) resources: Resources,
    pub(crate) dogs: usize,
    pub(crate) animals: Animals, // indexed by AnimalType
    pastures: Vec<Pasture>,
}
impl Player {
    fn new(food: usize) -> Self {
        let outdoor = [[Tile::Forest; HALF_WIDTH]; BOARD_HEIGHT];
        let mut indoor = [[Tile::Mountain; HALF_WIDTH]; BOARD_HEIGHT];
        indoor[3][0] = Tile::Dwelling;
        indoor[2][0] = Tile::Cave;
        let mut player = Player {
            dwarfs: vec![Dwarf { weapon: 0, placed_on: None }, Dwarf { weapon: 0, placed_on: None }],
            children: 0,
            outdoor,
            indoor,
            resources: Resources::zero(),
            dogs: 0,
            animals: [0; 4],
            pastures: vec![],
        };
        player.resources.food = food;
        player
    }

    pub fn food_needed(&self) -> usize {
        self.dwarfs.len() * 2 + self.children
    }

    pub fn dwellings_needed(&self) -> usize {
        self.dwarfs.len() + self.children
    }

    fn dwelling_capacity(&self) -> usize {
        self.indoor.iter().flatten().filter(|&&t| t == Tile::Dwelling).count() + 1
    }

    fn grow_children(&mut self) {
        for _ in 0..self.children {
            self.dwarfs.push(Dwarf { placed_on: None, weapon: 0 });
        }
        self.children = 0;
    }

    fn tile_placements(&self, tile: TileGroup) -> Vec<(Side, Board)> {
        let first = match tile { TileGroup::Single(t) => t, TileGroup::Twin((t, _)) => t };
        match first.side() {
            Side::Outdoor => tile_placements_on(&self.outdoor, tile).into_iter().map(|b| (Side::Outdoor, b)).collect(),
            Side::Indoor  => tile_placements_on(&self.indoor,  tile).into_iter().map(|b| (Side::Indoor,  b)).collect(),
        }
    }

    pub fn points(&self) -> i32 {
        self.dwarfs.len() as i32 +
        self.resources.gold as i32 +
        self.resources.rubies as i32 +
        {
            let field_wheat: usize = self.outdoor.iter().flatten()
                .filter_map(|&t| if let Tile::Field((w, _)) = t { Some(w as usize) } else { None })
                .sum();
            let field_veg: usize = self.outdoor.iter().flatten()
                .filter_map(|&t| if let Tile::Field((_, v)) = t { Some(v as usize) } else { None })
                .sum();
            let wheat = self.resources.wheat + field_wheat;
            let veg   = self.resources.vegetables + field_veg;
            ((wheat + 1) / 2 + veg) as i32
        } +
        self.dogs as i32 +
        self.animals.iter().sum::<usize>() as i32 +
        {
            // 3 points per dwelling; subtract 3 for the initial unfurnished one
            let dwellings = self.indoor.iter().flatten().filter(|&&t| t == Tile::Dwelling).count() as i32;
            dwellings * 3 - 3
        } -
        self.outdoor.iter().flatten().map(|&t| t.points()).sum::<i32>() -
        self.indoor.iter().flatten().map(|&t| t.points()).sum::<i32>() -
        self.resources.begging as i32 * 3 -
        self.outdoor.iter().flatten().filter(|&&t| t.is_undeveloped()).count() as i32 -
        self.indoor.iter().flatten().filter(|&&t| t.is_undeveloped()).count() as i32 -
        self.animals.iter().filter(|&&n| n == 0).count() as i32
    }
    fn trim_animals(&self, animals: Animals) -> Animals {
        // Boars can live in forest stables (1 per stable)
        let boar_fixed = self.outdoor.iter().flatten()
            .filter(|&&t| t == Tile::ForestStable)
            .count();
        // Sheep can live on unfenced meadows, guarded by dogs (dogs+1 total if any meadow exists)
        let sheep_meadow = if self.outdoor.iter().flatten().any(|&t| t == Tile::Meadow) {
            self.dogs + 1
        } else {
            0
        };
        // Flexible slots: each MeadowStable holds 1 farm animal; dwelling holds 2
        let flex = self.outdoor.iter().flatten()
            .filter(|&&t| t == Tile::MeadowStable)
            .count() + 2;

        // Capacity per pasture: cells * 2, doubled for each stable in the pasture
        let pasture_caps: Vec<usize> = self.pastures.iter().map(|p| {
            let stables = p.cells.iter()
                .filter(|&&(x, y)| self.outdoor[y][x] == Tile::PastureStable)
                .count();
            p.cells.len() * 2 * (1 << stables)
        }).collect();

        // Donkeys can live in mines (1 per ore mine or ruby mine)
        let donkey_fixed_cap = self.indoor.iter().flatten()
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
        self.outdoor.iter().flatten()
            .filter(|&&t| matches!(t, Tile::ForestStable | Tile::MeadowStable | Tile::PastureStable))
            .count()
    }

    fn feed(&mut self) {
        let needed = self.food_needed();
        if self.resources.food >= needed {
            self.resources.food -= needed;
        } else {
            self.resources.begging += needed - self.resources.food;
            self.resources.food = 0;
        }
    }
    fn harvest(&mut self) {
        for y in 0..BOARD_HEIGHT {
            for x in 0..HALF_WIDTH {
                match self.outdoor[y][x] {
                    Tile::Field((w, _)) if w > 0 => {
                        self.resources.wheat += 1;
                        self.outdoor[y][x] = Tile::Field((w - 1, 0));
                    }
                    Tile::Field((_, v)) if v > 0 => {
                        self.resources.vegetables += 1;
                        self.outdoor[y][x] = Tile::Field((0, v - 1));
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
                    matches!(self.outdoor[y][x], Tile::Pasture | Tile::PastureStable),
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
pub(crate) enum Phase {
    Placement,
    Expedition { space: ActionSpace, remaining_picks: usize, remaining_expeditions: usize, used_items: u16 },
    Trading,
}

#[derive(Clone)]
pub struct State {
    pub players: Vec<Player>,
    pub round: u32,
    pub starting_player: u8,
    pub accumulated: [u32; ActionSpace::COUNT],
    pub(crate) current_player: usize,
    pub(crate) phase: Phase,
}
impl State {
    pub fn new(count: u32) -> Self {
        let mut players = Vec::new();
        for _ in 0..count {
            players.push(Player::new(2));
        }
        State { players, round: 0, starting_player: 0, accumulated: [0u32; ActionSpace::COUNT], current_player: 0, phase: Phase::Placement }
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
    fn space_available(&self, space: ActionSpace) -> bool {
        (space as usize) <= 12 + self.round as usize
    }

    fn replenish(&mut self) {
        for &space in &ActionSpace::ALL {
            if !self.space_available(space) { continue; }
            let i = space as usize;
            let taken = self.players.iter()
                .any(|p| p.dwarfs.iter().any(|d| d.placed_on == Some(space)));
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

    fn grow_children(&mut self) {
        for player in &mut self.players {
            player.grow_children();
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
            Phase::Expedition { space, remaining_picks, .. } => {
                assert!(remaining_picks >= 1, "Expedition remaining_picks must be >= 1");
                assert!(
                    self.players[self.current_player].dwarfs.iter().any(|d| d.placed_on == Some(space)),
                    "current player has no dwarf on {:?} during Expedition", space as usize
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
            .flat_map(|y| (0..HALF_WIDTH).map(move |x| (x, y)))
            .filter(|&(x, y)| player.outdoor[y][x].is_fenceable())
            .collect();

        if player.resources.wood >= 2 {
            for &(x, y) in &meadows {
                let mut child = self.clone();
                child.players[player_idx].outdoor[y][x] = fence(player.outdoor[y][x]);
                child.players[player_idx].resources.wood -= 2;
                child.players[player_idx].pastures.push(Pasture { cells: vec![(x, y)], animals: None });
                results.push(child);
            }
        }

        if player.resources.wood >= 4 {
            for &(x, y) in &meadows {
                for (x2, y2) in [(x + 1, y), (x, y + 1)] {
                    if x2 >= HALF_WIDTH || y2 >= BOARD_HEIGHT { continue; }
                    if !player.outdoor[y2][x2].is_fenceable() { continue; }
                    let mut child = self.clone();
                    child.players[player_idx].outdoor[y][x] = fence(player.outdoor[y][x]);
                    child.players[player_idx].outdoor[y2][x2] = fence(player.outdoor[y2][x2]);
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
            for x in 0..HALF_WIDTH {
                if let Some(t) = player.outdoor[y][x].maybe_add_stable() {
                    let mut child = self.clone();
                    child.players[player_idx].outdoor[y][x] = t;
                    child.players[player_idx].resources.stone -= 1;
                    results.push(child);
                }
            }
        }

        results
    }

    fn expedition_options(&self, player_idx: usize, weapon: u8, used_items: u16) -> Vec<(Self, u16)> {
        if weapon == 0 {
            return vec![(self.clone(), 0)];
        }
        // Item indices: 0 reserved (future lvl-1 item), then pairs per level:
        //   lvl 1 → 1,2 | lvl 2 → 3,4 | lvl 3 → 5,6 | lvl 4 → 7,8
        //   lvl 5 → 9,10 | lvl 6 → 11,12 | lvl 7 → 13,14
        // min_weapon(i) = ((i+1)/2).max(1)
        let min_weapon = |i: usize| -> u8 { (((i + 1) / 2).max(1)) as u8 };
        let avail = |i: usize| weapon >= min_weapon(i) && used_items & (1 << i) == 0;
        let mut results = vec![];
        // Level 1 (indices 1, 2)
        if avail(1) { let mut c = self.clone(); c.players[player_idx].resources.wood += 1; results.push((c, 1u16 << 1)); }
        if avail(2) { let mut c = self.clone(); c.players[player_idx].dogs += 1; results.push((c, 1u16 << 2)); }
        // Level 2 (indices 3, 4)
        if avail(3) { let mut c = self.clone(); c.players[player_idx].resources.wheat += 1; results.push((c, 1u16 << 3)); }
        if avail(4) { let mut c = self.clone(); c.players[player_idx].animals[AnimalType::Sheep as usize] += 1; results.push((c, 1u16 << 4)); }
        // Level 3 (indices 5, 6)
        if avail(5) { let mut c = self.clone(); c.players[player_idx].resources.stone += 1; results.push((c, 1u16 << 5)); }
        if avail(6) { let mut c = self.clone(); c.players[player_idx].animals[AnimalType::Donkey as usize] += 1; results.push((c, 1u16 << 6)); }
        // Level 4 (indices 7, 8)
        if avail(7) { let mut c = self.clone(); c.players[player_idx].resources.vegetables += 1; results.push((c, 1u16 << 7)); }
        if avail(8) { let mut c = self.clone(); c.players[player_idx].resources.coal += 2; results.push((c, 1u16 << 8)); }
        // Level 5 (indices 9, 10)
        if avail(9) { let mut c = self.clone(); c.players[player_idx].animals[AnimalType::Boar as usize] += 1; results.push((c, 1u16 << 9)); }
        // Level 6 (indices 11, 12)
        if avail(11) { let mut c = self.clone(); c.players[player_idx].resources.gold += 2; results.push((c, 1u16 << 11)); }
        // Level 7 (indices 13, 14)
        if avail(13) {
            for c in self.furnish_cave_options(player_idx, FURNISHINGS) {
                results.push((c, 1u16 << 13));
            }
        }
        // Weapon > 0 but all reachable items already picked — pass through with no reward
        if results.is_empty() { results.push((self.clone(), 0)); }
        results
    }

    fn furnish_cave_options(&self, player_idx: usize, furnishings: &[Furnishing]) -> Vec<Self> {
        let p = &self.players[player_idx];
        let mut opts = vec![];
        for y in 0..BOARD_HEIGHT {
            for x in 0..HALF_WIDTH {
                if p.indoor[y][x] != Tile::Cave {
                    continue;
                }
                for f in furnishings {
                    if p.resources.wood < f.cost_wood || p.resources.stone < f.cost_stone {
                        continue;
                    }
                    let used = self.players.iter()
                        .flat_map(|pl| pl.indoor.iter().flatten())
                        .filter(|&&t| t == f.tile)
                        .count();
                    if used >= f.max_count {
                        continue;
                    }
                    let mut c = self.clone();
                    c.players[player_idx].resources.wood -= f.cost_wood;
                    c.players[player_idx].resources.stone -= f.cost_stone;
                    c.players[player_idx].indoor[y][x] = f.tile;
                    opts.push(c);
                }
            }
        }
        opts
    }

    fn family_growth_option(&self, player_idx: usize) -> Option<Self> {
        let p = &self.players[player_idx];
        if p.dwellings_needed() < p.dwelling_capacity() {
            let mut c = self.clone();
            c.players[player_idx].children += 1;
            Some(c)
        } else {
            None
        }
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
        let fields: Vec<(usize, usize)> = (0..BOARD_HEIGHT)
            .flat_map(|y| (0..HALF_WIDTH).map(move |x| (x, y)))
            .filter(|&(x, y)| player.outdoor[y][x] == Tile::Field((0, 0)))
            .collect();
        let max_wheat = player.resources.wheat.min(2);
        let max_veg = player.resources.vegetables.min(2);
        let mut results = vec![];
        for w in 0..=max_wheat {
            for v in 0..=max_veg {
                if w == 0 && v == 0 { continue; }
                if w + v > fields.len() { continue; }
                let mut child = self.clone();
                child.players[player_idx].resources.wheat -= w;
                child.players[player_idx].resources.vegetables -= v;
                for &(x, y) in &fields[..w] {
                    child.players[player_idx].outdoor[y][x] = Tile::Field((3, 0));
                }
                for &(x, y) in &fields[w..w + v] {
                    child.players[player_idx].outdoor[y][x] = Tile::Field((0, 2));
                }
                results.push(child);
            }
        }
        results
    }
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
                    if !self.space_available(space) { continue; }
                    let occupied = self.players.iter()
                        .any(|p| p.dwarfs.iter().any(|d| d.placed_on == Some(space)));
                    if occupied { continue; }

                    let mut child = self.clone();
                    child.players[current].dwarfs.iter_mut()
                        .filter(|d| d.placed_on.is_none())
                        .min_by_key(|d| d.weapon)
                        .expect("current player has no unplaced dwarf")
                        .placed_on = Some(space);
                    space.gain_resources(child.accumulated[space as usize], &mut child.players[current].resources);
                    space.gain_animals(child.accumulated[space as usize], &mut child.players[current].animals);
                    if space == ActionSpace::StartingPlayer {
                        child.starting_player = current as u8;
                    }
                    if space == ActionSpace::OreMining || space == ActionSpace::OreDelivery {
                        let mines = child.players[current].indoor.iter().flatten()
                            .filter(|&&t| t == Tile::OreMine).count();
                        child.players[current].resources.coal += mines * 2;
                    }
                    if space == ActionSpace::RubyMining {
                        let has_mine = child.players[current].indoor.iter().flatten()
                            .any(|&t| t == Tile::RubyMine);
                        if has_mine { child.players[current].resources.rubies += 1; }
                    }
                    if space == ActionSpace::RubyDelivery {
                        let mines = child.players[current].indoor.iter().flatten()
                            .filter(|&&t| t == Tile::RubyMine).count();
                        if mines >= 2 { child.players[current].resources.rubies += 1; }
                    }

                    let next = child.next_placement_player();

                    let tile_choices = space.place_tile();
                    let mut candidates = if tile_choices.is_empty() {
                        vec![child]
                    } else {
                        tile_choices.into_iter().flat_map(|tile| {
                            let placements = child.players[current].tile_placements(tile);
                            if placements.is_empty() {
                                vec![child.clone()]
                            } else {
                                placements.into_iter().map(|(side, new_board)| {
                                    let mut c = child.clone();
                                    let (replaced, changed) = match side {
                                        Side::Outdoor => (
                                            board_delta(&child.players[current].outdoor, &new_board),
                                            changed_cells(&child.players[current].outdoor, &new_board),
                                        ),
                                        Side::Indoor => (
                                            board_delta(&child.players[current].indoor, &new_board),
                                            changed_cells(&child.players[current].indoor, &new_board),
                                        ),
                                    };
                                    space.gain_placement_resources(replaced, &mut c.players[current].resources);
                                    for pos in changed {
                                        let p = &mut c.players[current];
                                        match side {
                                            Side::Outdoor => apply_outdoor_location_bonus(pos, &mut p.resources, &mut p.animals),
                                            Side::Indoor  => apply_indoor_location_bonus(pos, &mut p.resources, &mut p.animals),
                                        }
                                    }
                                    match side {
                                        Side::Outdoor => c.players[current].outdoor = new_board,
                                        Side::Indoor  => c.players[current].indoor  = new_board,
                                    }
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

                    if space == ActionSpace::Housework {
                        for c in &mut candidates {
                            c.players[current].dogs += 1;
                        }
                        candidates = candidates.into_iter()
                            .flat_map(|c| {
                                let opts = c.furnish_cave_options(current, FURNISHINGS);
                                if opts.is_empty() { vec![c] } else { opts }
                            })
                            .collect();
                    }

                    if space == ActionSpace::WishForChildren {
                        candidates = candidates.into_iter()
                            .flat_map(|c| {
                                let mut opts = vec![];
                                if let Some(grown) = c.family_growth_option(current) {
                                    opts.push(grown);
                                }
                                opts.extend(c.furnish_cave_options(current, DWELLING_FURNISHINGS));
                                opts
                            })
                            .collect();
                    }

                    if space == ActionSpace::FamilyLife {
                        candidates = candidates.into_iter()
                            .flat_map(|c| {
                                let mut opts: Vec<Self> = vec![];
                                if let Some(grown) = c.family_growth_option(current) {
                                    opts.push(grown.clone()); // grow only
                                    opts.extend(grown.sow_options(current)); // grow + sow
                                }
                                opts.extend(c.sow_options(current)); // sow only (empty if nothing to sow)
                                opts
                            })
                            .collect();
                    }

                    if space == ActionSpace::SlashAndBurn {
                        candidates = candidates.into_iter()
                            .flat_map(|c| {
                                let sow = c.sow_options(current);
                                if sow.is_empty() { vec![c] } else { sow }
                            })
                            .collect();
                    }

                    if matches!(space, ActionSpace::SheepFarming | ActionSpace::DonkeyFarming) {
                        candidates = candidates.into_iter()
                            .flat_map(|c| c.pasture_options(current))
                            .flat_map(|c| c.stable_options(current))
                            .collect();
                    }

                    let expeditions = space.expedition_count();

                    for c in &mut candidates {
                        if expeditions > 0 {
                            c.phase = Phase::Expedition { space, remaining_picks: space.picks_per_expedition(), remaining_expeditions: expeditions, used_items: 0 };
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

            Phase::Expedition { space, remaining_picks, remaining_expeditions, used_items } => {
                let weapon = self.players[current].dwarfs.iter()
                    .find(|d| d.placed_on == Some(space))
                    .map(|d| d.weapon)
                    .unwrap_or(0);
                self.expedition_options(current, weapon, used_items)
                    .into_iter()
                    .map(|(mut c, item_bit)| {
                        let new_used = used_items | item_bit;
                        if remaining_picks == 1 {
                            if remaining_expeditions == 1 {
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
                                c.phase = Phase::Expedition { space, remaining_picks: space.picks_per_expedition(), remaining_expeditions: remaining_expeditions - 1, used_items: 0 };
                            }
                        } else {
                            c.phase = Phase::Expedition { space, remaining_picks: remaining_picks - 1, remaining_expeditions, used_items: new_used };
                        }
                        c
                    })
                    .collect()
            }

            Phase::Trading => {
                let food_needed = self.players[current].food_needed();
                let mut children = vec![];

                // "Done trading" — advance to next player or execute harvest
                let mut done = self.clone();
                if current + 1 < self.players.len() {
                    done.current_player = current + 1;
                } else {
                    done.replenish();
                    done.return_dwarfs();
                    done.harvest();
                    done.grow_children();
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
