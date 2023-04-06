MODE := "release"
PROJECT_NAME := "stm"

.PHONY: build
build: ## Build the binary
	cargo build --release

.PHONY: objcopy
objcopy: build ## Create stm32 rom image
	cargo objcopy -- -O binary target/thumbv7em-none-eabihf/${MODE}/${PROJECT_NAME} rom.bin

.PHONY: flash
flash: objcopy ## Flash rom to stm
	st-flash write rom.bin 0x8000000

## Help display.
## Pulls comments from beside commands and prints a nicely formatted
## display with the commands and their usage information.

.DEFAULT_GOAL := help

help: ## Prints this help
	@grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
