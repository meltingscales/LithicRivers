import logging
import os.path
import sys

from asciimatics.exceptions import ResizeScreenError
from asciimatics.screen import Screen

from lithicrivers.game import Game
from lithicrivers.settings import LOGFILENAME
from lithicrivers.ui import demo

if os.path.exists(LOGFILENAME):
    os.remove(LOGFILENAME)

logging.basicConfig(filename=LOGFILENAME, level=logging.DEBUG)

# Press the green button in the gutter to run the script.
if __name__ == '__main__':

    GAME = Game()

    print(f"see {LOGFILENAME} for logs.")

    logging.info('wow its PyCharm!')

    logging.debug(GAME.render_world())

    last_scene = None
    while True:
        try:
            Screen.wrapper(demo, catch_interrupt=True, arguments=[last_scene, GAME])
            sys.exit(0)
        except ResizeScreenError as e:
            last_scene = e.scene

# See PyCharm help at https://www.jetbrains.com/help/pycharm/