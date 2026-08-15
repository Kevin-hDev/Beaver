use super::super::super::{OllamaFsError, OllamaFsErrorKind};
use super::handles;
use std::mem::offset_of;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_NO_MORE_FILES, HANDLE};
use windows_sys::Win32::Storage::FileSystem::{
    FileIdBothDirectoryInfo, FileIdBothDirectoryRestartInfo, GetFileInformationByHandleEx,
    FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_ID_BOTH_DIR_INFO,
};

const ENUM_BUFFER_BYTES: usize = 64 * 1024;
const MAX_DELETE_ENTRIES: usize = 8 * 1024;
const MAX_NAME_UNITS: usize = 16 * 1024;

pub(super) fn remove_contents(
    parent: HANDLE,
    volume: HANDLE,
    volume_serial: u32,
    depth: usize,
    removed_entries: &mut usize,
) -> Result<(), OllamaFsError> {
    if depth > super::MAX_DELETE_DEPTH {
        return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
    }
    let mut buffer = vec![0_u8; ENUM_BUFFER_BYTES];
    let mut restart = true;
    loop {
        let class = if restart {
            FileIdBothDirectoryRestartInfo
        } else {
            FileIdBothDirectoryInfo
        };
        let ok = unsafe {
            GetFileInformationByHandleEx(
                parent,
                class,
                buffer.as_mut_ptr().cast(),
                buffer.len() as u32,
            )
        };
        if ok == 0 {
            return match unsafe { GetLastError() } {
                ERROR_NO_MORE_FILES => Ok(()),
                code => Err(super::super::win_error(code)),
            };
        }
        restart = false;
        let mut offset = 0usize;
        loop {
            let (entry, next) = parse_entry(&buffer, offset)?;
            if !is_dot(&entry.name) {
                *removed_entries = removed_entries.saturating_add(1);
                if *removed_entries > MAX_DELETE_ENTRIES {
                    return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
                }
                remove_child(&entry, volume, volume_serial, depth, removed_entries)?;
            }
            if next == 0 {
                break;
            }
            offset = offset
                .checked_add(next)
                .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
            if offset >= buffer.len() {
                return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
            }
        }
    }
}

struct DirectoryEntry {
    file_id: i64,
    attributes: u32,
    directory: bool,
    name: Vec<u16>,
}

fn is_dot(name: &[u16]) -> bool {
    name == [b'.' as u16] || name == [b'.' as u16, b'.' as u16]
}

fn parse_entry(buffer: &[u8], offset: usize) -> Result<(DirectoryEntry, usize), OllamaFsError> {
    let header = offset_of!(FILE_ID_BOTH_DIR_INFO, FileName);
    let next: u32 = read_field(
        buffer,
        offset,
        offset_of!(FILE_ID_BOTH_DIR_INFO, NextEntryOffset),
    )?;
    let attributes: u32 = read_field(
        buffer,
        offset,
        offset_of!(FILE_ID_BOTH_DIR_INFO, FileAttributes),
    )?;
    let name_length: u32 = read_field(
        buffer,
        offset,
        offset_of!(FILE_ID_BOTH_DIR_INFO, FileNameLength),
    )?;
    let file_id: i64 = read_field(buffer, offset, offset_of!(FILE_ID_BOTH_DIR_INFO, FileId))?;
    let record_end = if next == 0 {
        buffer.len()
    } else {
        if next as usize <= header {
            return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
        }
        offset
            .checked_add(next as usize)
            .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?
    };
    if record_end > buffer.len() || name_length % 2 != 0 {
        return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
    }
    let name_start = offset
        .checked_add(header)
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    let name_end = name_start
        .checked_add(name_length as usize)
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    if name_length as usize / 2 > MAX_NAME_UNITS || name_end > record_end {
        return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
    }
    let name = unsafe {
        std::slice::from_raw_parts(
            buffer.as_ptr().add(name_start).cast::<u16>(),
            name_length as usize / 2,
        )
    };
    Ok((
        DirectoryEntry {
            file_id,
            attributes,
            directory: attributes & FILE_ATTRIBUTE_DIRECTORY != 0,
            name: name.to_vec(),
        },
        next as usize,
    ))
}

fn read_field<T: Copy>(buffer: &[u8], offset: usize, field: usize) -> Result<T, OllamaFsError> {
    let start = offset
        .checked_add(field)
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    let end = start
        .checked_add(std::mem::size_of::<T>())
        .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::InvalidInput))?;
    if end > buffer.len() {
        return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
    }
    Ok(unsafe { std::ptr::read_unaligned(buffer.as_ptr().add(start).cast::<T>()) })
}

fn remove_child(
    entry: &DirectoryEntry,
    volume: HANDLE,
    volume_serial: u32,
    depth: usize,
    removed_entries: &mut usize,
) -> Result<(), OllamaFsError> {
    if entry.attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
    }
    let handle = handles::open_child(volume, entry.file_id, entry.directory)?;
    let info = handles::file_info(handle.raw())?;
    if info.dwVolumeSerialNumber != volume_serial
        || handles::file_id(&info) != entry.file_id as u64
        || info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || (info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0) != entry.directory
    {
        return Err(OllamaFsError::new(OllamaFsErrorKind::InvalidInput));
    }
    if entry.directory {
        remove_contents(
            handle.raw(),
            volume,
            volume_serial,
            depth.saturating_add(1),
            removed_entries,
        )?;
    }
    handles::mark_deleted(handle.raw())
}
