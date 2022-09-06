mod maps;

use std::io::{BufRead, BufReader, Read};

use clap::Parser;

/// Memory Grep
#[derive(Parser, Debug)]
#[clap(author = "Eldad Zack <eldad@fogrefinery.com>", version, about, long_about = None)]
struct Args {
    /// PID
    #[clap(short, long)]
    pid: i32,

    /// Search text
    #[clap()]
    text: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let pid = args.pid;
    let text = args.text;

    println!("Attaching to PID {pid}, searching for {text}");

    let maps = std::fs::File::open(format!("/proc/{pid}/maps"))?;
    let buf_reader = BufReader::new(maps);

    buf_reader
        .lines()
        .filter_map(Result::ok)
        .map(maps::MapsRecord::try_from_line)
        .filter_map(Result::ok)
        .filter(|record| record.inode == 0)
        .for_each(|record| println!("{record:#?}"));

    Ok(())
}
