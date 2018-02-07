from caverna import *


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
