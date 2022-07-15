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

struct Player {
    dwarfs: Vec<u8>,  // raid level

    // resources
    points: usize,
    begging: usize,

    wood: usize,
    stone: usize,
    coal: usize,
    rubies: usize,
    food: usize,
    
    wheat: usize,
    vegetables: usize,

    dogs: usize,
    sheep: usize,
    boars: usize,
    donkeys: usize,
    cows: usize,
}
impl Player {
    fn new(food: usize) -> Self {
        Player{
            dwarfs: vec![0, 0],
            points: 0, begging: 0, wood: 0, stone: 0, coal: 0, rubies: 0, food: food,
            wheat: 0, vegetables: 0, dogs: 0, sheep: 0, boars: 0, donkeys: 0, cows: 0,
        }
    }
    fn points(&self) -> usize {
        self.dwarfs.len() +
        self.points +
        self.rubies + 
        self.wheat / 2 + 
        self.vegetables +
        self.dogs + 
        self.sheep + 
        self.boars + 
        self.donkeys +
        self.cows
    }
}

struct State {
    round: u32,
    turn: u32,
    players: Vec<Player>,
}
impl State {
    fn new(count: u32) -> Self {
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

fn main() {
    let state = State::new(2);
    for player in state.players.iter() {
        println!("points: {}", player.points());
    }
}
