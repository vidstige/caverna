class Player(object):
    def __init__(self):
        self.dwarfs = [0, 0]
        self.resources = dict(food=2)
        self.tiles = {(3, 2): 'excavated', (3, 3): 'starting cave'}


class Action(object):
    def __init__(self, name, resources):
        self.name = name
        self.resources = resources


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
            Action("Drift Mining", dict(stones=(1, 1))),
            Action("Logging", dict(wood=(3, 1))),
            Action("Wood Gathering", dict(wood=(1, 1))),
            Action("Excavation", dict(stone=(1, 1))),
            Action("Supplies", dict(wood=(1, 0), stone=(1, 0), coal=(1, 0), food=(1, 0), coins=(2, 0))),
            Action("Clearing", dict(wood=(1, 1))),
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
        print(player.resources)
        return player.resources.get('coins', 0) + (len(player.tiles) - 24)


def available_actions(game, state=None):
    s = state or game.state
    return [a for a in game.actions if a not in s.dwarfs]


class Controller(object):
    def __init__(self, player):
        self.player = player

    def select_action(self, state):
        return None
