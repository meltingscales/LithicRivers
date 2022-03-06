#!/usr/bin/env bash

# normal unit tests
poetry run coverage run -m unittest discover lithicrivers
poetry run coverage lcov -o coverage/lcov.info

exit 0