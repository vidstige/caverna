from typing import List, Tuple
import random
from caverna import *

leafs = 0
def minmax(game: Game, state: Game.State, player: Player) -> Tuple[Action, int]:
    global leafs
    #print(state.round)
    if game.over(state):
        leafs += 1
        if leafs % 307 == 0:
            print("leaf {}".format(leafs))
        return None, game.score(state, player)

    if not game.round(state):
        game.return_dwarfs(state)
        game.harvest(state)
        state.round += 1

    #evaluations = {action: minmax(game, game.take(action, state), player)[1] for action in available_actions(game, state)}
    evaluations = {}
    for action in available_actions(game, state):
        #print("{} Decending into {}".format(state.round, action))
        evaluations[action] = minmax(game, game.take(action, state), player)[1]
    best = max(evaluations, key=evaluations.get)
    return best, evaluations[best]


class MinMax(Controller):
    def select_action(self, game, state):
        action, evaluation = minmax(game, state, self.player)
        return action


class Random(Controller):
    def select_action(self, game, state):
        return random.choice(available_actions(game, state))
