use rand::{Rng, seq::{IndexedRandom}};

pub trait GameState: Sized + Clone {
    fn current_player(&self) -> usize;
    fn num_players(&self) -> usize;
    fn children<R: Rng>(&self, rng: &mut R) -> Vec<Self>;
    fn winner(&self) -> Option<usize>;
}

// sqrt(2) — balances exploration vs exploitation
const C: f64 = 1.414;

struct Node<S> {
    state: S,
    visits: u32,
    scores: Vec<f64>,
    children: Vec<Node<S>>,
}

impl<S: GameState> Node<S> {
    fn new(state: S, num_players: usize) -> Self {
        Node {
            state,
            visits: 0,
            scores: vec![0.0; num_players],
            children: Vec::new(),
        }
    }

    fn ucb1(&self, parent_visits: u32, player: usize) -> f64 {
        if self.visits == 0 {
            return f64::INFINITY;
        }
        self.scores[player] / self.visits as f64
            + C * ((parent_visits as f64).ln() / self.visits as f64).sqrt()
    }

    fn best_ucb_child_index(&self) -> usize {
        let player = self.state.current_player();
        let visits = self.visits;
        self.children
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.ucb1(visits, player)
                    .partial_cmp(&b.ucb1(visits, player))
                    .unwrap()
            })
            .map(|(i, _)| i)
            .unwrap()
    }

    fn rollout<R: Rng>(&self, rng: &mut R) -> Vec<f64> {
        let n = self.scores.len();
        let mut state = self.state.clone();
        loop {
            if let Some(winner) = state.winner() {
                let mut result = vec![0.0; n];
                result[winner] = 1.0;
                return result;
            }
            let children = state.children(rng);
            state = children.choose(rng).unwrap().clone();
        }
    }

    fn iterate<R: Rng>(&mut self, rng: &mut R) -> Vec<f64> {
        let n = self.scores.len();
        let scores = if let Some(winner) = self.state.winner() {
            let mut s = vec![0.0; n];
            s[winner] = 1.0;
            s
        } else if self.visits == 0 {
            // unvisited leaf: rollout
            self.rollout(rng)
        } else {
            if self.children.is_empty() {
                // expand all children on second visit
                let states = self.state.children(rng);
                self.children = states.into_iter().map(|s| Node::new(s, n)).collect();
            }
            let idx = self.best_ucb_child_index();
            self.children[idx].iterate(rng)
        };
        self.visits += 1;
        for (i, &s) in scores.iter().enumerate() {
            self.scores[i] += s;
        }
        scores
    }
}

pub fn search<S: GameState, R: Rng>(state: &S, rng: &mut R, iterations: usize) -> Option<S> {
    let n = state.num_players();
    let mut root = Node::new(state.clone(), n);
    for _ in 0..iterations {
        root.iterate(rng);
    }
    if root.children.is_empty() {
        return None;
    }
    // pick the most-visited child — most robust under MCTS
    let best = root
        .children
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| c.visits)
        .map(|(i, _)| i)
        .unwrap();
    Some(root.children.remove(best).state)
}

pub fn random_move<S: GameState, R: Rng>(state: &S, rng: &mut R) -> S {
    let children = state.children(rng);
    children.choose(rng).unwrap().clone()
}
