from typing import List, Tuple
import random
from caverna import *


inf = 2 ** 64


def minmax(game: Game, state: Game.State, player: Player, d: int, alpha: int, beta: int) -> Tuple[Action, int]:
    def heuristic(action: Action):
        return -sum(state.action_resources.get(action, {}).values())

    if d <= 0:
        return None, game.score(state, player), 1

    if not game.round(state):
        game.return_dwarfs(state)
        game.harvest(state)
        state.round += 1

    if game.over(state):
        return None, game.score(state, player), 1

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
