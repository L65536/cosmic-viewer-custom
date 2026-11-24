// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    loader::decode_image_bytes,
    nav::{is_supported_image, natural_cmp},
};
use cosmic::widget::image::Handle;
use image::DynamicImage;
use std::{
    fs::File,
    io::{Read, Seek},
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    sync::{Arc, Condvar, Mutex},
};
use thiserror::Error;
use tokio::task::spawn_blocking;
use zip::ZipArchive;

pub const MAX_ARCHIVE_ENTRY_SIZE: u64 = 64 * 1024 * 1024;
const ZIP_READER_COUNT: usize = 2;
const MAX_QUEUED_ARCHIVE_READS: usize = 8;
static ARCHIVE_READS_IN_FLIGHT: AtomicUsize = AtomicUsize::new(0);
static ACTIVE_ARCHIVE_DECODES: Mutex<usize> = Mutex::new(0);
static ARCHIVE_DECODES_READY: Condvar = Condvar::new();

#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error("Failed to open ZIP archive: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid ZIP archive: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Archive member {name} is too large ({size} bytes; limit is {limit} bytes)")]
    EntryTooLarge { name: String, size: u64, limit: u64 },
    #[error("Archive read pool is unavailable")]
    ReaderPoolUnavailable,
    #[error("Archive read queue is full; try again")]
    ReadQueueFull,
    #[error("Failed to decode archive image: {0}")]
    Decode(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    pub name: String,
    pub size: u64,
    index: usize,
}

#[derive(Clone)]
pub struct ArchiveCatalog {
    path: PathBuf,
    entries: Vec<ArchiveEntry>,
    readers: Arc<ReaderPool>,
}

impl std::fmt::Debug for ArchiveCatalog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArchiveCatalog")
            .field("path", &self.path)
            .field("entries", &self.entries.len())
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub struct LoadedArchiveImage {
    pub index: usize,
    pub handle: Handle,
    pub image: DynamicImage,
    pub width: u32,
    pub height: u32,
}

impl std::fmt::Debug for LoadedArchiveImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedArchiveImage")
            .field("index", &self.index)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
}

struct ReaderPool {
    available: Mutex<Vec<ZipArchive<File>>>,
    ready: Condvar,
}

struct ArchiveReadPermit<'a> {
    count: &'a AtomicUsize,
}

struct ArchiveDecodePermit;

impl Drop for ArchiveReadPermit<'_> {
    fn drop(&mut self) {
        self.count.fetch_sub(1, Ordering::AcqRel);
    }
}

impl Drop for ArchiveDecodePermit {
    fn drop(&mut self) {
        let mut active = ACTIVE_ARCHIVE_DECODES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *active = active.saturating_sub(1);
        ARCHIVE_DECODES_READY.notify_one();
    }
}

fn acquire_archive_decode_slot() -> Result<ArchiveDecodePermit, ArchiveError> {
    let mut active = ACTIVE_ARCHIVE_DECODES
        .lock()
        .map_err(|_| ArchiveError::ReaderPoolUnavailable)?;
    while *active >= ZIP_READER_COUNT {
        active = ARCHIVE_DECODES_READY
            .wait(active)
            .map_err(|_| ArchiveError::ReaderPoolUnavailable)?;
    }
    *active += 1;
    Ok(ArchiveDecodePermit)
}

fn reserve_archive_read(
    count: &AtomicUsize,
    limit: usize,
) -> Result<ArchiveReadPermit<'_>, ArchiveError> {
    let mut in_flight = count.load(Ordering::Acquire);
    loop {
        if in_flight >= limit {
            return Err(ArchiveError::ReadQueueFull);
        }
        match count.compare_exchange_weak(
            in_flight,
            in_flight + 1,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => return Ok(ArchiveReadPermit { count }),
            Err(current) => in_flight = current,
        }
    }
}

impl ReaderPool {
    fn open(path: &Path) -> Result<Self, ArchiveError> {
        let mut readers = Vec::with_capacity(ZIP_READER_COUNT);
        for _ in 0..ZIP_READER_COUNT {
            readers.push(ZipArchive::new(File::open(path)?)?);
        }

        Ok(Self {
            available: Mutex::new(readers),
            ready: Condvar::new(),
        })
    }

