#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct FileIdentity(u128);

#[cfg(unix)]
pub(super) fn from_metadata(metadata: &std::fs::Metadata) -> Option<FileIdentity> {
    use std::os::unix::fs::MetadataExt;

    Some(FileIdentity(
        (u128::from(metadata.dev()) << 64) | u128::from(metadata.ino()),
    ))
}

#[cfg(windows)]
pub(super) fn from_metadata(metadata: &std::fs::Metadata) -> Option<FileIdentity> {
    use std::os::windows::fs::MetadataExt;

    let volume = metadata.volume_serial_number()?;
    let file = metadata.file_index()?;
    (volume != 0 && file != 0)
        .then_some(FileIdentity((u128::from(volume) << 64) | u128::from(file)))
}

#[cfg(not(any(unix, windows)))]
pub(super) fn from_metadata(_: &std::fs::Metadata) -> Option<FileIdentity> {
    None
}
