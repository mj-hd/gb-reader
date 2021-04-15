use anyhow::Result;
use clap::{AppSettings, Clap};
use gb_reader::{board::CubicStyleBoard, mbc::new_mbc_reader};
use std::fs::File;
use std::io::{Read as _, Write as _};

#[derive(Clap)]
#[clap(version = "0.1.0", author = "mjhd <mjhd.devlion@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Read(Read),
}

#[derive(Clap)]
struct Read {
    #[clap(short, long)]
    output: String,
}

fn read_rom(output: String) -> Result<()> {
    let mut board = CubicStyleBoard::new()?;
    let mut reader = new_mbc_reader(&mut board)?;
    let mut file = File::create(output)?;

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    file.write_all(&buffer)?;
    file.flush()?;

    Ok(())
}

fn main() {
    let opts: Opts = Opts::parse();

    let result = match opts.subcmd {
        SubCommand::Read(t) => read_rom(t.output),
    };

    result.unwrap();
}