    fn acquire(&self) -> Result<ZipArchive<File>, ArchiveError> {
        let mut available = self
            .available
            .lock()
            .map_err(|_| ArchiveError::ReaderPoolUnavailable)?;
        loop {
            if let Some(reader) = available.pop() {
                return Ok(reader);
            }
            available = self
                .ready
                .wait(available)
                .map_err(|_| ArchiveError::ReaderPoolUnavailable)?;
        }
    }

    fn release(&self, reader: ZipArchive<File>) -> Result<(), ArchiveError> {
        self.available
            .lock()
            .map_err(|_| ArchiveError::ReaderPoolUnavailable)?
            .push(reader);
        self.ready.notify_one();
        Ok(())
    }

    fn read_member(&self, entry: &ArchiveEntry) -> Result<Vec<u8>, ArchiveError> {
        let mut reader = self.acquire()?;
        let result = read_member(
            &mut reader,
            entry.index,
            &entry.name,
            MAX_ARCHIVE_ENTRY_SIZE,
        );
        self.release(reader)?;
        result
    }
}

impl ArchiveCatalog {
    /// Index a ZIP archive off the async executor's thread.
    pub async fn open(path: PathBuf) -> Result<Self, ArchiveError> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        spawn_blocking(move || {
            let _ = tx.send(Self::open_sync(path));
        });
        rx.await.map_err(|_| ArchiveError::ReaderPoolUnavailable)?
    }

    fn open_sync(path: PathBuf) -> Result<Self, ArchiveError> {
        let readers = Arc::new(ReaderPool::open(&path)?);
        let mut reader = readers.acquire()?;
        let mut entries = Vec::new();

        for index in 0..reader.len() {
            let Ok(file) = reader.by_index(index) else {
                continue;
            };
            if file.is_dir() {
                continue;
            }

            let name = file.name().to_owned();
            if is_supported_image(Path::new(&name)) {
                entries.push(ArchiveEntry {
                    name,
                    size: file.size(),
                    index,
                });
            }
        }

        readers.release(reader)?;
        entries.sort_by(|a, b| natural_cmp(&a.name, &b.name).then_with(|| a.name.cmp(&b.name)));

        Ok(Self {
            path,
            entries,
            readers,
        })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    /// Read and decode one indexed member without extracting it to disk.
    pub async fn load_image(&self, index: usize) -> Result<LoadedArchiveImage, ArchiveError> {
        let entry =
            self.entries.get(index).cloned().ok_or_else(|| {
                ArchiveError::Decode(format!("invalid archive image index {index}"))
            })?;
        let permit = reserve_archive_read(&ARCHIVE_READS_IN_FLIGHT, MAX_QUEUED_ARCHIVE_READS)?;
        let readers = Arc::clone(&self.readers);
        let (tx, rx) = tokio::sync::oneshot::channel();

        spawn_blocking(move || {
            let _permit = permit;
            let result = acquire_archive_decode_slot().and_then(|_decode_permit| {
                readers.read_member(&entry).and_then(|bytes| {
                    let extension = Path::new(&entry.name)
                        .extension()
                        .and_then(|extension| extension.to_str())
                        .unwrap_or_default();
                    let decoded = decode_image_bytes(&bytes, extension)
                        .map_err(|error| ArchiveError::Decode(error.to_string()))?;
                    Ok(LoadedArchiveImage {
                        index,
                        handle: decoded.handle,
                        image: decoded.image,
                        width: decoded.width,
                        height: decoded.height,
                    })
                })
            });
            let _ = tx.send(result);
        });

        rx.await.map_err(|_| ArchiveError::ReaderPoolUnavailable)?
    }
}

