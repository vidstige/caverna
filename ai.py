from typing import List, Tuple
import random
from caverna import *


inf = 2 ** 64


def evaluate(game: Game, state: Game.State, player: Player):
    #return sum(v for k, v in state.player_states[player].resources.items() if k in ('food', 'wheat', 'sheep', 'donkey', 'ruby'))
    #return game.score(state, player)
    ps = state.player_states[player]
    w = defaultdict(int, food=4, ruby=2, sheep=1, donkey=1, wheat=2)
    return sum(r.get('wheat', 0) for r in ps.field_resources.values()) * 4 \
        + sum(w[k] * v for k, v in ps.resources.items())


def minmax(game: Game, state: Game.State, player: Player, d: int, alpha: int, beta: int) -> Tuple[Action, int]:
    def heuristic(action: Action):
        return 1
        #return -sum(state.action_resources.get(action, {}).values())

    if d <= 0:
        return None, evaluate(game, state, player), 1

    if game.over(state):
        return None, game.score(state, player), 1

    if not game.work_phase(state):
        game.return_dwarfs(state)
        game.harvest(state)

    maximizing = game.current_player(state) == player 
    f = max if maximizing else min
    evaluations = {}
    nn = 0
    actions = available_actions(game, state)
    for action in sorted(actions, key=heuristic):
        a, e, n = minmax(game, game.take(action, state), player, d-1, alpha, beta)
        if maximizing:
            alpha = f(alpha, e)
        else:
            beta = f(beta, e)
        evaluations[action] = e
        nn += n
        if beta <= alpha:
            break

    best = f(evaluations, key=evaluations.get)
    return best, evaluations[best], nn


class MinMax(Controller):
    total = 0
    def select_action(self, game, state):
        depth = 4
        action, _, n = minmax(game, state, self.player, depth, -inf, inf)
        print(n)
        self.total += n
        return action


class Random(Controller):
    def select_action(self, game, state):
        return random.choice(available_actions(game, state))
