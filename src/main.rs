use anyhow::Result;
use clap::{AppSettings, Clap};
use gb_reader::{board::CubicStyleBoard, mbc::new_mbc_reader, mbc::new_repl_mbc_reader};
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{Read as _, Write as _};
use std::str;

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

    #[clap(short, long)]
    repl: bool,
}

fn read_rom(output: String, repl: bool) -> Result<()> {
    println!("[0/4] 拡張ボードの初期化中...");
    let mut board = CubicStyleBoard::new()?;

    println!("[1/4] ROMヘッダの解析中...");
    let (mut reader, header) = if repl {
        new_repl_mbc_reader(&mut board)?
    } else {
        new_mbc_reader(&mut board)?
    };

    println!(
        "タイトル: {}, MBC: {:?}, ROMサイズ: {}",
        str::from_utf8(&header.title[..]).unwrap_or("ERR"),
        header.mbc_type,
        HumanBytes(header.rom_size as u64)
    );

    println!("[2/4] 出力ファイルの作成中...");
    let mut file = File::create(output)?;

    let total = reader.size();

    let reading = ProgressBar::new(total as u64);
    reading.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}({eta})] {msg} [{bar:.cyan/blue}] {bytes}/{total_bytes}")
            .progress_chars("#>-"),
    );

    println!("[3/4] ROM読み込み中...");

    loop {
        let mut buffer = [0; 0x0100];

        let size = reader.read(&mut buffer)?;

        if size == 0 {
            break;
        }

        file.write(&buffer[0..size])?;

        reading.inc(size as u64);
        reading.set_message(&reader.status());
    }

    println!("[4/4] 仕上げ中...");
    file.flush()?;

    println!("完了！");
    reading.finish_and_clear();

    Ok(())
}

fn main() {
    let opts: Opts = Opts::parse();

    let result = match opts.subcmd {
        SubCommand::Read(t) => read_rom(t.output, t.repl),
    };

    result.unwrap();
}