fn read_member<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    index: usize,
    name: &str,
    max_size: u64,
) -> Result<Vec<u8>, ArchiveError> {
    let mut file = archive.by_index(index)?;
    let size = file.size();
    if size > max_size {
        return Err(ArchiveError::EntryTooLarge {
            name: name.to_owned(),
            size,
            limit: max_size,
        });
    }

    let mut bytes = Vec::with_capacity(usize::try_from(size).unwrap_or(0));
    file.by_ref().take(max_size + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_size {
        return Err(ArchiveError::EntryTooLarge {
            name: name.to_owned(),
            size: bytes.len() as u64,
            limit: max_size,
        });
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;

    fn archive_with(members: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, contents) in members {
            writer
                .start_file(*name, SimpleFileOptions::default())
                .expect("start ZIP member");
            writer.write_all(contents).expect("write ZIP member");
        }
        writer.finish().expect("finish ZIP archive").into_inner()
    }

    #[tokio::test]
    async fn indexes_supported_nested_members_in_natural_order() {
        let bytes = archive_with(&[
            ("nested/image10.png", b"invalid"),
            ("nested/image2.png", b"invalid"),
            ("nested/image999999999999999999999.png", b"invalid"),
            ("nested/image1000000000000000000000.png", b"invalid"),
            ("nested/readme.txt", b"ignored"),
            ("nested/other/image1.jpg", b"invalid"),
            ("folder/", b""),
        ]);
        let path = std::env::temp_dir().join(format!(
            "cosmic-viewer-archive-index-{}.zip",
            std::process::id()
        ));
        std::fs::write(&path, bytes).expect("write test archive");

        let catalog = ArchiveCatalog::open(path.clone())
            .await
            .expect("index archive");
        assert_eq!(
            catalog
                .entries()
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            [
                "nested/image2.png",
                "nested/image10.png",
                "nested/image999999999999999999999.png",
                "nested/image1000000000000000000000.png",
                "nested/other/image1.jpg"
            ]
        );
        std::fs::remove_file(path).expect("remove test archive");
    }

    #[test]
    fn rejects_entries_over_the_decompressed_size_limit() {
        let bytes = archive_with(&[("large.png", b"12345")]);
        let mut archive = ZipArchive::new(Cursor::new(bytes)).expect("open test archive");
        let error = read_member(&mut archive, 0, "large.png", 4)
            .expect_err("oversized member should be rejected");
        assert!(matches!(error, ArchiveError::EntryTooLarge { .. }));
    }

    #[test]
    fn bounds_the_number_of_queued_archive_reads() {
        let in_flight = AtomicUsize::new(0);
        let first = reserve_archive_read(&in_flight, 2).expect("first slot");
        let second = reserve_archive_read(&in_flight, 2).expect("second slot");
        assert!(matches!(
            reserve_archive_read(&in_flight, 2),
            Err(ArchiveError::ReadQueueFull)
        ));
        drop(first);
        assert!(reserve_archive_read(&in_flight, 2).is_ok());
        drop(second);
    }

    #[tokio::test]
    async fn handles_empty_and_corrupt_archives_per_member() {
        let path = std::env::temp_dir().join(format!(
            "cosmic-viewer-archive-empty-{}.zip",
            std::process::id()
        ));
        std::fs::write(&path, archive_with(&[])).expect("write empty archive");
        let catalog = ArchiveCatalog::open(path.clone())
            .await
            .expect("index empty archive");
        assert!(catalog.entries().is_empty());
        std::fs::remove_file(&path).expect("remove empty archive");

        let path = std::env::temp_dir().join(format!(
            "cosmic-viewer-archive-corrupt-{}.zip",
            std::process::id()
        ));
        std::fs::write(&path, archive_with(&[("bad.png", b"not an image")]))
            .expect("write corrupt member archive");
        let catalog = ArchiveCatalog::open(path.clone())
            .await
            .expect("index archive with corrupt image");
        assert!(catalog.load_image(0).await.is_err());
        std::fs::remove_file(path).expect("remove corrupt archive");
    }

    #[tokio::test]
    async fn decodes_supported_members_in_memory() {
        let bytes = archive_with(&[(
            "folder/test.png",
            include_bytes!("../../test_images/test_format.png"),
        )]);
        let path = std::env::temp_dir().join(format!(
            "cosmic-viewer-archive-image-{}.zip",
            std::process::id()
        ));
        std::fs::write(&path, bytes).expect("write image archive");

        let catalog = ArchiveCatalog::open(path.clone())
            .await
            .expect("index image archive");
        let loaded = catalog.load_image(0).await.expect("decode member");
        assert!(loaded.width > 0);
        assert!(loaded.height > 0);
        std::fs::remove_file(path).expect("remove image archive");
    }

    #[tokio::test]
    async fn rejects_invalid_zip_files() {
        let path = std::env::temp_dir().join(format!(
            "cosmic-viewer-archive-invalid-{}.zip",
            std::process::id()
        ));
        std::fs::write(&path, b"not a zip file").expect("write invalid archive");
        assert!(ArchiveCatalog::open(path.clone()).await.is_err());
        std::fs::remove_file(path).expect("remove invalid archive");
    }
}
