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
//!   ┌──────────────────────────────────────────────────────────┐
//!   │ HEADER (32 bytes)                                        │
//!   │   [ 0.. 8] MAGIC = "agidbSIG"                            │
//!   │   [ 8..12] FORMAT_VERSION  (u32 LE)                      │
//!   │   [12..16] reserved (zero)                               │
//!   │   [16..24] next_offset     (u64 LE)                      │
//!   │   [24..32] reserved (zero)                               │
//!   ├──────────────────────────────────────────────────────────┤
//!   │ HV 0    (1024 bytes)                                     │
//!   │ HV 1    (1024 bytes)                                     │
//!   │ ...                                                      │
//!   └──────────────────────────────────────────────────────────┘
//! ```
//!
//! The file is grown in 1 MiB chunks; the logical end (`next_offset`)
//! is stored in the header so a reopen knows the true HV count even
//! when the file has unused tail capacity.

use crate::error::{Result, AgidbError};
use crate::hdc::{D_BYTES, HV};
use memmap2::MmapMut;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;

/// Magic bytes at the start of every `signatures.dat` (`"agidbSIG"`).
pub const MAGIC: [u8; 8] = *b"agidbSIG";

/// On-disk format version. Bumped whenever the layout changes.
pub const FORMAT_VERSION: u32 = 1;

/// Size of the header in bytes. 32 leaves room for two u64 fields after
/// MAGIC + version + pad without crowding.
pub const HEADER_BYTES: u64 = 32;

/// Byte offset of the `next_offset` slot within the header.
const NEXT_OFFSET_FIELD: usize = 16;

/// HV slot size as `u64` (so arithmetic in the file domain doesn't
/// constantly cast).
const HV_SLOT: u64 = D_BYTES as u64;

/// File grows in 1 MiB chunks once the mmap fills up.
const GROWTH_CHUNK: u64 = 1024 * 1024;

/// Initial file size for a brand-new `signatures.dat`. Header + one
/// growth chunk so the first ~1000 appends don't trigger a remap.
const INITIAL_FILE_SIZE: u64 = HEADER_BYTES + GROWTH_CHUNK;

/// Owning handle to a `signatures.dat`. Holds the file and its mmap.
pub struct SignatureFile {
    pub path: PathBuf,
    file: File,
    // Wrapped in Option so we can drop the mmap before resizing the
    // file. Always `Some` outside of an in-progress `grow`.
    mmap: Option<MmapMut>,
}

impl SignatureFile {
    /// Open or create a `signatures.dat` at `path`. Verifies the magic
    /// + format version on existing files; writes them on new files.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;

        let metadata = file.metadata()?;
        let file_len = metadata.len();

        if file_len == 0 {
            // Brand-new file. Write header + grow to initial size.
            file.set_len(INITIAL_FILE_SIZE)?;
            let mut mmap = unsafe { MmapMut::map_mut(&file)? };
            mmap[0..8].copy_from_slice(&MAGIC);
            mmap[8..12].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
            // next_offset starts at HEADER_BYTES.
            mmap[NEXT_OFFSET_FIELD..NEXT_OFFSET_FIELD + 8]
                .copy_from_slice(&HEADER_BYTES.to_le_bytes());
            mmap.flush()?;
            Ok(Self {
                path,
                file,
                mmap: Some(mmap),
            })
        } else if file_len < HEADER_BYTES {
            // Truncated / corrupt file.
            Err(AgidbError::CorruptSignature { offset: 0, path })
        } else {
            // Existing file. Verify header.
            let mmap = unsafe { MmapMut::map_mut(&file)? };
            if mmap[0..8] != MAGIC {
                return Err(AgidbError::CorruptSignature { offset: 0, path });
            }
            let version = u32::from_le_bytes(mmap[8..12].try_into().unwrap());
            if version != FORMAT_VERSION {
                return Err(AgidbError::FormatVersion {
                    got: version,
                    expected: FORMAT_VERSION,
                });
            }
            Ok(Self {
                path,
                file,
                mmap: Some(mmap),
            })
        }
    }

    /// Append a hypervector and return its byte offset. The offset is
    /// stable across reopens because `next_offset` is persisted in the
    /// header.
    pub fn append(&mut self, hv: &HV) -> Result<u64> {
        let offset = self.next_offset();
        let end = offset + HV_SLOT;
        let mmap_len = self.mmap_ref().len() as u64;
        if end > mmap_len {
            // Grow the file. Double-or-need-it, whichever is larger.
            let new_size = end.max(mmap_len.saturating_mul(2));
            self.grow(new_size)?;
        }
        let mmap = self.mmap_mut();
        mmap[offset as usize..end as usize].copy_from_slice(&hv.0);
        // Persist the new next_offset in the header — every append is
        // self-describing on disk so a crash mid-batch leaves a
        // recoverable state.
        mmap[NEXT_OFFSET_FIELD..NEXT_OFFSET_FIELD + 8].copy_from_slice(&end.to_le_bytes());
        Ok(offset)
    }

    /// Read the HV stored at `offset`. Returns `SignatureOutOfBounds`
    /// if the offset is past the logical end of file (not the mmap
    /// length — tail capacity is invisible to readers).
    pub fn read(&self, offset: u64) -> Result<HV> {
        let next = self.next_offset();
        if offset < HEADER_BYTES || offset + HV_SLOT > next {
            return Err(AgidbError::SignatureOutOfBounds { offset, len: next });
        }
        let mmap = self.mmap_ref();
        let mut bytes = [0u8; D_BYTES];
        bytes.copy_from_slice(&mmap[offset as usize..(offset + HV_SLOT) as usize]);
        Ok(HV(bytes))
    }

    /// Current number of HVs stored in the file.
    pub fn len(&self) -> u64 {
        (self.next_offset() - HEADER_BYTES) / HV_SLOT
    }

    /// `true` iff no HVs have been appended yet.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Sync any pending mmap writes to disk. Called inside the redb
    /// commit hook so storage stays consistent.
    pub fn flush(&self) -> Result<()> {
        self.mmap_ref().flush()?;
        self.file.sync_all()?;
        Ok(())
    }

    // -- internal helpers -------------------------------------------------

    fn mmap_ref(&self) -> &MmapMut {
        self.mmap
            .as_ref()
            .expect("SignatureFile mmap missing — only None during in-progress grow")
    }

    fn mmap_mut(&mut self) -> &mut MmapMut {
        self.mmap
            .as_mut()
            .expect("SignatureFile mmap missing — only None during in-progress grow")
    }

    fn next_offset(&self) -> u64 {
        let mmap = self.mmap_ref();
        u64::from_le_bytes(
            mmap[NEXT_OFFSET_FIELD..NEXT_OFFSET_FIELD + 8]
                .try_into()
                .unwrap(),
        )
    }

    fn grow(&mut self, new_size: u64) -> Result<()> {
        // Drop the existing mmap so the kernel releases the mapping;
        // set_len under a live mmap is undefined behavior on some
        // platforms.
        let _ = self.mmap.take();
        self.file.set_len(new_size)?;
        self.mmap = Some(unsafe { MmapMut::map_mut(&self.file)? });
        Ok(())
    }
}

impl Drop for SignatureFile {
    fn drop(&mut self) {
        // Best-effort flush on drop so the test harness's "drop store →
        // reopen" pattern doesn't lose the last batch of writes.
        // Failures are ignored: there's no error channel from Drop, and
        // an explicit `flush()` would have surfaced any real I/O issue.
        if let Some(m) = &self.mmap {
            let _ = m.flush();
            let _ = self.file.sync_all();
        }
    }
}
