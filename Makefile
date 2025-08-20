.DEFAULT_GOAL := help

.PHONY: help
help:
	@just

# Pass-through: `make <target>` -> `just <target>`
# Extra args can be passed as: `make <target> ARGS="--release"`
%:
	@just $@ $(ARGS)