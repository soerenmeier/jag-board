
use crate::util::{BUILD_DIR, create_build_dir};
use crate::partslist::find_in_parts_list;

use std::fs;

use clap::Parser;

use serde::{Serialize, Deserialize};

#[derive(Debug, Parser)]
pub struct Bom {
	#[clap(long, default_value_t = true)]
	uses_comma: bool
}

pub fn bom(args: Bom) {
	create_build_dir();

	let raw_csv = fs::read_to_string("./bom.csv").expect("failed to read csv");

	let delimiter = if args.uses_comma {
		b','
	} else {
		b';'
	};

	let reader = csv::ReaderBuilder::new()
		.flexible(true)
		.delimiter(delimiter)
		.from_reader(raw_csv.trim().as_bytes());

	let custom_entries: Vec<CustomEntry> = reader.into_deserialize()
		.collect::<Result<_, _>>()
		.expect("failed to deserialize bom");

	let bom_path = format!("{}/bom.csv", BUILD_DIR);
	let mut w = csv::Writer::from_path(&bom_path).unwrap();

	let ids: Vec<_> = custom_entries.iter()
		.map(|e| e.jlcpcb_part.clone())
		.collect();

	let parts = find_in_parts_list(|p| ids.iter().any(|i| i == p.lcsc.trim()));

	// convert to JlcpcbEntry
	for entry in custom_entries {
		let Some(part) = parts.iter()
			.find(|p| p.lcsc.trim() == entry.jlcpcb_part) else
		{
			panic!("could not find {:?}", entry);
		};

		w.serialize(JlcpcbEntry {
			comment: part.desc.clone(),
			designators: entry.designators,
			footprint: part.package.clone(),
			jlcpcb_part: entry.jlcpcb_part
		}).unwrap();
	}
	w.flush().unwrap();

	println!("written to {:?}", bom_path);
}

#[derive(Debug, Deserialize)]
struct CustomEntry {
	/// Comma separated designator list
	#[serde(rename = "Designator")]
	designators: String,
	#[serde(rename = "JLCPCB Part")]
	jlcpcb_part: String
}

#[derive(Debug, Serialize)]
struct JlcpcbEntry {
	#[serde(rename = "Comment")]
	comment: String,
	/// Comma separated designator list
	#[serde(rename = "Designator")]
	designators: String,
	#[serde(rename = "Footprint")]
	footprint: String,
	#[serde(rename = "JLCPCB Part")]
	jlcpcb_part: String
}