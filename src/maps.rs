//! NOTE: Documentation sourced from the `proc(5)` manpage.

use std::{fmt::Display, num::ParseIntError};

use thiserror::Error;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum MapsPath {
    Stack,
    Heap,

    /// vDSO (virtual dynamic shared object). See `vdso(7)`.
    VDSO,

    /// vvar page
    VVar,

    /// virtual syscall page
    VSyscall,

    /// Anonymous mapping as obtained via `mmap(2)`.
    /// There is no easy way to coordinate this back to a process's source, short of running it through `gdb(1)`, `strace(1)`, or similar.
    Anonymous,

    /// File
    #[allow(unused)]
    File(String),
}

impl From<Option<&str>> for MapsPath {
    fn from(path: Option<&str>) -> Self {
        match path {
            None => Self::Anonymous,
            Some(path) => match path {
                "[stack]" => Self::Stack,
                "[heap]" => Self::Heap,
                "[vdso]" => Self::VDSO,
                "[vvar]" => Self::VVar,
                "[vsyscall]" => Self::VSyscall,
                _ => Self::File(path.to_owned()),
            },
        }
    }
}

/// Represents one record (one row) from `/proc/[pid]/maps`
#[derive(Debug)]
pub struct MapsRecord {
    /// Lower bound of the address space in the process that the mapping occupies.
    pub address_lower: usize,

    /// Upper bound of the address space in the process that the mapping occupies.
    pub address_upper: usize,

    /// ```
    /// r/- = read
    /// w/- = write
    /// x/- = execute
    /// s/p = shared / private (copy on write)
    /// ```
    pub perms: String,

    /// Offset into the file.
    #[allow(unused)]
    pub offset: usize,

    /// Device major [12 bits]:minor [20 bits]
    #[allow(unused)]
    pub dev: String,

    /// inode on that device. 0 indicates no inode is associated with the memory region as would be the case with BSS (uninitialized data).
    pub inode: u64,

    /// The pathname field will usually be the file that is backing the mapping. For ELF files, you can easily coordinate
    /// with the offset field by looking at the Offset field in the ELF program headers (readelf -l).
    ///
    /// There are additional helpful pseudo-paths (see `MapsPath`).
    ///
    /// If the pathname field is blank, this is an anonymous mapping as obtained via mmap(2).  There is no easy way to coor‐
    /// dinate this back to a process's source, short of running it through gdb(1), strace(1), or similar.
    ///
    /// pathname is shown unescaped except for newline characters, which are replaced with an octal escape sequence.   As  a
    /// result,  it  is not possible to determine whether the original pathname contained a newline character or the literal
    /// \012 character sequence.
    ///
    /// If the mapping is file-backed and the file has been deleted, the string " (deleted)" is appended  to  the  pathname.
    /// Note that this is ambiguous too.
    ///
    /// Under Linux 2.0, there is no field giving pathname.
    pub path: MapsPath,
}

#[derive(Error, Debug)]
pub enum MapsError {
    #[error("Missing field {}", .0)]
    MissingField(String),
    #[error("Address space parse error")]
    AddressSpaceParseError,
    #[error("Parsing error")]
    ParsingError(#[from] ParseIntError),
}

macro_rules! next {
    ($iter:ident, $field:literal) => {
        $iter.next().ok_or_else(|| MapsError::MissingField($field.to_owned()))
    };
}

macro_rules! from_hex {
    ($t:ty, $v:ident) => {
        <$t>::from_str_radix($v, 16)
    };
}

impl MapsRecord {
    pub fn try_from_line<T>(line: T) -> Result<Self, MapsError>
    where
        String: std::convert::From<T>,
    {
        let line_ref: String = line.into();
        let mut iter = line_ref.split_ascii_whitespace();

        let (address_lower, address_upper) = next!(iter, "address_space")?
            .split_once('-')
            .ok_or(MapsError::AddressSpaceParseError)?;
        let perms = next!(iter, "perms")?;
        let offset = next!(iter, "offset")?;
        let dev = next!(iter, "dev")?;
        let inode = next!(iter, "inode")?;
        let path_name = iter.next();

        Ok(Self {
            address_lower: from_hex!(usize, address_lower)?,
            address_upper: from_hex!(usize, address_upper)?,
            perms: perms.to_owned(),
            offset: from_hex!(usize, offset)?,
            dev: dev.to_owned(),
            inode: from_hex!(u64, inode)?,
            path: path_name.into(),
        })
    }
}

impl Display for MapsRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#018x}-{:#018x}", self.address_lower, self.address_upper)?;
        write!(f, " <{:?}>", self.path)?;
        Ok(())
    }
}
