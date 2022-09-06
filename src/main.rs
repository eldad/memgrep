mod maps;

use std::{
    io::{BufRead, BufReader},
    os::unix::prelude::FileExt,
};

use clap::Parser;
use maps::MapsRecord;
use memmem::{TwoWaySearcher, Searcher};

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

const MAX_MEM: usize = 1_073_741_824; // 1GiB

fn grep_memory_region(pid: i32, record: MapsRecord, text: &str) -> anyhow::Result<Option<String>> {
    let mem = std::fs::File::open(format!("/proc/{pid}/mem"))?;

    if record.address_upper <= record.address_lower {
        return Err(anyhow::anyhow!(
            "bad record (zero size or lower bound > upper bound)"
        ));
    }
    let size = record.address_upper - record.address_lower - 1;

    if size > MAX_MEM {
        return Err(anyhow::anyhow!(
            "too large"
        ));
    }

    let mut buf = vec![0; size];
    let bufslice = &mut buf[0..size - 1];

    let _n = mem.read_at(bufslice, record.address_lower as u64)?;

    let search = TwoWaySearcher::new(text.as_bytes());
    let result = search.search_in(bufslice);

    Ok(result.map(|pos| format!("record:{record:?}, pos: {pos}")))
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
        .filter(|record| record.perms.starts_with("rw"))
        .map(|record| grep_memory_region(pid, record, &text))
        .filter_map(Result::ok)
        .flatten()
        .for_each(|hit| println!("hit: {hit}"));

    Ok(())
}
