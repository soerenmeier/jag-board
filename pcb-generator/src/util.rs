
use std::{fs, env};
use std::path::Path;
use std::process::Command;

pub const BUILD_DIR: &str = "./build";
const CONFIG_DIR: &str = ".config/pcb-generator";

pub fn create_build_dir() {
	if !Path::new(BUILD_DIR).is_dir() {
		fs::create_dir(BUILD_DIR).expect("failed to create build dir");
	}
}

pub fn config_dir() -> String {
	format!("{}/{}", env::var("HOME").unwrap(), CONFIG_DIR)
}

pub fn create_config_dir() -> String {
	let dir = config_dir();
	fs::create_dir_all(&dir).expect("could not create config dir");
	dir
}

pub fn create_zip(name: &str, path: &str) {
	let status = Command::new("zip")
		.arg("-r")
		.args(&[name, path])
		.status()
		.expect("could not load zip");

	if !status.success() {
		panic!("could not zip {}", path);
	}
}