use crate::mcts::GameState;

#[derive(Clone, Copy, PartialEq)]
pub enum ActionSpace {
    Logging,
    WoodGathering,
    Supplies,
    StartingPlayer,
}

#[derive(Clone)]
pub struct Dwarf {
    pub weapon: u8,
    pub placed_on: Option<ActionSpace>,
}

#[derive(Clone)]
struct Resources {
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
        Resources {
            points: 0,
            begging: 0,
            wood: 0,
            stone: 0,
            coal: 0,
            rubies: 0,
            food: 0,
            wheat: 0,
            vegetables: 0,
        }
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
}
impl State {
    pub fn new(count: u32) -> Self {
        let mut players = Vec::new();
        for _ in 0..count {
            players.push(Player::new(2));
        }
        State { players, round: 0, starting_player: 0 }
    }
    fn rounds(self) -> u32 {
        // for two players
        12
    }
    fn done(self) -> bool {
        self.round >= self.rounds() - 1
    }
}

impl GameState for State {
    fn current_player(&self) -> usize {
        0
    }

    fn num_players(&self) -> usize {
        2
    }

    fn children<R: rand::prelude::Rng>(&self, rng: &mut R) -> Vec<Self> {
        todo!()
    }

    fn winner(&self) -> Option<usize> {
        todo!()
    }
}