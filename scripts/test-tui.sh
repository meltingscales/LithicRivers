#!/usr/bin/env bash

# tui tests
# start new tmux session
tmux new-session -d -s  lrtest

# start game
tmux send-keys      -t  lrtest:  "bash ./scripts/start.sh" "Enter"
sleep 10

#  ensure we are on 'Help' screen
screen=$(tmux capture-pane -t    lrtest: -S - -E - -p | cat)
if [[ ! "$screen" == *"Help"* ]]; then
  echo "${screen}"
  echo "We should be on the 'Help' screen, but we are not! Failing!"
  exit 1
else
  echo "We are on the 'Help' screen as expected."
fi

# go to the game display screen
tmux send-keys      -t  lrtest:  "Space"
sleep 1

# ensure we are see "Cookie"
screen=$(tmux capture-pane -t    lrtest: -S - -E - -p | cat)
if [[ ! "$screen" == *"Cookie"* ]]; then
  echo "${screen}"
  echo "We should see 'Cookie', but we do not! Failing!"
  exit 1
else
  echo "We see 'Cookie' as expected.";
fi

# stop game
tmux kill-session -t lrtest:

exit 0