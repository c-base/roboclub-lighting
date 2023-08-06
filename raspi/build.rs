use std::{env, path::PathBuf};

use prost_wkt_build::{FileDescriptorSet, Message};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let out = PathBuf::from(env::var("OUT_DIR").unwrap());
	let descriptor_file = out.join("descriptors.bin");

	tonic_build::configure()
		.build_client(false)
		.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
		.extern_path(".google.protobuf.Any", "::prost_wkt_types::Any")
		.extern_path(".google.protobuf.Timestamp", "::prost_wkt_types::Timestamp")
		.extern_path(".google.protobuf.Value", "::prost_wkt_types::Value")
		.extern_path(".google.protobuf.Struct", "::prost_wkt_types::Struct")
		.file_descriptor_set_path(&descriptor_file)
		.compile(&["proto/control.proto"], &["proto/"])?;

	let descriptor_bytes = std::fs::read(descriptor_file).unwrap();

	let descriptor = FileDescriptorSet::decode(&descriptor_bytes[..]).unwrap();

	prost_wkt_build::add_serde(out, descriptor);

	Ok(())

	// println!("cargo:rustc-link-search=./sysroot/usr/lib");
	// println!("cargo:rustc-link-search=./sysroot/usr/lib/pulseaudio");
	//
	// println!("cargo:rustc-link-lib=pulsecommon-12.2");
	// println!("cargo:rustc-link-lib=cap");
	// println!("cargo:rustc-link-lib=dbus-1");
}
