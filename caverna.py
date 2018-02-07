from typing import Tuple


class Tile(object):
    def __init__(self, dwarfs=0, animals=0):
        self.dwarfs = 0
        self.animals = 0

    def parts(self):
        yield self


class TwinTile(object):
    def __init__(self, left: Tile, right: Tile):
        self.left = left
        self.right = right

    def parts(self):
        yield self.left
        yield self.right

    def __repr__(self):
        return "{}({}, {})".format(
            self.__class__.__name__, self.left, self.right)

Meadow = Tile()
Field = Tile()
Pasture = Tile()
Excavated = Tile()
Mine = Tile()
EntryLevelDwelling = Tile(dwarfs=2, animals=2)


# Twin tiles
Outdoor = TwinTile(Meadow, Field)
ExcavatedTwin = TwinTile(Excavated, Excavated)
ExcavatedAndMine = TwinTile(Excavated, Mine)


class Player(object):
    def __init__(self):
        self.dwarfs = [0, 0]
        self.resources = dict(food=2)
        self.tiles = {(3, 2): Excavated, (3, 3): EntryLevelDwelling}


class Action(object):
    def __init__(self, name, resources, tiles=None):
        self.name = name
        self.resources = resources
        self.tiles = tiles


class Game(object):
    class State(object):
        """Encompasses and entire game state"""
        def __init__(self, players):
            self.players = players
            self.round = 0
            self.action_resources = {}
            self.dwarfs = {}  # placed dwarfs

    def __init__(self, players):
        self.actions = [
            Action("Drift Mining", dict(stones=(1, 1)), tiles=(ExcavatedAndMine,)),
            Action("Logging", dict(wood=(3, 1)), tiles=(Outdoor,)),
            Action("Wood Gathering", dict(wood=(1, 1))),
            Action("Excavation", dict(stone=(1, 1)), tiles=(ExcavatedTwin, ExcavatedAndMine)),
            Action("Supplies", dict(wood=(1, 0), stone=(1, 0), coal=(1, 0), food=(1, 0), coins=(2, 0))),
            Action("Clearing", dict(wood=(1, 1)), tiles=(Outdoor,)),
        ]
        self.players = players
        self.state = Game.State(players.values())

    def return_dwarfs(self):
        for player, dwarf in self.state.dwarfs.values():
            player.dwarfs.append(dwarf)
        self.state.dwarfs = {}

    def replenish(self):
        for action in self.actions:
            for resource, rc in action.resources.items():
                initial, per_round = rc
                current = self.state.action_resources.get(action, {})
                if resource in current:
                    current[resource] += per_round
                else:
                    current[resource] = initial
                self.state.action_resources[action] = current

    def gain_resources(self, action: Action, player: Player) -> None:
        action_resources = self.state.action_resources.pop(action)
        for resource, count in action_resources.items():
            if resource not in player.resources:
                player.resources[resource] = 0
            player.resources[resource] += count

    def over(self):
        return self.state.round > 12

    def score(self, player):
        return player.resources.get('coins', 0) + \
            (len(player.tiles) - 24) + \
            len(player.dwarfs)


def available_actions(game, state=None):
    s = state or game.state
    return [a for a in game.actions if a not in s.dwarfs]


class Controller(object):
    def __init__(self, player):
        self.player = player

    def select_action(self, state):
        return None

    def place(self, tiles: Tuple[Tile]):
        pass
