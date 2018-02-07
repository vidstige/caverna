from caverna import Controller, available_actions

class Random(Controller):
    def select_action(self, game):
        return available_actions(game)[0]
