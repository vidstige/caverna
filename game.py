from typing import Dict, List
from caverna import *
import ai


def play_round(game: Game, state: Game.State, controllers: Dict[Player, Controller]) -> None:
    # Take actions while any player has dwarfs
    while game.round(state):
        player = game.current_player(state)
        action = controllers[player].select_action(game, state)
        print("{} selects {}".format(player, action.name))
        state = game.take(action, state)

    # Return dwarfs
    game.return_dwarfs(state)
    game.harvest(state)
    return state


def main():
    samuel = Player('Samuel')
    maria = Player('Maria')
    controllers = {samuel: ai.MinMax(samuel), maria: ai.Random(maria)}
    game = Game([samuel, maria])
    state = game.initial()
    while not game.over(state):
        print("Round {}".format(state.round))
        state = play_round(game, state, controllers)
        state.round += 1

    for player in game.players:
        name = player
        score = game.score(state, player)
        print("{name}: {score}".format(name=name, score=score))
        print(state.player_states[player].resources)
    print(controllers[samuel].total)

if __name__ == "__main__":
    main()
