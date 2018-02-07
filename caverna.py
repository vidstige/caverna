class Player(object):
    def __init__(self):
        self.dwarfs = [0, 0]
        self.resources = dict(food=2)


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
            self.dwarfs = {}  # placed dwarfs

    def __init__(self, players):
        self.actions = [
            Action("Drift Mining", dict(stones=(1, 1))),
            Action("Logging", dict(wood=(3, 1))),
        ]
        self.players = players
        self.state = Game.State(players.values())

    def over(self):
        return self.state.round > 12

    def score(self, player):
        return 0


def available_actions(game, state=None):
    s = state or game.state
    return [a for a in game.actions if a not in s.dwarfs]


class Controller(object):
    def __init__(self, player):
        self.player = player

    def select_action(self, state):
        return None
