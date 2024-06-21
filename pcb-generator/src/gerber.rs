
use crate::util::{self, create_zip, create_build_dir};

use std::fs;
use std::path::Path;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Gerber {
}

pub fn gerber(_args: Gerber) {
	let output = "./output";
	let tmp_dir = "./build_tmp";
	if Path::new(tmp_dir).is_dir() {
		panic!("delete build_tmp dir first");
	}
	fs::create_dir(tmp_dir).expect("failed to create build dir");
	create_build_dir();

	let read_dir = fs::read_dir(output).expect("could not read output");
	for entry in read_dir {
		let entry = entry.unwrap();
		let name = entry.file_name().into_string().expect("entry not utf8");

		if name.ends_with(".gbr") || name.ends_with(".drl") {
			fs::copy(entry.path(), Path::new(tmp_dir).join(name))
					.expect("failed to copy from output dir");
		}
	}

	create_zip(&format!("{}/gerber.zip", util::BUILD_DIR), tmp_dir);

	fs::remove_dir_all(tmp_dir).expect("could not clean tmp_dir");

	println!("created ./build/gerber.zip");
}

