from typing import Dict, List, Tuple
from collections import defaultdict
from copy import deepcopy

class Tile(object):
    def __init__(self, dwarfs=0, animals=0):
        self.dwarfs = 0
        self.animals = 0

    def parts(self):
        yield self

    def __deepcopy__(self, memo):
        return self  # don't copy


class TwinTile(Tile):
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


class Player(str):
    pass

class PlayerState(object):
    def __init__(self):
        self.dwarfs = [0, 0]
        self.resources = defaultdict(lambda: 0, food=2)  # resources on hand
        self.tiles = {(3, 2): Excavated, (3, 3): EntryLevelDwelling}
        self.field_resources = {}  # mapping from coordinate -> resources dict


class Action(object):
    """Represents an action space in Caverna"""
    def __init__(self, name, resources, tiles=None, actions=[]):
        self.name = name
        self.resources = resources
        self.tiles = tiles
        self.actions = actions

    def __repr__(self):
        return "Action(name={name})".format(name=self.name)
    
    def __deepcopy__(self, memo):
        return self  # don't copy

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

def next_free(tiles: Dict[Tuple[int, int], Tile]) -> Tuple[int, int]:
    for p in TILE_ORDER:
        if p not in tiles:
            return p
    return None


def autoplace(player_state: PlayerState, tiles: Tuple[Tile]):
    tile = tiles[0]  # select first tile of multiple available
    
    for t in tile.parts():
        p = next_free(player_state.tiles)
        if p:
            player_state.tiles[p] = t


FIELD_GAIN = {'wheat': 2, 'vegetable': 1}
def sow(ps: PlayerState, resource_name: str):
    """If possible, sows one resource_name from hand to empty field"""
    # coordinate of next free Field tile
    empty_field = next((tc for tc, tile in ps.tiles.items() if tile == Field and not ps.field_resources.get(tc)), None)    
    if empty_field and ps.resources[resource_name] > 0:
        ps.resources[resource_name] -= 1
        ps.field_resources[empty_field] = {resource_name: 1 + FIELD_GAIN[resource_name]}


class Game(object):
    class State(object):
        """Encompasses and entire game state"""
        def __init__(self, players: List[Player]):
            self.player_states = {p: PlayerState() for p in players}
            self.round = 0
            self.action_resources = {}
            self.dwarfs = {}  # placed dwarfs
            self.starting = 0  # starting player index
            self.current = 0  # current player index

    def __init__(self, players: List[Player]):
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
            Action("House Work", dict(), actions=[self.furinsh_cavern]),
            Action("Slash and Burn", dict(), tiles=(Outdoor,), actions=[self.sow]),

            # TODO: These should be randomized
            Action("Sheep Farming", dict(sheep=(1,1)), actions=[self.build_pastry, self.build_stable]),
            Action("Blacksmithing", dict(), actions=[self.equip, self.expedition]),
            Action("Ore mine construction", dict()),
        ]
        self.players = players

    def initial(self) -> State:
        state = Game.State(self.players) 
        self.replenish(state)
        return state

    def current_player(self, state: State) -> Player:
        return self.players[state.current]

    # action functions
    def equip(self, player: Player, state: State):
        pass

    def expedition(self, player: Player, state: State):
        pass

    def build_pastry(self, player: Player, state: State):
        pass

    def build_stable(self, player: Player, state: State):
        pass

    def starting_player(self, player: Player, state: State):
        # sets the current player to starting player
        state.starting = self.players.index(player)

    def sow(self, player: Player, state: State):
        ps = state.player_states[player]
        sow(ps, 'wheat')
        sow(ps, 'wheat')
        sow(ps, 'vegetable')
        sow(ps, 'vegetable')

    def furinsh_cavern(self, player: Player, state: State):
        pass

    def round(self, state: State):
        """Whether the dwarf placing phase is still ongoing"""
        return any(ps.dwarfs for ps in state.player_states.values())

    def take(self, action: Action, state1: State) -> State:
        state = deepcopy(state1)
        #state = self.state

        player = self.current_player(state)
        ps = state.player_states[player]
        dwarf = ps.dwarfs.pop()

        # place dwarf
        state.dwarfs[action] = (player, dwarf)
        
        # gain resources
        self.gain_resources(state, action, player)

        # actions such as starting player, sow
        for a in action.actions:
            a(player, state)

        # place tiles, if any
        if action.tiles:
            autoplace(state.player_states[player], action.tiles)

        # next player
        if self.round(state):
            # find next player with dwarfs left
            state.current = (state.current + 1) % len(self.players)
            while not state.player_states[self.players[state.current]].dwarfs:
                state.current = (state.current + 1) % len(self.players)
        else:
            state.current = state.starting

        return state

    def return_dwarfs(self, state: State):
        for player, dwarf in state.dwarfs.values():
            state.player_states[player].dwarfs.append(dwarf)
        state.dwarfs = {}
    
    def harvest(self, state: State):
        # Harvest crops
        for ps in state.player_states.values():
            for tc, tile in ps.tiles.items():
                field_resources = ps.field_resources.get(tc, {})
                for r in field_resources:
                    if field_resources[r] > 0:
                        field_resources[r] -= 1
                        ps.resources[r] += 1
                    

        # Feed dwarfs
        for player in self.players:
            pass

        # Breed animals
        pass

        self.replenish(state)

    def replenish(self, state: State):
        for action in self.actions:
            for resource, rc in action.resources.items():
                initial, per_round = rc
                current = state.action_resources.get(action, {})
                if resource in current:
                    current[resource] += per_round
                else:
                    current[resource] = initial
                state.action_resources[action] = current

    def gain_resources(self, state: State, action: Action, player: Player) -> None:
        action_resources = state.action_resources.pop(action, {})
        for resource, count in action_resources.items():
            state.player_states[player].resources[resource] += count

    def over(self, state: State):
        return state.round > 8

    def score(self, state: State, player: Player):
        animals = ('sheep', 'donkey')
        ps = state.player_states[player]
        return \
            (ps.resources.get('wheat', 0) + sum(r.get('wheat', 0) for r in ps.field_resources.values()) + 1) // 2 + \
            sum(ps.resources.get(a, 0) for a in animals) + \
            ps.resources.get('coin', 0) + \
            ps.resources.get('ruby', 0) + \
            (len(ps.tiles) - 24) + \
            len(ps.dwarfs)


def available_actions(game, state):
    return [a for a in game.actions if a not in state.dwarfs]


class Controller(object):
    def __init__(self, player):
        self.player = player

    def select_action(self, state):
        return None

    def place(self, tiles: Tuple[Tile]):
        pass
