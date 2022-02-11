#!/usr/bin/env bash

echo "Make sure pipenv exists..."

which pipenv
# if exit code is nonzero, it is not a command.
if [ "$?" -eq "1" ]; then
  if [[ $platform == 'linux' ]]; then
    python -m pip install pipenv
  else #osx needs to use python3
    python3 -m pip install pipenv
  fi
fi


pipenv install

python -m pipenv install --dev
