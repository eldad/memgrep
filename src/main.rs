mod maps;

use std::{
    io::{BufRead, BufReader},
    os::unix::prelude::FileExt, fmt::Debug,
};

use clap::Parser;
use maps::MapsRecord;
use memmem::{Searcher, TwoWaySearcher};

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

    /// Erase text with spaces (0x20)
    #[clap(short, long)]
    erase: bool,
}

const MAX_MEM: usize = 1_073_741_824; // 1GiB

fn grep_memory_region(
    pid: i32,
    record: MapsRecord,
    text: &str,
    erase: bool,
) -> anyhow::Result<Option<String>> {
    if record.address_upper <= record.address_lower {
        return Err(anyhow::anyhow!(
            "bad record (zero size or lower bound > upper bound)"
        ));
    }
    let size = record.address_upper - record.address_lower - 1;

    if size > MAX_MEM {
        return Err(anyhow::anyhow!("too large"));
    }

    let mem = std::fs::File::options()
        .read(true)
        .write(true)
        .open(format!("/proc/{pid}/mem"))?;

    let mut buf = vec![0; size];
    let bufslice = &mut buf[0..size - 1];

    let _n = mem.read_at(bufslice, record.address_lower as u64)?;

    let search = TwoWaySearcher::new(text.as_bytes());
    let result = search.search_in(bufslice);

    if let Some(pos) = result {
        if erase {
            let spaces = vec![0x20; text.len()];
            let offset = record.address_lower + pos;
            let res = mem.write_at(&spaces, offset as u64);
            println!("erase: {res:#?}");
        }
    }

    Ok(result.map(|pos| format!("record:{record:?}, pos: {pos}")))
}

fn ok_but_complain<T, E>(result: Result<T, E>) -> Option<T>
where
    E: Debug
{
    match result {
        Ok(val) => Some(val),
        Err(err) => {
            eprintln!("Warning: {err:?}");
            None
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let pid = args.pid;
    let text = args.text;
    let erase = args.erase;

    println!("Attaching to PID {pid}, searching for {text}");

    let maps = std::fs::File::open(format!("/proc/{pid}/maps"))?;
    let buf_reader = BufReader::new(maps);

    buf_reader
        .lines()
        .filter_map(ok_but_complain)
        .map(maps::MapsRecord::try_from_line)
        .filter_map(ok_but_complain)
        .filter(|record| record.inode == 0)
        .filter(|record| record.perms.starts_with("rw"))
        .map(|record| grep_memory_region(pid, record, &text, erase))
        .filter_map(ok_but_complain)
        .flatten()
        .for_each(|hit| println!("hit: {hit}"));

    Ok(())
}
