mod grep;
mod maps;

use std::{
    fmt::Debug,
    io::{BufRead, BufReader},
};

use clap::Parser;
use maps::MapsRecord;
use tracing::{debug, info, Level};

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

    /// Erase text
    #[clap(short, long)]
    erase: bool,

    /// If erase is enabled, use this value to erase the search text
    #[clap(short, long, default_value_t = 0x20)]
    erase_value: u8,

    /// Set log level to debug
    #[clap(short, long)]
    debug: bool,

    /// Set maximum region size. Regions larger than this size will not be searched.
    #[clap(short, long, default_value_t = 1_073_741_824)]
    max_region_size: usize,
}

fn ok_but_complain<T, E>(result: Result<T, E>) -> Option<T>
where
    E: Debug,
{
    match result {
        Ok(val) => Some(val),
        Err(err) => {
            debug!("Warning: {err:?}");
            None
        }
    }
}

fn setup_tracing(debug: bool) -> Result<(), tracing::dispatcher::SetGlobalDefaultError> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(if debug { Level::DEBUG } else { Level::INFO })
        .finish();
    tracing::subscriber::set_global_default(subscriber)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let pid = args.pid;
    let text = args.text;

    let erase = if args.erase {
        Some(args.erase_value)
    } else {
        None
    };

    setup_tracing(args.debug)?;

    info!("Attaching to PID {pid}, searching for {text}");

    let maps = std::fs::File::open(format!("/proc/{pid}/maps"))?;
    let buf_reader = BufReader::new(maps);

    buf_reader
        .lines()
        .filter_map(ok_but_complain)
        .map(maps::MapsRecord::try_from_line)
        .filter_map(ok_but_complain)
        .filter(|record| record.inode == 0)
        .filter(|record| record.perms.starts_with("rw"))
        .map(|record| grep::grep_memory_region(pid, record, &text, erase, args.max_region_size))
        .filter_map(ok_but_complain)
        .flatten()
        .for_each(|hit| println!("{hit}"));

    Ok(())
}
