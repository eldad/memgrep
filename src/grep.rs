use std::os::unix::prelude::FileExt;

use super::MapsRecord;
use memmem::{Searcher, TwoWaySearcher};
use thiserror::Error;
use tracing::{error, info};

#[derive(Error, Debug)]
pub enum GrepError {
    #[error("bad record (zero size or lower bound > upper bound)")]
    BadAddressSpaceRecord,
    #[error("max region size exceeded, region size: {}", .0)]
    MaxRegionSizeExceeded(usize),
    #[error("IO Error: {}", .0)]
    IOError(#[from] std::io::Error),
}

pub fn grep_memory_region(
    pid: i32,
    record: MapsRecord,
    text: &str,
    erase: bool,
    max_region_size: usize,
) -> Result<Option<String>, GrepError> {
    if record.address_upper <= record.address_lower {
        return Err(GrepError::BadAddressSpaceRecord);
    }
    let size = record.address_upper - record.address_lower - 1;

    if size > max_region_size {
        return Err(GrepError::MaxRegionSizeExceeded(size))?;
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

            match res {
                Err(err) => error!("erase error: {err}"),
                Ok(pos) => info!("Erased from [{record}] at position {pos}"),
            }
        }
    }

    Ok(result.map(|pos| format!("[{record}] @ position {pos:#18x} ({pos})")))
}
