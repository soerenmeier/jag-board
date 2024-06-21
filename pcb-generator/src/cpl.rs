
use crate::util::{create_build_dir, BUILD_DIR};

use std::fs;
use std::collections::HashMap;
use std::path::Path;

use clap::Parser;

use serde::{Serialize, Deserialize};

#[derive(Debug, Parser)]
pub struct Cpl {
	#[clap(long, default_value_t = true)]
	uses_comma: bool
}

pub fn cpl(args: Cpl) {
	let output = "./output";
	create_build_dir();

	let mut entry_csv = None;

	let read_dir = fs::read_dir(output).expect("could not read output");
	for entry in read_dir {
		let entry = entry.unwrap();
		let name = entry.file_name().into_string().expect("entry not utf8");

		if name.ends_with("top-pos.csv") {
			entry_csv = Some(
				fs::read_to_string(entry.path()).expect("failed to read csv")
			);
			break;
		}
	}

	let raw_csv = entry_csv.expect("did not find *top-pos.csv file");

	let delimiter = if args.uses_comma {
		b','
	} else {
		b';'
	};

	let reader = csv::ReaderBuilder::new()
		.flexible(true)
		.delimiter(delimiter)
		.from_reader(raw_csv.trim().as_bytes());

	let kicad_entries: Vec<KicadEntry> = reader.into_deserialize()
		.collect::<Result<_, _>>()
		.expect("failed to deserialize cpl");

	let rotation_table = read_rotation_table();

	let jlcpcb_entries: Vec<_> = kicad_entries.into_iter()
		.map(|e| {
			let entry = rotation_table.get(&e.designator)
				.unwrap_or(RotationEntry::DEFAULT);

			JlcpcbEntry {
				designator: e.designator,
				mid_x: format!("{}mm", e.pos_x + entry.pos_x),
				mid_y: format!("{}mm", e.pos_y + entry.pos_y),
				layer: format!("Top"),
				rotation: e.rotation + entry.rotation
			}
		})
		.collect();

	let cpl_path = format!("{}/cpl.csv", BUILD_DIR);
	let mut w = csv::Writer::from_path(&cpl_path).unwrap();
	for entry in jlcpcb_entries {
		w.serialize(entry).unwrap();
	}
	w.flush().unwrap();

	println!("created {}", cpl_path);
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct KicadEntry {
	#[serde(rename = "Ref")]
	designator: String,
	/// Comma separated designator list
	#[serde(rename = "Val")]
	#[allow(dead_code)]
	value: String,
	#[serde(rename = "Package")]
	package: String,
	// in mm
	#[serde(rename = "PosX")]
	pos_x: f32,
	// in mm
	#[serde(rename = "PosY")]
	pos_y: f32,
	// in deg
	#[serde(rename = "Rot")]
	rotation: f32,
	#[serde(rename = "Side")]
	side: String
}

#[derive(Debug, Serialize)]
struct JlcpcbEntry {
	#[serde(rename = "Designator")]
	designator: String,
	/// Comma separated designator list
	#[serde(rename = "Mid X")]
	mid_x: String,
	#[serde(rename = "Mid Y")]
	mid_y: String,
	#[serde(rename = "Layer")]
	layer: String,
	#[serde(rename = "Rotation")]
	rotation: f32
}

fn read_rotation_table() -> HashMap<String, RotationEntry> {
	let rotation_table_path = "./rotation-table.csv";
	if !Path::new(rotation_table_path).is_file() {
		return HashMap::new()
	}

	let reader = csv::ReaderBuilder::new()
		.flexible(true)
		.from_path(rotation_table_path)
		.expect("cannot read rotation table");

	let mut map = HashMap::new();

	for entry in reader.into_deserialize() {
		let entry: RotationEntry = entry.expect("failed to deserialize");
		let exists = map.insert(entry.designator.clone(), entry);
		assert!(exists.is_none(), "designator already exists");
	}

	map
}

#[derive(Debug, Default, Deserialize)]
struct RotationEntry {
	#[serde(rename = "Designator")]
	designator: String,
	#[serde(rename = "Rotation")]
	rotation: f32,
	#[serde(rename = "Pos X", default)]
	pos_x: f32,
	#[serde(rename = "Pos Y", default)]
	pos_y: f32
}

impl RotationEntry {
	const DEFAULT: &RotationEntry = &RotationEntry {
		designator: String::new(),
		rotation: 0f32,
		pos_x: 0f32,
		pos_y: 0f32
	};
}