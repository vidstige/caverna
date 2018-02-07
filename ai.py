import random
from caverna import Controller, available_actions

class Random(Controller):
    def select_action(self, game):
        return random.choice(available_actions(game))
