from typing import Dict, List
from caverna import *
import ai


def play(game: Game, controllers: Dict[Player, Controller]) -> None:
    # Take actions while any player has dwarfs
    while game.round():
        player = game.current()
        action = controllers[player].select_action(game)
        print("{} selects {}".format(game.names[player], action.name))
        game.take(action)

    # Return dwarfs
    game.return_dwarfs()

    # Harvest crops
    pass

    # Feed dwarfs
    for player in game.state.players:
        pass

    # Breed animals
    pass


def main():
    samuel = Player()
    maria = Player()
    controllers = {samuel: ai.Random(samuel), maria: ai.Random(maria)}
    game = Game({samuel: 'samuel', maria: 'maria'})
    while not game.over():
        print("Round {}".format(game.state.round))
        play(game, controllers)
        game.state.round += 1

    for player in game.state.players:
        name = game.names[player]
        score = game.score(player)
        print(player.resources)
        print("{name}: {score}".format(name=name, score=score))


if __name__ == "__main__":
    main()
