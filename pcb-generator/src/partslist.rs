use crate::util::{config_dir, create_config_dir};

use std::io;
use std::fs::{File};

use clap::Parser;

use serde::{Deserialize};

use encoding_rs_io::DecodeReaderBytesBuilder;
use reqwest::blocking::{Client};


const PARTS_LIST_FILE: &str = "jlcpcb-parts-list.csv";

fn parts_list_path() -> String {
	format!("{}/{}", config_dir(), PARTS_LIST_FILE)
}


#[derive(Debug, Parser)]
pub struct DownloadPartsList {}

pub fn download_parts_list(_args: DownloadPartsList) {
	let client = Client::new();
	let resp = client
		.get("https://jlcpcb.com/componentSearch/uploadComponentInfo")
		.send().expect("failed to request parts list");

	let config_dir = create_config_dir();
	let parts_list_path = format!("{}/{}", config_dir, PARTS_LIST_FILE);

	// todo this is not optimal but i didn't find an encoding library
	// implementing io::Write in a short time
	// so let's just waste memory

	let mut reader = DecodeReaderBytesBuilder::new()
		.encoding(Some(encoding_rs::GB18030))
		.build(resp);

	// url: 
	let mut csv_file = File::create(&parts_list_path)
		.expect("could not create parts list");
	io::copy(&mut reader, &mut csv_file).expect("could not write parts list");

	println!("parts list written to {:?}", parts_list_path);
}

#[derive(Debug, Parser)]
pub struct SearchPartsList {
	#[clap(long)]
	pub id: Option<String>,
	#[clap(long)]
	pub cat: Option<String>
}

impl SearchPartsList {
	fn matches(&self, part: &Part) -> bool {
		if let Some(id) = &self.id {
			if part.lcsc.trim() == id.trim() {
				return true
			}
		}

		if let Some(cat) = &self.cat {
			let a = part.first_cat.to_lowercase();
			let b = part.second_cat.to_lowercase();
			if a.trim().contains(cat) || b.trim().contains(cat) {
				return true
			}
		}

		false
	}
}

pub fn search_parts_list(mut args: SearchPartsList) {
	if let Some(cat) = &mut args.cat {
		*cat = cat.trim().to_lowercase();
	}

	let list = find_in_parts_list(|part| { args.matches(part) });

	for part in list {
		println!("part: {:?}", part);
	}
}

#[allow(dead_code)]
pub fn read_parts_list() -> PartsList {
	let file = File::open(parts_list_path())
		.expect("could not open parts list");
	let reader = csv::ReaderBuilder::new()
		.flexible(true)
		// .delimiter(delimiter)
		.from_reader(file);

	PartsList {
		inner: reader.into_deserialize()
			.collect::<Result<_, _>>()
			.expect("failed to deserialize parts list")
	}
}

pub fn find_in_parts_list<F>(f: F) -> Vec<Part>
where F: Fn(&Part) -> bool {
	let file = File::open(parts_list_path())
		.expect("could not open parts list");
	let reader = csv::ReaderBuilder::new()
		.flexible(true)
		// .delimiter(delimiter)
		.from_reader(file);

	let mut l = vec![];
	for part in reader.into_deserialize() {
		let p: Part = part.expect("could not deserialize part");

		if f(&p) {
			l.push(p);
		}
	}

	l
}

// probably not the best idear to store the entire list in memory
// but well, I have enough :)
#[allow(dead_code)]
pub struct PartsList {
	inner: Vec<Part>
}

/// Price,Stock
#[derive(Debug, Clone, Deserialize)]
pub struct Part {
	#[serde(rename = "LCSC Part")]
	pub lcsc: String,
	#[serde(rename = "First Category")]
	pub first_cat: String,
	#[serde(rename = "Second Category")]
	pub second_cat: String,
	#[serde(rename = "MFR.Part")]
	pub mfr_part: String,
	#[serde(rename = "Package")]
	pub package: String,
	#[serde(rename = "Solder Joint")]
	pub solder_joint: String,
	#[serde(rename = "Manufacturer")]
	pub manufacturer: String,
	#[serde(rename = "Library Type")]
	pub library_type: String,
	#[serde(rename = "Description")]
	pub desc: String,
	#[serde(rename = "Datasheet")]
	pub datasheet: String,
	#[serde(rename = "Price")]
	pub price: String,
	#[serde(rename = "Stock")]
	pub stock: usize
}