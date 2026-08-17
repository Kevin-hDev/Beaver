#![cfg(test)]

use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) enum TarMember<'a> {
    Regular(&'a str, &'a [u8], u32),
    Raw(&'a str, tar::EntryType, &'a [u8], u32, Option<&'a str>),
}

impl<'a> TarMember<'a> {
    pub(crate) fn regular(name: &'a str, bytes: &'a [u8], mode: u32) -> Self {
        Self::Regular(name, bytes, mode)
    }

    pub(crate) fn raw(
        name: &'a str,
        entry_type: tar::EntryType,
        bytes: &'a [u8],
        mode: u32,
        link: Option<&'a str>,
    ) -> Self {
        Self::Raw(name, entry_type, bytes, mode, link)
    }
}

pub(crate) fn write_gzip_tar(path: &Path, members: &[TarMember<'_>]) {
    let file = std::fs::File::create(path).unwrap();
    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    for member in members {
        match member {
            TarMember::Regular(name, bytes, mode) => {
                let mut header = tar::Header::new_gnu();
                header.set_size(bytes.len() as u64);
                header.set_mode(*mode);
                header.set_entry_type(tar::EntryType::Regular);
                builder.append_data(&mut header, *name, *bytes).unwrap();
            }
            TarMember::Raw(name, entry_type, bytes, mode, link) => {
                let header = raw_header(name, *entry_type, bytes.len() as u64, *mode, *link);
                builder.append(&header, *bytes).unwrap();
            }
        }
    }
    let encoder = builder.into_inner().unwrap();
    encoder.finish().unwrap();
}

pub(crate) fn raw_header(
    name: &str,
    entry_type: tar::EntryType,
    size: u64,
    mode: u32,
    link: Option<&str>,
) -> tar::Header {
    let mut header = tar::Header::new_gnu();
    header.set_size(size);
    header.set_mode(mode);
    header.set_entry_type(entry_type);
    let name_bytes = name.as_bytes();
    header.as_gnu_mut().unwrap().name[..name_bytes.len()].copy_from_slice(name_bytes);
    if let Some(link) = link {
        let link_bytes = link.as_bytes();
        header.as_gnu_mut().unwrap().linkname[..link_bytes.len()].copy_from_slice(link_bytes);
    }
    header.set_cksum();
    header
}

pub(crate) enum ZipMember<'a> {
    Regular(&'a str, &'a [u8]),
}

impl<'a> ZipMember<'a> {
    pub(crate) fn regular(name: &'a str, bytes: &'a [u8]) -> Self {
        Self::Regular(name, bytes)
    }
}

pub(crate) fn write_zip(path: &Path, members: &[ZipMember<'_>]) {
    let file = std::fs::File::create(path).unwrap();
    let mut archive = zip::ZipWriter::new(file);
    for member in members {
        let ZipMember::Regular(name, bytes) = member;
        let options = zip::write::SimpleFileOptions::default().unix_permissions(0o755);
        archive.start_file(name, options).unwrap();
        archive.write_all(bytes).unwrap();
    }
    archive.finish().unwrap();
}

pub(crate) fn write_duplicate_zip(path: &Path, name: &str, first: &[u8], second: &[u8]) {
    let mut bytes = Vec::new();
    let mut central = Vec::new();
    append_stored_zip_entry(&mut bytes, &mut central, name, first, 0, 20);
    append_stored_zip_entry(&mut bytes, &mut central, name, second, 0, 20);
    finish_zip(path, bytes, central, 2);
}

pub(crate) fn write_symlink_zip(path: &Path, name: &str, target: &[u8]) {
    let mut bytes = Vec::new();
    let mut central = Vec::new();
    append_stored_zip_entry(
        &mut bytes,
        &mut central,
        name,
        target,
        0o120777 << 16,
        (3 << 8) | 20,
    );
    finish_zip(path, bytes, central, 1);
}

fn finish_zip(path: &Path, mut bytes: Vec<u8>, central: Vec<u8>, entries: u16) {
    let central_offset = bytes.len() as u32;
    bytes.extend_from_slice(&central);
    let central_size = central.len() as u32;
    bytes.extend_from_slice(&0x06054b50_u32.to_le_bytes());
    bytes.extend_from_slice(&[0; 4]);
    bytes.extend_from_slice(&entries.to_le_bytes());
    bytes.extend_from_slice(&entries.to_le_bytes());
    bytes.extend_from_slice(&central_size.to_le_bytes());
    bytes.extend_from_slice(&central_offset.to_le_bytes());
    bytes.extend_from_slice(&[0; 2]);
    std::fs::write(path, bytes).unwrap();
}

fn append_stored_zip_entry(
    local: &mut Vec<u8>,
    central: &mut Vec<u8>,
    name: &str,
    data: &[u8],
    external_attributes: u32,
    version_made_by: u16,
) {
    let name = name.as_bytes();
    let crc = crc32(data);
    let offset = local.len() as u32;
    local.extend_from_slice(&0x04034b50_u32.to_le_bytes());
    local.extend_from_slice(&20_u16.to_le_bytes());
    local.extend_from_slice(&0_u16.to_le_bytes());
    local.extend_from_slice(&0_u16.to_le_bytes());
    local.extend_from_slice(&[0; 4]);
    local.extend_from_slice(&crc.to_le_bytes());
    local.extend_from_slice(&(data.len() as u32).to_le_bytes());
    local.extend_from_slice(&(data.len() as u32).to_le_bytes());
    local.extend_from_slice(&(name.len() as u16).to_le_bytes());
    local.extend_from_slice(&0_u16.to_le_bytes());
    local.extend_from_slice(name);
    local.extend_from_slice(data);

    central.extend_from_slice(&0x02014b50_u32.to_le_bytes());
    central.extend_from_slice(&version_made_by.to_le_bytes());
    central.extend_from_slice(&20_u16.to_le_bytes());
    central.extend_from_slice(&0_u16.to_le_bytes());
    central.extend_from_slice(&0_u16.to_le_bytes());
    central.extend_from_slice(&[0; 4]);
    central.extend_from_slice(&crc.to_le_bytes());
    central.extend_from_slice(&(data.len() as u32).to_le_bytes());
    central.extend_from_slice(&(data.len() as u32).to_le_bytes());
    central.extend_from_slice(&(name.len() as u16).to_le_bytes());
    central.extend_from_slice(&[0; 6]);
    central.extend_from_slice(&0_u16.to_le_bytes());
    central.extend_from_slice(&external_attributes.to_le_bytes());
    central.extend_from_slice(&offset.to_le_bytes());
    central.extend_from_slice(name);
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & (0_u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}

pub(crate) fn empty_staging(root: &Path) -> PathBuf {
    let staging = root.join("staging");
    std::fs::create_dir(&staging).unwrap();
    staging
}

#[cfg(unix)]
pub(crate) fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).unwrap().permissions().mode() & 0o111 != 0
}
