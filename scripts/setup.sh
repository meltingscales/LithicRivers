#!/usr/bin/env bash

if [[ $platform == 'linux' ]]; then
  PYCMD=python
else #osx needs to use python3
  PYCMD=python3
fi


echo "Make sure poetry exists..."

which poetry
# if exit code is nonzero, it is not a command.
if [ "$?" -eq "1" ]; then
  $PYCMD -m pip install poetry
fi

$PYCMD -m poetry install --dev
