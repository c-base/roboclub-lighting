# Base image for rapsberrypi 3 target
FROM rustembedded/cross:armv7-unknown-linux-gnueabihf

# Install libdbus libraries and pkg-config
RUN dpkg --add-architecture armhf && \
	    apt-get update && \
	    apt-get install --assume-yes libpulse-dev libpulse-dev:armhf pkg-config
