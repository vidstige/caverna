from caverna import *
import ai

def main():
    samuel = Player()
    maria = Player()
    controllers = {samuel: ai.Random(samuel), maria: ai.Random(maria)}
    game = Game({'samuel': samuel, 'maria': maria})
    while not game.over():
        print("Round {}".format(game.state.round))

        # Take actions
        for name, player in game.players.items():
            if player.dwarfs:
                dwarf = player.dwarfs.pop()
                action = controllers[player].select_action(game)
                print("{} selects {}".format(name, action.name))
                game.state.dwarfs[action] = (player, dwarf)

        # Return dwarfs
        for player, dwarf in game.state.dwarfs.values():
            player.dwarfs.append(dwarf)
        game.state.dwarfs = {}

        # Replenish resources

        # Harvest crops

        # Feed dwarfs
        for player in game.state.players:
            pass

        # Breed animals

        # Next turn
        game.state.round += 1

    for name, player in game.players.items():
        score = game.score(player)
        print("{name}: {score}".format(name=name, score=score))


if __name__ == "__main__":
    main()
