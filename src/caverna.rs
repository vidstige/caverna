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

    fn apply_gains(self, accumulated: u32, player: &mut Player) {
        self.gain_resources(accumulated, &mut player.resources);
        self.gain_animals(accumulated, &mut player.animals);
        if matches!(self, ActionSpace::OreMining | ActionSpace::OreDelivery) {
            let mines = player.indoor.iter().flatten().filter(|&&t| t == Tile::OreMine).count();
            player.resources.coal += mines * 2;
        }
        if self == ActionSpace::RubyMining {
            if player.indoor.iter().flatten().any(|&t| t == Tile::RubyMine) {
                player.resources.rubies += 1;
            }
        }
        if self == ActionSpace::RubyDelivery {
            let mines = player.indoor.iter().flatten().filter(|&&t| t == Tile::RubyMine).count();
            if mines >= 2 { player.resources.rubies += 1; }
        }
        if self == ActionSpace::Housework { player.dogs += 1; }
    }

    // Returns (state, sub_stack) candidates after the dwarf has been placed and gains applied.
    // stack vec: last element = current sub-action (first to execute).
    fn sub_actions(self, base: State, current: usize) -> Vec<(State, Vec<SubAction>)> {
        match self {
            ActionSpace::Excavation => vec![
                (base.clone(), vec![SubAction::PlaceTile(TileGroup::Twin((Tile::Tunnel, Tile::Cave)))]),
                (base.clone(), vec![SubAction::PlaceTile(TileGroup::Twin((Tile::Cave, Tile::Cave)))]),
                (base, vec![]),
            ],
            ActionSpace::Blacksmithing =>
                base.forge_options(current, self).into_iter()
                    .map(|c| (c, vec![SubAction::ExpeditionPick { space: self, picks_remaining: 3, used_items: 0 }]))
                    .collect(),
            ActionSpace::Adventure => {
                let exp_stack = vec![
                    SubAction::ExpeditionPick { space: self, picks_remaining: 1, used_items: 0 },
                    SubAction::ExpeditionPick { space: self, picks_remaining: 1, used_items: 0 },
                ];
                let armed = base.players[current].dwarfs.iter()
                    .find(|d| d.placed_on == Some(self))
                    .map_or(false, |d| d.weapon > 0);
                if armed {
                    vec![(base, exp_stack)]
                } else {
                    let mut opts = vec![(base.clone(), exp_stack.clone())];
                    opts.extend(base.forge_options(current, self).into_iter().map(|c| (c, exp_stack.clone())));
                    opts
                }
            }
            ActionSpace::WishForChildren => {
                let mut opts = vec![];
                if let Some(grown) = base.family_growth_option(current) {
                    opts.push((grown, vec![]));
                }
                opts.push((base, vec![SubAction::Furnish]));
                opts
            }
            ActionSpace::FamilyLife => {
                let mut opts = vec![];
                if let Some(grown) = base.family_growth_option(current) {
                    opts.push((grown.clone(), vec![]));
                    opts.push((grown, vec![SubAction::Sow]));
                }
                opts.push((base.clone(), vec![SubAction::Sow]));
                opts.push((base, vec![]));
                opts
            }
            ActionSpace::SheepFarming | ActionSpace::DonkeyFarming =>
                vec![(base, vec![SubAction::Stable, SubAction::Pasture])],
            ActionSpace::Housework => vec![
                (base.clone(), vec![SubAction::Furnish]),
                (base, vec![]),
            ],
            ActionSpace::Logging =>
                vec![(base, vec![SubAction::ExpeditionPick { space: self, picks_remaining: 1, used_items: 0 }])],
            ActionSpace::OreMineConstruction => {
                let tile = TileGroup::Twin((Tile::DeepTunnel, Tile::OreMine));
                let exp = SubAction::ExpeditionPick { space: self, picks_remaining: 1, used_items: 0 };
                vec![
                    (base.clone(), vec![exp.clone(), SubAction::PlaceTile(tile)]),
                    (base, vec![exp]),
                ]
            }
            ActionSpace::SlashAndBurn => {
                let tile = TileGroup::Twin((Tile::Meadow, Tile::Field((0, 0))));
                vec![
                    (base.clone(), vec![SubAction::Sow, SubAction::PlaceTile(tile.clone())]),
                    (base.clone(), vec![SubAction::PlaceTile(tile)]),
                    (base, vec![SubAction::Sow]),
                ]
            }
            ActionSpace::DriftMining => vec![
                (base.clone(), vec![SubAction::PlaceTile(TileGroup::Twin((Tile::Tunnel, Tile::Cave)))]),
                (base, vec![]),
            ],
            ActionSpace::Clearing | ActionSpace::Sustenance => {
                let tile = TileGroup::Twin((Tile::Meadow, Tile::Field((0, 0))));
                vec![
                    (base.clone(), vec![SubAction::PlaceTile(tile)]),
                    (base, vec![]),
                ]
            }
            ActionSpace::RubyMineConstruction => vec![
                (base.clone(), vec![SubAction::PlaceTile(TileGroup::Single(Tile::RubyMine))]),
                (base, vec![]),
            ],
            _ => vec![(base, vec![])],
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

#[derive(Clone)]
pub(crate) enum TileGroup {
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

    // Places the dwarf with lowest weapon on the given action space.
    pub(crate) fn place_dwarf(&mut self, space: ActionSpace) {
        self.dwarfs.iter_mut()
            .filter(|d| d.placed_on.is_none())
            .min_by_key(|d| d.weapon)
            .expect("no unplaced dwarf")
            .placed_on = Some(space);
    }

}

#[derive(Clone)]
pub(crate) enum SubAction {
    SelectActionSpace,
    PlaceTile(TileGroup),
    Pasture,
    Stable,
    Furnish,
    Sow,
    ExpeditionPick { space: ActionSpace, picks_remaining: usize, used_items: u16 },
    TradeFood { player_idx: usize },
    FinishRound,
}

#[derive(Clone)]
pub struct State {
    pub players: Vec<Player>,
    pub round: u32,
    pub starting_player: u8,
    pub accumulated: [u32; ActionSpace::COUNT],
    pub(crate) current_player: usize,
    pub(crate) pending: Vec<SubAction>,
}
impl State {
    pub fn new(count: u32) -> Self {
        let mut players = Vec::new();
        for _ in 0..count {
            players.push(Player::new(2));
        }
        State { players, round: 0, starting_player: 0, accumulated: [0u32; ActionSpace::COUNT], current_player: 0, pending: vec![SubAction::SelectActionSpace] }
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

    fn advance_round(mut self) -> Self {
        self.replenish();
        self.return_dwarfs();
        for player in &mut self.players {
            player.harvest();
        }
        // Trading phase: each player may trade resources for food, then FinishRound feeds and breeds.
        let n = self.players.len();
        let mut pending = vec![SubAction::FinishRound];
        for i in (0..n).rev() {
            pending.push(SubAction::TradeFood { player_idx: i });
        }
        self.pending = pending;
        self.current_player = 0;
        self
    }

    fn is_round_over(&self) -> bool {
        self.players[self.current_player].dwarfs.iter().all(|d| d.placed_on.is_some())
    }

    fn pop_subaction(mut self) -> Self {
        self.pending.pop();
        if self.pending.is_empty() {
            self.current_player = self.next_placement_player()
                .unwrap_or(self.starting_player as usize);
            self.pending = vec![SubAction::SelectActionSpace];
        }
        self
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

    // Returns (new_state, item_bit, needs_furnish_subaction)
    fn expedition_options(&self, player_idx: usize, weapon: u8, used_items: u16) -> Vec<(Self, u16, bool)> {
        if weapon == 0 {
            return vec![(self.clone(), 0, false)];
        }
        // Item indices: 0 reserved (future lvl-1 item), then pairs per level:
        //   lvl 1 → 1,2 | lvl 2 → 3,4 | lvl 3 → 5,6 | lvl 4 → 7,8
        //   lvl 5 → 9,10 | lvl 6 → 11,12 | lvl 7 → 13,14
        // min_weapon(i) = ((i+1)/2).max(1)
        let min_weapon = |i: usize| -> u8 { (((i + 1) / 2).max(1)) as u8 };
        let avail = |i: usize| weapon >= min_weapon(i) && used_items & (1 << i) == 0;
        let mut results = vec![];
        // Level 1 (indices 1, 2)
        if avail(1) { let mut c = self.clone(); c.players[player_idx].resources.wood += 1; results.push((c, 1u16 << 1, false)); }
        if avail(2) { let mut c = self.clone(); c.players[player_idx].dogs += 1; results.push((c, 1u16 << 2, false)); }
        // Level 2 (indices 3, 4)
        if avail(3) { let mut c = self.clone(); c.players[player_idx].resources.wheat += 1; results.push((c, 1u16 << 3, false)); }
        if avail(4) { let mut c = self.clone(); c.players[player_idx].animals[AnimalType::Sheep as usize] += 1; results.push((c, 1u16 << 4, false)); }
        // Level 3 (indices 5, 6)
        if avail(5) { let mut c = self.clone(); c.players[player_idx].resources.stone += 1; results.push((c, 1u16 << 5, false)); }
        if avail(6) { let mut c = self.clone(); c.players[player_idx].animals[AnimalType::Donkey as usize] += 1; results.push((c, 1u16 << 6, false)); }
        // Level 4 (indices 7, 8)
        if avail(7) { let mut c = self.clone(); c.players[player_idx].resources.vegetables += 1; results.push((c, 1u16 << 7, false)); }
        if avail(8) { let mut c = self.clone(); c.players[player_idx].resources.coal += 2; results.push((c, 1u16 << 8, false)); }
        // Level 5 (indices 9, 10)
        if avail(9) { let mut c = self.clone(); c.players[player_idx].animals[AnimalType::Boar as usize] += 1; results.push((c, 1u16 << 9, false)); }
        // Level 6 (indices 11, 12)
        if avail(11) { let mut c = self.clone(); c.players[player_idx].resources.gold += 2; results.push((c, 1u16 << 11, false)); }
        // Level 7 (index 13): push Furnish sub-action rather than inlining
        if avail(13) && !self.furnish_cave_options(player_idx).is_empty() {
            results.push((self.clone(), 1u16 << 13, true));
        }
        // Weapon > 0 but all reachable items already picked — pass through with no reward
        if results.is_empty() { results.push((self.clone(), 0, false)); }
        results
    }

    fn furnish_cave_options(&self, player_idx: usize) -> Vec<Self> {
        let p = &self.players[player_idx];
        let mut opts = vec![];
        for y in 0..BOARD_HEIGHT {
            for x in 0..HALF_WIDTH {
                if p.indoor[y][x] != Tile::Cave { continue; }
                for f in FURNISHINGS {
                    if p.resources.wood < f.cost_wood || p.resources.stone < f.cost_stone { continue; }
                    let used = self.players.iter()
                        .flat_map(|pl| pl.indoor.iter().flatten())
                        .filter(|&&t| t == f.tile)
                        .count();
                    if used >= f.max_count { continue; }
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

    // Lightweight feasibility check: does the current pending sub-action have any valid moves?
    // Used to filter dead-end and duplicate candidates without full children() enumeration.
    pub(crate) fn has_children(&self) -> bool {
        let current = self.current_player;
        match self.pending.last() {
            Some(SubAction::PlaceTile(tile)) =>
                !self.players[current].tile_placements(tile.clone()).is_empty(),
            Some(SubAction::Sow) => {
                let p = &self.players[current];
                let has_field = p.outdoor.iter().flatten().any(|&t| t == Tile::Field((0, 0)));
                has_field && (p.resources.wheat > 0 || p.resources.vegetables > 0)
            }
            Some(SubAction::Furnish) =>
                !self.furnish_cave_options(current).is_empty(),
            _ => true,
        }
    }
}

impl GameState for State {
    fn current_player(&self) -> usize { self.current_player }
    fn num_players(&self) -> usize { self.players.len() }

    fn children<R: rand::prelude::Rng>(&self, _rng: &mut R) -> Vec<Self> {
        if self.done() { return vec![]; }
        let current = self.current_player;

        match self.pending.last().expect("pending stack is empty") {
            SubAction::SelectActionSpace => {
                // Current player has no dwarfs to place — finish the round now.
                // Deferred from pop_subaction() so this round-boundary state is a distinct node,
                // separating action gains from end-of-round effects like harvest and begging.
                if self.is_round_over() {
                    return vec![self.clone().advance_round()];
                }

                let mut children = vec![];
                for &space in &ActionSpace::ALL {
                    if !self.space_available(space) { continue; }
                    let occupied = self.players.iter()
                        .any(|p| p.dwarfs.iter().any(|d| d.placed_on == Some(space)));
                    if occupied { continue; }

                    let mut base = self.clone();
                    base.players[current].place_dwarf(space);
                    space.apply_gains(base.accumulated[space as usize], &mut base.players[current]);
                    if space == ActionSpace::StartingPlayer { base.starting_player = current as u8; }

                    let mut candidates = space.sub_actions(base, current);

                    // Filter candidates whose first sub-action has no valid moves
                    candidates.retain(|(c, stack)| {
                        if stack.is_empty() { return true; }
                        let mut test = c.clone();
                        test.pending = stack.clone();
                        test.has_children()
                    });

                    // Tile placement is technically optional, but if any candidate includes one
                    // we trim those that don't — pruning rarely-optimal moves for a shallower tree.
                    let has_tile = candidates.iter().any(|(_, s)| s.iter().any(|a| matches!(a, SubAction::PlaceTile(_))));
                    if has_tile { candidates.retain(|(_, s)| s.iter().any(|a| matches!(a, SubAction::PlaceTile(_)))); }

                    for (mut c, sub_stack) in candidates {
                        if sub_stack.is_empty() {
                            c = c.pop_subaction();
                        } else {
                            c.pending = sub_stack;
                        }
                        children.push(c);
                    }
                }
                children
            }

            SubAction::PlaceTile(tile) => {
                let tile = tile.clone();
                let placements = self.players[current].tile_placements(tile.clone());
                if placements.is_empty() {
                    return vec![self.clone().pop_subaction()];
                }
                placements.into_iter().map(|(side, new_board)| {
                    let mut c = self.clone();
                    let old_board = match side { Side::Outdoor => c.players[current].outdoor, Side::Indoor => c.players[current].indoor };
                    let replaced = board_delta(&old_board, &new_board);
                    let changed = changed_cells(&old_board, &new_board);
                    // Placement bonuses derived from what was placed and what it replaced
                    let placed: Vec<Tile> = changed.iter().map(|&(x, y)| new_board[y][x]).collect();
                    if placed.iter().any(|&t| t == Tile::OreMine) {
                        c.players[current].resources.coal += 3;
                    }
                    if placed.iter().any(|&t| t == Tile::RubyMine) && matches!(replaced, TileGroup::Single(Tile::DeepTunnel)) {
                        c.players[current].resources.rubies += 1;
                    }
                    for pos in changed {
                        let p = &mut c.players[current];
                        match side {
                            Side::Outdoor => apply_outdoor_location_bonus(pos, &mut p.resources, &mut p.animals),
                            Side::Indoor  => apply_indoor_location_bonus(pos, &mut p.resources, &mut p.animals),
                        }
                    }
                    match side { Side::Outdoor => c.players[current].outdoor = new_board, Side::Indoor => c.players[current].indoor = new_board }
                    c.pop_subaction()
                }).collect()
            }

            SubAction::Pasture =>
                self.pasture_options(current).into_iter().map(|c| c.pop_subaction()).collect(),

            SubAction::Stable =>
                self.stable_options(current).into_iter().map(|c| c.pop_subaction()).collect(),

            SubAction::Furnish => {
                let opts = self.furnish_cave_options(current);
                if opts.is_empty() { vec![self.clone().pop_subaction()] }
                else { opts.into_iter().map(|c| c.pop_subaction()).collect() }
            }

            SubAction::Sow => {
                let opts = self.sow_options(current);
                if opts.is_empty() { vec![self.clone().pop_subaction()] }
                else { opts.into_iter().map(|c| c.pop_subaction()).collect() }
            }

            &SubAction::ExpeditionPick { space, picks_remaining, used_items } => {
                let weapon = self.players[current].dwarfs.iter()
                    .find(|d| d.placed_on == Some(space))
                    .map(|d| d.weapon).unwrap_or(0);
                self.expedition_options(current, weapon, used_items)
                    .into_iter()
                    .map(|(state, item_bit, needs_furnish)| {
                        let mut c = state;
                        c.pending.pop();
                        if picks_remaining > 1 {
                            c.pending.push(SubAction::ExpeditionPick { space, picks_remaining: picks_remaining - 1, used_items: used_items | item_bit });
                        } else {
                            let has_more = c.pending.iter().any(|a| matches!(a, SubAction::ExpeditionPick { space: s2, .. } if *s2 == space));
                            if !has_more {
                                if let Some(d) = c.players[current].dwarfs.iter_mut().find(|d| d.placed_on == Some(space)) {
                                    if d.weapon > 0 { d.weapon = d.weapon.saturating_add(1); }
                                }
                            }
                        }
                        if needs_furnish && !c.furnish_cave_options(current).is_empty() {
                            c.pending.push(SubAction::Furnish);
                        }
                        if c.pending.is_empty() { return c.pop_subaction(); }
                        c
                    })
                    .collect()
            }

            SubAction::TradeFood { player_idx } => {
                let player_idx = *player_idx;
                let food_needed = self.players[player_idx].food_needed();
                let food_have = self.players[player_idx].resources.food;
                let food_gap = food_needed.saturating_sub(food_have);

                let sheep  = self.players[player_idx].animals[AnimalType::Sheep  as usize];
                let boar   = self.players[player_idx].animals[AnimalType::Boar   as usize];
                let cow    = self.players[player_idx].animals[AnimalType::Cow    as usize];
                let donkey = self.players[player_idx].animals[AnimalType::Donkey as usize];
                let wheat  = self.players[player_idx].resources.wheat;
                let veg    = self.players[player_idx].resources.vegetables;
                let rubies = self.players[player_idx].resources.rubies;

                let mut children = vec![];

                // "Pass" — done trading; advance to next trader or FinishRound
                let mut done = self.clone();
                done.pending.pop();
                if let Some(SubAction::TradeFood { player_idx: next }) = done.pending.last() {
                    done.current_player = *next;
                }
                children.push(done);

                if food_gap > 0 {
                    // 1 sheep → 1 food
                    for n in 1..=sheep.min(food_gap) {
                        let mut c = self.clone();
                        c.players[player_idx].animals[AnimalType::Sheep as usize] -= n;
                        c.players[player_idx].resources.food += n;
                        children.push(c);
                    }
                    // 1 boar → 2 food
                    for n in 1..=boar.min((food_gap + 1) / 2) {
                        let mut c = self.clone();
                        c.players[player_idx].animals[AnimalType::Boar as usize] -= n;
                        c.players[player_idx].resources.food += n * 2;
                        children.push(c);
                    }
                    // 1 cow → 3 food
                    for n in 1..=cow.min((food_gap + 2) / 3) {
                        let mut c = self.clone();
                        c.players[player_idx].animals[AnimalType::Cow as usize] -= n;
                        c.players[player_idx].resources.food += n * 3;
                        children.push(c);
                    }
                    // 1 donkey → 1 food (only when a pair trade isn't possible)
                    if donkey == 1 {
                        let mut c = self.clone();
                        c.players[player_idx].animals[AnimalType::Donkey as usize] -= 1;
                        c.players[player_idx].resources.food += 1;
                        children.push(c);
                    }
                    // 2 donkeys → 3 food
                    for n in 1..=(donkey / 2).min((food_gap + 2) / 3) {
                        let mut c = self.clone();
                        c.players[player_idx].animals[AnimalType::Donkey as usize] -= n * 2;
                        c.players[player_idx].resources.food += n * 3;
                        children.push(c);
                    }
                    // 1 wheat → 1 food
                    for n in 1..=wheat.min(food_gap) {
                        let mut c = self.clone();
                        c.players[player_idx].resources.wheat -= n;
                        c.players[player_idx].resources.food += n;
                        children.push(c);
                    }
                    // 1 vegetable → 2 food
                    for n in 1..=veg.min((food_gap + 1) / 2) {
                        let mut c = self.clone();
                        c.players[player_idx].resources.vegetables -= n;
                        c.players[player_idx].resources.food += n * 2;
                        children.push(c);
                    }
                    // 1 ruby → 2 food
                    for n in 1..=rubies.min((food_gap + 1) / 2) {
                        let mut c = self.clone();
                        c.players[player_idx].resources.rubies -= n;
                        c.players[player_idx].resources.food += n * 2;
                        children.push(c);
                    }
                }

                children
            }

            SubAction::FinishRound => {
                let mut c = self.clone();
                c.pending.pop();
                for player in &mut c.players {
                    player.feed();
                    player.breed();
                }
                c.grow_children();
                c.round += 1;
                c.current_player = c.starting_player as usize;
                c.pending = vec![SubAction::SelectActionSpace];
                vec![c]
            }
        }
    }

    fn winner(&self) -> Option<usize> {
        if !self.done() { return None; }
        self.players.iter().enumerate().max_by_key(|(_, p)| p.points()).map(|(i, _)| i)
    }
}
