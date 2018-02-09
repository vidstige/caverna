from typing import Dict, List
from caverna import *
import ai


def play_round(game: Game, controllers: Dict[Player, Controller]) -> None:
    # Take actions while any player has dwarfs
    while game.round():
        player = game.state.current_player()
        action = controllers[player].select_action(game)
        print("{} selects {}".format(player.name, action.name))
        game.take(action)

    # Return dwarfs
    game.return_dwarfs(game.state)
    game.harvest()


def main():
    samuel = Player('Samuel')
    maria = Player('Maria')
    controllers = {samuel: ai.Random(samuel), maria: ai.Random(maria)}
    game = Game([samuel, maria])
    while not game.over():
        print("Round {}".format(game.state.round))
        play_round(game, controllers)
        game.state.round += 1

    for player in game.state.players:
        name = player.name
        score = game.score(player)
        print(game.state.player_states[player].resources)
        print("{name}: {score}".format(name=name, score=score))


if __name__ == "__main__":
    main()
