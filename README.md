# LithicRivers

rocks and shit

ascii dwarf mine game, very wip...

## play it

<https://github.com/HenryFBP/LithicRivers/releases>

### how to play

read the help page

## what is this

an unfinished game :P

take a look at some of our reviews below...

![actual screenshot](/media/screenshot.png)

![why do we exist...just to suffer...?](/media/dafuq.png)

![gnerf](http://images3.memedroid.com/images/UPLOADED727/5c1d01829c2ff.jpeg)

god help me

## running

    pipenv install
    pipenv shell
    python -m lithicrivers

## notes

- https://github.com/peterbrittain/asciimatics/blob/v1.13/samples/tab_demo.py
- https://stackoverflow.com/questions/9575409/calling-parent-class-init-with-multiple-inheritance-whats-the-right-way/50465583#50465583

## TODO

- add procedurally generated catgirls
- do perf testing to fix lag on startup...thanks numpy...
- auto-scale the viewport based off of viewable area
    - determine from screen size
    - determine from "blank space" in asciimatics (if this is even doable)
- don't use numpy, its a bloated big ass chungoid that adds to the pyinstaller exe size