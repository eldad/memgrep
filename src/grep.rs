use std::os::unix::prelude::FileExt;

use super::MapsRecord;
use memmem::{Searcher, TwoWaySearcher};
use thiserror::Error;
use tracing::{error, info};

#[derive(Error, Debug)]
pub enum GrepError {
    #[error("Bad record (zero size or lower bound > upper bound)")]
    BadAddressSpaceRecord,
    #[error("Max region size exceeded, region size: {}", .0)]
    MaxRegionSizeExceeded(usize),
    #[error("IO Error: {}", .0)]
    IOError(#[from] std::io::Error),
    #[error("Memory read resulted in less bytes than expected (region size: {size}, read: {bytes_read})")]
    MemoryReadBytesMismatch { size: usize, bytes_read: usize },
}

pub fn grep_memory_region(
    pid: i32,
    record: MapsRecord,
    text: &[u8],
    erase: Option<u8>,
    max_region_size: usize,
) -> Result<Option<(MapsRecord, usize)>, GrepError> {
    if record.address_upper <= record.address_lower {
        return Err(GrepError::BadAddressSpaceRecord);
    }
    let size = record.address_upper - record.address_lower - 1;

    if size > max_region_size {
        Err(GrepError::MaxRegionSizeExceeded(size))?;
    }

    let mem = std::fs::File::options()
        .read(true)
        .write(true)
        .open(format!("/proc/{pid}/mem"))?;

    let mut buf = vec![0; size];

    let n = mem.read_at(&mut buf, record.address_lower as u64)?;
    if n != size {
        return Err(GrepError::MemoryReadBytesMismatch { size, bytes_read: n });
    }

    let search = TwoWaySearcher::new(text);
    let mut result = search.search_in(&buf);

    let mut cursor = 0;

    if let Some(erase_val) = erase {
        let eraser = vec![erase_val; text.len()];

        while let Some(pos) = result {
            let offset = record.address_lower + cursor + pos;
            let res = mem.write_at(&eraser, offset as u64);

            match res {
                Err(err) => error!("erase error: {err}"),
                Ok(_) => info!("Erased from [{record}] at position {pos}"),
            }

            cursor += pos + text.len();
            result = search.search_in(&buf[cursor..]);
        }
    }

    Ok(result.map(|pos| (record, pos)))
}
