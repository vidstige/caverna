use crate::mcts::GameState;

#[derive(Clone, Copy, PartialEq)]
pub enum ActionSpace {
    Logging,
    WoodGathering,
    Supplies,
    StartingPlayer,
}
impl ActionSpace {
    const COUNT: usize = 4;

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
    pub fn points(&self) -> usize {
        self.dwarfs.len() +
        self.resources.points +
        self.resources.rubies + 
        self.resources.wheat / 2 + 
        self.resources.vegetables +
        self.animals.dogs + 
        self.animals.sheep + 
        self.animals.boars + 
        self.animals.donkeys +
        self.animals.cows
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
        State { players, round: 0, starting_player: 0, accumulated: [Resources::zero(); ActionSpace::COUNT] }
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
        for _ in 0..=(total_dwarves.iter().sum::<usize>()) {
            if placed[seat] < total_dwarves[seat] {
                if turns == total_placed {
                    return seat;
                }
                turns += 1;
            }
            seat = (seat + 1) % n;
        }
        panic!("current_player called when all dwarves are placed");
    }

    fn num_players(&self) -> usize {
        2
    }

    fn children<R: rand::prelude::Rng>(&self, rng: &mut R) -> Vec<Self> {
        todo!()
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