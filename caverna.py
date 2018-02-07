import ai

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
        self.state = Game.State(players)

    def players(self):
        return self.state.players.values()

    def over(self):
        return self.state.round > 12

    def score(self, player):
        return 0


def main():
    samuel = Player()
    maria = Player()
    controllers = {samuel: ai.Random(samuel), maria: ai.Random(maria)}
    game = Game({'samuel': samuel, 'maria': maria})
    while not game.over():
        print("Round {}".format(game.state.round))

        # Take actions
        for player in game.players():
            if player.dwarfs:
                dwarf = player.dwarfs.pop()
                action = controllers[player].select_action()
                game.state.dwarfs[action] = (player, dwarf)

        # Return dwarfs
        for player, dwarf in game.state.dwarfs.values():
            player.dwarfs.append(dwarf)
        game.dwarfs = {}

        # Replenish resources

        # Harvest crops

        # Feed dwarfs
        for player in game.players():
            pass

        # Breed animals

        # Next turn
        game.state.round += 1

    for name, player in game.state.players.items():
        score = game.score(player)
        print("{name}: {score}".format(name=name, score=score))


if __name__ == "__main__":
    main()
