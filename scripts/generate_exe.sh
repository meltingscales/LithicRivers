#!/usr/bin/env bash

if [[ $platform == 'linux' ]]; then
  PYCMD=python
else #osx needs to use python3
  PYCMD=python3
fi

$PYCMD -m pipenv run pyinstaller ./lithicrivers.spec