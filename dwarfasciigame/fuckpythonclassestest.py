class Entity(object):

    def __init__(self):
        self.x = 0
        self.y = 0
        self.health = 100
        self.stamina = 100


class Renderable(object):
    def __init__(self, sprite_sheet):
        self.sprite_sheet = sprite_sheet
        if not sprite_sheet:
            self.sprite_sheet = ['?', '??\n'
                                      '??', '???\n'
                                            '???\n'
                                            '???']

    def render(self, scale: int = 1) -> str:
        normscale = scale - 1
        return self.sprite_sheet[normscale]


class Player(Entity, Renderable):

    def __init__(self):
        # super(Entity, self).__init__()

        # super(Renderable, self).__init__(sprite_sheet=['$', '[]\n'
        #                                                     '%%'])

        super(Entity).__init__()
        super(Renderable).__init__(['$', '[]\n'
                                         '%%'])


p = Player()
