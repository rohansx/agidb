//! `signatures.dat` — the mmap'd flat file that stores every HV.
//!
//! Layer 3 plumbing. Episode and SemanticAtom rows in redb hold a
//! `signature_offset` pointing into this file; the HVs themselves live
//! here so the POPCOUNT scan in `recall` can sweep a contiguous region
//! of memory without redb's B-tree overhead.
//!
//! File layout:
//!
//! ```text
//!   ┌────────────────────────────────────────────────┐
//!   │ MAGIC  (8B)  │ FORMAT_VERSION (4B) │ PAD (4B)  │  16 bytes header
//!   ├────────────────────────────────────────────────┤
//!   │ HV 0    (1024 bytes)                           │
//!   │ HV 1    (1024 bytes)                           │
//!   │ ...                                            │
//!   └────────────────────────────────────────────────┘
//! ```
//!
//! Offsets in rows are byte offsets from the start of the file (which
//! lets us mmap and index without an extra translation table).
//!
//! Phase 2 lands real `append`, `read`, and crash-safety code. The
//! public surface in this module is final; the implementations are
//! `todo!()` so phase 2 work has a concrete RED state.

use crate::error::Result;
use crate::hdc::HV;
use std::path::PathBuf;

/// Magic bytes at the start of every `signatures.dat` (`"sochdSIG"`).
pub const MAGIC: [u8; 8] = *b"sochdSIG";

/// On-disk format version. Bumped whenever the layout changes.
pub const FORMAT_VERSION: u32 = 1;

/// Size of the header, in bytes.
pub const HEADER_BYTES: u64 = 16;

/// Owning handle to a `signatures.dat`. Holds the file and its mmap.
pub struct SignatureFile {
    pub path: PathBuf,
    // Real fields land in phase 2 — keep the type opaque so callers
    // don't depend on the internal mmap layout.
    _marker: std::marker::PhantomData<()>,
}

impl SignatureFile {
    /// Open or create a `signatures.dat` at `path`. Verifies the magic
    /// + format version on existing files; writes them on new files.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let _ = path.into();
        todo!("phase 2: open signatures.dat with mmap; verify header")
    }

    /// Append a hypervector and return its byte offset. The offset is
    /// stable across reopens.
    pub fn append(&mut self, _hv: &HV) -> Result<u64> {
        todo!("phase 2: append hv, return offset, grow mmap if needed")
    }

    /// Read the HV stored at `offset`. Returns `SignatureOutOfBounds`
    /// if the offset is past the current end of file.
    pub fn read(&self, _offset: u64) -> Result<HV> {
        todo!("phase 2: bounds-check offset, copy 1024 bytes from mmap")
    }

    /// Current number of HVs stored in the file.
    pub fn len(&self) -> u64 {
        todo!("phase 2: derive from file size")
    }

    /// `true` iff no HVs have been appended yet.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Sync any pending mmap writes to disk. Called inside the redb
    /// commit hook so storage stays consistent.
    pub fn flush(&self) -> Result<()> {
        todo!("phase 2: mmap.flush()")
    }
}
