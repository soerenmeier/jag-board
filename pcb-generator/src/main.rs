
mod util;
mod gerber;
mod bom;
mod cpl;
mod partslist;

use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
	#[clap(subcommand)]
	subcmd: SubCommand
}

#[derive(Debug, Parser)]
enum SubCommand {
	Gerber(gerber::Gerber),
	Bom(bom::Bom),
	Cpl(cpl::Cpl),
	DownloadPartsList(partslist::DownloadPartsList),
	SearchPartsList(partslist::SearchPartsList)
}



fn main() {
	let args = Args::parse();

	match args.subcmd {
		SubCommand::Gerber(args) => {
			gerber::gerber(args);
		},
		SubCommand::Bom(args) => {
			bom::bom(args);
		},
		SubCommand::Cpl(args) => {
			cpl::cpl(args);
		},
		SubCommand::DownloadPartsList(args) => {
			partslist::download_parts_list(args);
		},
		SubCommand::SearchPartsList(args) => {
			partslist::search_parts_list(args);
		}
	}
}

