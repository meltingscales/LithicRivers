#!/usr/bin/env bash

if [[ $platform == 'linux' ]]; then
  PYCMD=python
else #osx needs to use python3
  PYCMD=python3
fi

echo "Installing uv..."

# Install uv if not already installed
which uv
if [ "$?" -eq "1" ]; then
  $PYCMD -m pip install uv
fi

# Install dependencies with uv
uv sync --extra dev
