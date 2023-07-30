# == config ==

#BUILD := debug
BUILD := release

TARGET := aarch64-unknown-linux-musl

BINARY_NAME := robolab

# export PKG_CONFIG_ALLOW_CROSS = "1"
# export PKG_CONIFG_PATH = "/Users/hrmny/code/rust/roboclub-lighting/raspi/sysroot/usr/lib/aarch64-linux-gnu/pkgconfig"
# export PKG_CONFIG_SYSROOT_DIR = "C:\\Users\\harmony\\Code\\Rust\\roboclub\\raspi\\sysroot"
# export PKG_CONIFG_PATH = "C:\\Users\\harmony\\Code\\Rust\\roboclub\\raspi\\sysroot\\usr\\lib\\aarch64-linux-gnu\\pkgconfig"
# export PKG_CONIFG_PATH = "/usr/lib/aarch64-linux-gnu/pkgconfig"

export PKG_CONFIG_PATH = /Users/hrmny/code/rust/roboclub-lighting/raspi/sysroot/usr/lib/aarch64-linux-gnu/pkgconfig
export PKG_CONFIG_SYSROOT_DIR = /Users/hrmny/code/rust/roboclub-lighting/raspi/sysroot

TARGET_HOST := roboblast.cbrp3.c-base.org
TARGET_USER := alien
TARGET_PASS := alien
TARGET_DIR := /home/${TARGET_USER}/leds

# ======================================

export CC_aarch64_unknown_linux_gnu = aarch64-unknown-linux-gnu-gcc
export CXX_aarch64_unknown_linux_gnu = aarch64-unknown-linux-gnu-g++
export AR_aarch64_unknown_linux_gnu = aarch64-unknown-linux-gnu-ar
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-unknown-linux-gnu-gcc

BIN := target/${TARGET}/${BUILD}/${BINARY_NAME}

# Replace target in .cargo/config.toml to always match the one here
_ := $(shell sed -i "" -e "s/^target =.*/target = \"${TARGET}\"/" .cargo/config.toml)

.PHONY: pkg-config
pkg-config: ## build the firmware
	@pkg-config --libs --cflags libspa-0.2

.PHONY: build
build: ## build the firmware
ifeq ($(BUILD), release)
	cargo build --target ${TARGET} --release
else
	cargo build --target ${TARGET}
endif

.PHONY: clean
clean: ## clean artifacts
	cargo clean

.PHONY: copy-bin
copy-bin: build ## copy the binary to the target device
	sshpass -p ${TARGET_PASS} rsync -avzz ./${BIN} ${TARGET_USER}@${TARGET_HOST}:${TARGET_DIR}
	@# sshpass -p ${TARGET_PASS} rsync -avzz --info=progress2 ./${BIN} ${TARGET_USER}@${TARGET_HOST}:${TARGET_DIR}

.PHONY: copy-public
copy-public: ## copy the public dir to the target device
	sshpass -p ${TARGET_PASS} rsync -avzz ../frontend/dist/ ${TARGET_USER}@${TARGET_HOST}:${TARGET_DIR}/public
	@# sshpass -p ${TARGET_PASS} rsync -avzz --info=progress2 ../frontend/dist/ ${TARGET_USER}@${TARGET_HOST}:${TARGET_DIR}/public

.PHONY: copy
copy: copy-bin copy-public ## copy everything target device

.PHONY: remote-run
remote-run: copy ## copy and run on the target device and monitor logs
	@sshpass -p ${TARGET_PASS} ssh -t ${TARGET_USER}@${TARGET_HOST} cd ${TARGET_DIR}; RUST_BACKTRACE=full RUST_LOG=debug ${TARGET_DIR}/${BINARY_NAME} /dev/spidev3.0

.PHONY: remote-restart
remote-restart: copy ## copy and restart the service on the target device
	@sshpass -p ${TARGET_PASS} ssh -t ${TARGET_USER}@${TARGET_HOST} systemctl --user restart leds

.PHONY: check
check: ## cargo check
	cargo check

## Help display.
## Pulls comments from beside commands and prints a nicely formatted
## display with the commands and their usage information.

.DEFAULT_GOAL := help

help: ## Prints this help
	@grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
