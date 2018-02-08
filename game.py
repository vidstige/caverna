from typing import List
from caverna import *
import ai


def play(game: Game, controllers: List[Controller]) -> None:
    # Replenish resources (this is done first to ensure initial supply)
    game.replenish()

    # Take actions while any player has dwarfs
    frozen_players = list(game.state.players)
    while any(p.dwarfs for p in game.state.players):
        for player in frozen_players:
            if player.dwarfs:
                dwarf = player.dwarfs.pop()
                action = controllers[player].select_action(game)
                print("{} selects {}".format(game.names[player], action.name))
                
                # place dwarf
                game.state.dwarfs[action] = (player, dwarf)
                
                # gain resources
                game.gain_resources(action, player)

                # actions such as starting player,
                # sow.
                for a in action.actions:
                    a(player)

                # place tile
                tiles = action.tiles
                if tiles:
                    controllers[player].place(tiles)

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
