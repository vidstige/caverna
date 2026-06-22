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
}
impl ActionSpace {
    const COUNT: usize = 6;
    const ALL: [ActionSpace; Self::COUNT] = [
        ActionSpace::Logging,
        ActionSpace::WoodGathering,
        ActionSpace::Supplies,
        ActionSpace::StartingPlayer,
        ActionSpace::Clearing,
        ActionSpace::Sustenance,
    ];

    fn initial(self) -> Resources {
        match self {
            ActionSpace::Logging        => Resources { wood: 1, ..Resources::zero() },
            ActionSpace::WoodGathering  => Resources { wood: 1, ..Resources::zero() },
            ActionSpace::Supplies       => Resources { wood: 1, stone: 1, food: 2, ..Resources::zero() },
            ActionSpace::StartingPlayer => Resources::zero(),
            ActionSpace::Clearing       => Resources { wood: 1, ..Resources::zero() },
            ActionSpace::Sustenance     => Resources { food: 1, ..Resources::zero() },
        }
    }
    fn per_round(self) -> Resources {
        match self {
            ActionSpace::Logging        => Resources { wood: 1, ..Resources::zero() },
            ActionSpace::WoodGathering  => Resources { wood: 1, ..Resources::zero() },
            ActionSpace::Supplies       => Resources::zero(),
            ActionSpace::StartingPlayer => Resources::zero(),
            ActionSpace::Clearing       => Resources { wood: 1, ..Resources::zero() },
            ActionSpace::Sustenance     => Resources { food: 1, ..Resources::zero() },
        }
    }
    fn place_tile(self) -> Option<TileToPlace> {
        match self {
            ActionSpace::Clearing   => Some(TileToPlace::Twin((Tile::Meadow, Tile::Field))),
            ActionSpace::Sustenance => Some(TileToPlace::Twin((Tile::Meadow, Tile::Field))),
            _ => None,
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
    Meadow,
    Field,
    // Indoor
    Mountain,
    Cavern,
    Dwelling,
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
    fn is_zero(&self) -> bool {
        self.points == 0 && self.begging == 0 && self.wood == 0 && self.stone == 0
            && self.coal == 0 && self.rubies == 0 && self.food == 0
            && self.wheat == 0 && self.vegetables == 0
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

#[derive(Clone)]
struct Animals {
    dogs: usize,
    sheep: usize,
    boars: usize,
    donkeys: usize,
    cows: usize,
}
impl Animals {
    fn zero() -> Animals {
        Animals { dogs: 0, sheep: 0, boars: 0, donkeys: 0, cows: 0, }
    }
}

#[derive(Clone)]
pub struct Player {
    pub dwarfs: Vec<Dwarf>,
    tiles: [[Tile; BOARD_WIDTH]; BOARD_HEIGHT],
    resources: Resources,
    animals: Animals,
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
        tiles[2][3] = Tile::Cavern;
        let mut player = Player {
            dwarfs: vec![Dwarf { weapon: 0, placed_on: None }, Dwarf { weapon: 0, placed_on: None }],
            tiles,
            resources: Resources::zero(),
            animals: Animals::zero(),
        };
        player.resources.food = food;
        player
    }

    fn adjacent_to_placed(&self, x: usize, y: usize) -> bool {
        adjacents(x, y).any(|(nx, ny)| matches!(self.tiles[ny][nx], Tile::Meadow | Tile::Field))
    }

    fn tile_placements(&self, tile: TileToPlace) -> Vec<[[Tile; BOARD_WIDTH]; BOARD_HEIGHT]> {
        let has_placed = self.tiles.iter().flatten()
            .any(|&t| matches!(t, Tile::Meadow | Tile::Field));
        let mut result = vec![];
        match tile {
            TileToPlace::Single(t) => {
                for y in 0..BOARD_HEIGHT {
                    for x in 0..BOARD_WIDTH {
                        if self.tiles[y][x] != Tile::Forest { continue; }
                        let valid = if !has_placed { x == 2 && y == 3 }
                                    else { self.adjacent_to_placed(x, y) };
                        if valid {
                            let mut board = self.tiles;
                            board[y][x] = t;
                            result.push(board);
                        }
                    }
                }
            }
            TileToPlace::Twin((t1, t2)) => {
                for y in 0..BOARD_HEIGHT {
                    for x in 0..BOARD_WIDTH {
                        for (x2, y2) in [(x + 1, y), (x, y + 1)] {
                            if x2 >= BOARD_WIDTH || y2 >= BOARD_HEIGHT { continue; }
                            if self.tiles[y][x] != Tile::Forest || self.tiles[y2][x2] != Tile::Forest { continue; }
                            let valid = if !has_placed {
                                (x == 2 && y == 3) || (x2 == 2 && y2 == 3)
                            } else {
                                self.adjacent_to_placed(x, y) || self.adjacent_to_placed(x2, y2)
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
        self.animals.dogs as i32 +
        self.animals.sheep as i32 +
        self.animals.boars as i32 +
        self.animals.donkeys as i32 +
        self.animals.cows as i32 -
        self.resources.begging as i32 * 3
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
}

#[derive(Clone)]
pub struct State {
    pub players: Vec<Player>,
    pub round: u32,
    pub starting_player: u8,
    pub accumulated: [Resources; ActionSpace::COUNT],
}
impl State {
    pub fn new(count: u32) -> Self {
        let mut players = Vec::new();
        for _ in 0..count {
            players.push(Player::new(2));
        }
        let mut accumulated = [Resources::zero(); ActionSpace::COUNT];
        for &space in &ActionSpace::ALL {
            accumulated[space as usize] = space.initial();
        }
        State { players, round: 0, starting_player: 0, accumulated }
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
    fn feeding(&mut self) {
        for player in &mut self.players {
            player.feed();
        }
    }
    fn replenish(&mut self) {
        for &space in &ActionSpace::ALL {
            let slot = &mut self.accumulated[space as usize];
            if slot.is_zero() {
                *slot += space.initial();
            } else {
                *slot += space.per_round();
            }
        }
    }
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
            child.players[current].resources += child.accumulated[space as usize];
            child.accumulated[space as usize] = Resources::zero();

            let mut candidates = if let Some(tile) = space.place_tile() {
                let boards = child.players[current].tile_placements(tile);
                if boards.is_empty() {
                    vec![child]
                } else {
                    boards.into_iter().map(|board| {
                        let mut c = child.clone();
                        c.players[current].tiles = board;
                        c
                    }).collect()
                }
            } else {
                vec![child]
            };

            let all_placed = candidates[0].players.iter()
                .all(|p| p.dwarfs.iter().all(|d| d.placed_on.is_some()));
            if all_placed {
                for c in &mut candidates {
                    c.return_dwarfs();
                    c.feeding();
                    c.round += 1;
                    c.replenish();
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