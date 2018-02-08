from typing import Dict, List, Tuple


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
    """Represents an action space in Caverna"""
    def __init__(self, name, resources, tiles=None, actions=[]):
        self.name = name
        self.resources = resources
        self.tiles = tiles
        self.actions = actions


# Tile autoplacing

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

def next_free(tiles: List[Tuple[int, int]]) -> Tuple[int, int]:
    for p in TILE_ORDER:
        if p not in tiles:
            return p
    return None


def autoplace(player: Player, tiles: Tuple[Tile]):
    tile = tiles[0]  # select first tile of multiple available
    
    for t in tile.parts():
        p = next_free(player.tiles)
        if p:
            player.tiles[p] = p


class Game(object):
    class State(object):
        """Encompasses and entire game state"""
        def __init__(self, players: List[Player]):
            self.players = players
            self.round = 0
            self.action_resources = {}
            self.dwarfs = {}  # placed dwarfs
            self.current = 0  # current player index
            self.starting = 0  # starting player index

    def __init__(self, players: Dict[Player, str]):
        self.actions = [
            Action("Drift Mining", dict(stones=(1, 1)), tiles=(ExcavatedAndMine,)),
            Action("Logging", dict(wood=(3, 1)), tiles=(Outdoor,)),
            Action("Wood Gathering", dict(wood=(1, 1))),

            Action("Excavation", dict(stone=(1, 1)), tiles=(ExcavatedTwin, ExcavatedAndMine)),
            Action("Supplies", dict(wood=(1, 0), stone=(1, 0), coal=(1, 0), food=(1, 0), coin=(2, 0))),
            Action("Clearing", dict(wood=(1, 1)), tiles=(Outdoor,)),

            Action("Starting Player", dict(food=(1, 1)), actions=[self.starting_player]),
            Action("Ore Mining", dict(coal=(2, 1))),
            Action("Sustenance", dict(food=(1, 1), wheat=(1, 0)), tiles=(Outdoor,)),

            Action("Ruby Mining", dict(ruby=(1, 1))),
            Action("House Work", dict(), actions=[self.furinsh_cavern]),  # TODO: Furinsh cavern
            Action("Slash and Burn", dict(), tiles=(Outdoor,), actions=[self.sow]),
        ]
        self.names = players
        self.state = Game.State(list(players.keys()))
        self.replenish()

    # action functions
    def starting_player(self, player: Player):
        self.state.starting = self.state.players.index(player)

    def sow(self, player: Player):
        pass
    def furinsh_cavern(self, player: Player):
        pass

    def round(self, state=None):
        """Whether the dwarft placing phase is still ongoing"""
        s = state or self.state
        return any(p.dwarfs for p in s.players)

    def current(self, state=None):
        s = state or self.state
        return s.players[s.current]

    def take(self, action: Action):
        player = self.current()
        dwarf = player.dwarfs.pop()

        # place dwarf
        self.state.dwarfs[action] = (player, dwarf)
        
        # gain resources
        self.gain_resources(action, player)

        # actions such as starting player, sow
        for a in action.actions:
            a(player)

        # place tiles, if any
        if action.tiles:
            autoplace(player, action.tiles)

        # next player
        if self.round():
            # find next player with dwarfs left
            self.state.current = (self.state.current + 1) % len(self.state.players)
            while not self.state.players[self.state.current].dwarfs:
                self.state.current = (self.state.current + 1) % len(self.state.players)
        else:
            self.state.current = self.state.starting

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
        action_resources = self.state.action_resources.pop(action, {})
        for resource, count in action_resources.items():
            if resource not in player.resources:
                player.resources[resource] = 0
            player.resources[resource] += count

    def over(self):
        return self.state.round > 12

    def score(self, player):
        return \
            (player.resources.get('wheat', 0) + 1) // 2 + \
            player.resources.get('coin', 0) + \
            player.resources.get('ruby', 0) + \
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
