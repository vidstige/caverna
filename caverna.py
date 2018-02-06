import ai

class Player(object):
    def __init__(self, name):
        self.name = name
        self.dwarfs = [0, 0]
        self.resources = dict(food=2)


class Action(object):
    def __init__(self, name, r):
        self.name = name
        self.resources = r


class Game(object):
    def __init__(self, players):
        self.players = players
        self.actions = [Action(name="Drift Mining", r=dict(stones=1))]
        self.turn = 0
        self.dwarfs = {}  # placed dwarfs

    def over(self):
        return self.turn > 8

    def score(self, player):
        return 0


def main():
    samuel = Player("samuel")
    maria = Player("maria")
    controllers = {samuel: ai.Random(samuel), maria: ai.Random(maria)}
    game = Game([samuel, maria])
    while not game.over():
        print("Turn {}".format(game.turn))

        # Take actions
        for player in game.players:
            if player.dwarfs:
                dwarf = player.dwarfs.pop()
                action = controllers[player].select_action()
                game.dwarfs[action] = (player, dwarf)

        # Return dwarfs
        for player, dwarf in game.dwarfs.values():
            player.dwarfs.append(dwarf)
        game.dwarfs = {}

        # Replenish resources

        # Harvest crops

        # Feed dwarfs
        for player in game.players:
            pass

        # Breed animals

        # Next turn
        game.turn += 1

    for player in game.players:
        score = game.score(player)
        print("{name}: {score}".format(name=player.name, score=score))


if __name__ == "__main__":
    main()
