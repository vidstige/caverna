
class Player(object):
    def __init__(self, name):
        self.name = name
        self.points = 0


class Game(object):
    def __init__(self, players):
        self.players = players
        self.counter = 0

    def over(self):
        return True


def main():
    players = [Player("samuel"), Player("maria")]
    game = Game(players)
    while not game.over():
        # Take actions
        for player in game.players:
            pass
        
        # Replenish resources

        # Harvest crops

        # Feed dwarfs
        for player in game.players:
            pass

        # Breed animals


if __name__ == "__main__":
    main()
