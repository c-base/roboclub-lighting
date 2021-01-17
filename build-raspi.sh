#!/bin/bash
set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly TARGET_ARCH=armv7-unknown-linux-gnueabihf
readonly LINK_FLAGS='-L /usr/arm-linux-gnueabihf/lib/ -L /usr/lib/arm-linux-gnueabihf/'

RUSTFLAGS=${LINK_FLAGS} cross build --release --target=${TARGET_ARCH} -p robolab
