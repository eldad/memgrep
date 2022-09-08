use std::os::unix::prelude::FileExt;

use super::MapsRecord;
use memmem::{Searcher, TwoWaySearcher};

const MAX_MEM: usize = 1_073_741_824; // 1GiB

pub fn grep_memory_region(
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
