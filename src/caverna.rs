use crate::mcts::GameState;

#[derive(Clone, Copy, PartialEq)]
#[repr(usize)]
pub enum ActionSpace {
    Logging = 0,
    WoodGathering = 1,
    Supplies = 2,
    StartingPlayer = 3,
}
impl ActionSpace {
    const COUNT: usize = 4;
    const ALL: [ActionSpace; Self::COUNT] = [
        ActionSpace::Logging,
        ActionSpace::WoodGathering,
        ActionSpace::Supplies,
        ActionSpace::StartingPlayer,
    ];

    fn initial(self) -> Resources {
        match self {
            ActionSpace::Logging       => Resources { wood: 1, ..Resources::zero() },
            ActionSpace::WoodGathering => Resources { wood: 1, ..Resources::zero() },
            ActionSpace::Supplies      => Resources { wood: 1, stone: 1, food: 2, ..Resources::zero() },
            ActionSpace::StartingPlayer => Resources::zero(),
        }
    }
    fn per_round(self) -> Resources {
        match self {
            ActionSpace::Logging       => Resources { wood: 1, ..Resources::zero() },
            ActionSpace::WoodGathering => Resources { wood: 1, ..Resources::zero() },
            ActionSpace::Supplies      => Resources::zero(),
            ActionSpace::StartingPlayer => Resources::zero(),
        }
    }
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

    resources: Resources,
    animals: Animals,
}
impl Player {
    fn new(food: usize) -> Self {
        let mut player = Player{
            dwarfs: vec![Dwarf { weapon: 0, placed_on: None }, Dwarf { weapon: 0, placed_on: None }],
            resources: Resources::zero(),
            animals: Animals::zero(),
        };
        player.resources.food = food;
        player
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

        let all_placed = self.players.iter()
            .all(|p| p.dwarfs.iter().all(|d| d.placed_on.is_some()));

        if all_placed {
            let mut next = self.clone();
            next.return_dwarfs();
            next.feeding();
            next.round += 1;
            next.replenish();
            return vec![next];
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
            children.push(child);
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