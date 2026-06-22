use crate::mcts::GameState;

// States
enum Terrain {
    Forest,
    Meadow,
    Soil,
}

struct OutdoorTile {
    terrain: Terrain,
    fence: bool,
    house: bool,
}

enum Cave { Mountain, Mine, Excavated }

struct CaveTile {
    cave: Cave,
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
    dwarfs: Vec<u8>,  // weapon level

    resources: Resources,
    animals: Animals,
}
impl Player {
    fn new(food: usize) -> Self {
        let mut player = Player{
            dwarfs: vec![0, 0],
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
    round: u32,
    turn: u32,
    pub players: Vec<Player>,
}
impl State {
    pub fn new(count: u32) -> Self {
        let mut players = Vec::new();
        for _ in 0..count {
            players.push(Player::new(2));
        }
        State{round: 0, turn: 0, players: players}
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