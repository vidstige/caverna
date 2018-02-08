from typing import List, Tuple
import random
from caverna import *


class Random(Controller):
    def select_action(self, game):
        return random.choice(available_actions(game))
