import logging
import os.path

from asciimatics.exceptions import ResizeScreenError
from asciimatics.screen import Screen

from lithicrivers.game import Game
from lithicrivers.model.modelpleasemoveme import StopGame
from lithicrivers.settings import LOGFILENAME, GAME_NAME, LOGGINGLEVEL
from lithicrivers.ui import demo

if os.path.exists(LOGFILENAME):
    os.remove(LOGFILENAME)

logging.basicConfig(filename=LOGFILENAME, level=LOGGINGLEVEL)

# Press the green button in the gutter to run the script.
if __name__ == '__main__':

    GAME = Game()

    print(f"Welcome to {GAME_NAME}.\n"
          f"See '{LOGFILENAME}' for logs.")

    last_scene = None
    while GAME.running:
        try:
            logging.debug("Running Screen.wrapper()")
            Screen.wrapper(demo, catch_interrupt=True, arguments=[last_scene, GAME])
        except StopGame as se:
            logging.debug("Caught StopGame!")
            GAME.running = False
        except ResizeScreenError as rse:
            logging.debug("Caught ResizeScreenError !")
            pass
            # Screen rendering stops and re-starts if we get a ResizeScreenError since we're in a while loop...

    print("Game is no longer running :3c")
    print("Goodbye!")

    exit(0)

# See PyCharm help at https://www.jetbrains.com/help/pycharm/
