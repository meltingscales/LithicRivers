#!/usr/bin/env bash

pipenv run coverage run -m unittest discover lithicrivers
pipenv run coverage lcov -o coverage/lcov.info