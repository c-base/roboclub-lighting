{
	description = "A basic flake with a shell";

	inputs = {
#		nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
		nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-22.05-darwin";
		#nixpkgs.url = "github:queezle42/nixpkgs?rev=2cdd8e45881664f40a74d0a729b3b56ddede6bf1";
#		nixpkgs.url = "github:Gaelan/nixpkgs?ref=e545700e7fcb8eb5116e657b337389f4e8a5ecaa";
		flake-utils.url = "github:numtide/flake-utils";
		rust-overlay.url = "github:oxalica/rust-overlay";
	};

	outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }: flake-utils.lib.eachDefaultSystem (system:
	let
		overlays = [
			(import rust-overlay)
		];
		#pkgsCross = import nixpkgs {
		pkgs = import nixpkgs {
			inherit system overlays;
#			inherit overlays;
			#crossSystem = "aarch64-linux";
		};
		pkgsCross = pkgs.pkgsCross.aarch64-multiplatform;

#		rustupToolchain = "nightly";

#		rustBuildTriple = {
#			x86_64-darwin = "x86_64-apple-darwin";
#		}.${system};

#		rustTargetTriple = "aarch64-unknown-linux-gnu";
	in { #pkgsCross.callPackage({ stdenv, mkShell, gcc, libgcc, rust-bin, cargo-make, pkg-config, libiconv, ... }:  {
#		inherit pkgsCross;

		devShells.default = pkgsCross.mkShell rec {
			nativeBuildInputs = with pkgs; [
#				pkgsCross.stdenv.cc
				#gcc
				#libgcc
				#(rust-bin.nightly.latest.minimal.override {
					#extensions = [ "rust-src" ];
					#targets = [ rustTargetTriple ];
				#})
				#cargo-make
				#pkg-config
			];

			buildInputs = with pkgsCross; [
#				libspa
#				libpipewire
				#gcc
				#libgcc
				#libiconv
			];

			# RUSTUP_HOME = "./.nix/.rustup";
			# CARGO_HOME = "./.nix/.cargo";
			# RUSTUP_HOME = "/Users/hrmny/.rustup";
			# CARGO_HOME = "/Users/hrmny/.cargo";

			# RUSTC_VERSION = rustupToolchain;
			# LIBCLANG_PATH= pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib pkgs.libiconv ];
			# CARGO_BUILD_TARGET = rustTargetTriple;
			#LIBCLANG_PATH = pkgsCross.lib.makeLibraryPath [ pkgsCross.llvmPackages_latest.libclang.lib ];
#      shellHook = ''
#        export PATH=$PATH:${CARGO_HOME}/bin
#      '';
			# export PATH=$PATH:${RUSTUP_HOME}/toolchains/${rustupToolchain}-${rustBuildTriple}/bin/
			# rustup component add rust-src
			# rustup target add ${rustTargetTriple}

			#CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${stdenv.cc.targetPrefix}cc";
#			CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${pkgsCross.stdenv.cc.targetPrefix}cc";
			# CARGO_TARGET_RISCV32IMC_ESP_ESPIDF_LINKER = "${pkgsCross.stdenv.cc.targetPrefix}cc";
		};
	});
	#}) {});
}
