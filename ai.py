from typing import List, Tuple
import random
from caverna import *


def minmax(game: Game, state: Game.State, player: Player) -> Tuple[Action, int]:
    if not game.round(state):
        game.return_dwarfs(state)
        game.harvest(state)
        state.round += 1

    if game.over(state):
        return None, game.score(state, player), 1

    f = max if game.current_player(state) == player else min
    #v = -1000000 if game.current_player(state) == player else 1000000
    evaluations = {}
    nn = 0
    for action in available_actions(game, state):
        a, e, n = minmax(game, game.take(action, state), player)
        evaluations[action] = e
        nn += n
    
    #print("{}".format(f.__name__))
    best = f(evaluations, key=evaluations.get)
    return best, evaluations[best], nn


class MinMax(Controller):
    def select_action(self, game, state):
        action, evaluation, n = minmax(game, state, self.player)
        print(n)
        return action


class Random(Controller):
    def select_action(self, game, state):
        return random.choice(available_actions(game, state))
