#!/usr/bin/env bash

# normal unit tests
uv run coverage run -m unittest discover lithicrivers
uv run coverage lcov -o coverage/lcov.info

exit 0