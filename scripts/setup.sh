#!/usr/bin/env bash

if [[ $platform == 'linux' ]]; then
  PYCMD=python
else #osx needs to use python3
  PYCMD=python3
fi


echo "Make sure pipenv exists..."

which pipenv
# if exit code is nonzero, it is not a command.
if [ "$?" -eq "1" ]; then
  $PYCMD -m pip install pipenv
fi

$PYCMD -m pipenv install --dev
