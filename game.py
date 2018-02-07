from caverna import *
import ai

def main():
    samuel = Player()
    maria = Player()
    controllers = {samuel: ai.Random(samuel), maria: ai.Random(maria)}
    game = Game({'samuel': samuel, 'maria': maria})
    while not game.over():
        print("Round {}".format(game.state.round))

        # Replenish resources (this is done first to ensure initial supply)
        for action in game.actions:
            for resource, rc in action.resources.items():
                initial, per_round = rc
                current = game.state.action_resources.get(action, {})
                if resource in current:
                    current[resource] += per_round
                else:
                    current[resource] = initial
                game.state.action_resources[action] = current

        # Take actions
        for name, player in game.players.items():
            if player.dwarfs:
                dwarf = player.dwarfs.pop()
                action = controllers[player].select_action(game)
                print("{} selects {}".format(name, action.name))
                
                # place dwarf
                game.state.dwarfs[action] = (player, dwarf)
                
                # gain resources
                action_resources = game.state.action_resources.pop(action)
                for resource, count in action_resources.items():
                    if resource not in player.resources:
                        player.resources[resource] = 0
                    player.resources[resource] += count

        # Return dwarfs
        for player, dwarf in game.state.dwarfs.values():
            player.dwarfs.append(dwarf)
        game.state.dwarfs = {}

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
