from typing import Tuple
import random
from caverna import Controller, available_actions, Tile

# 433xxx
# 452xxx
# 152xxx
# 100xxx
TILE_ORDER = [
    (2, 3), (1, 3),
    (0, 3), (0, 2),
    (2, 2), (2, 1),
    (2, 0), (1, 0),
    (0, 0), (0, 1),
    (1, 1), (1, 2),
]

class Random(Controller):
    def select_action(self, game):
        return random.choice(available_actions(game))

    def place(self, tiles: Tuple[Tile]):
        # for now just place the tiles in a certain order
        pass