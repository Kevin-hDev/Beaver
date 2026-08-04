import hashlib
import os
import stat
import unicodedata
from pathlib import Path, PurePosixPath

ERROR_MESSAGE = "SearXNG preparation failed"
MAX_PATH_LENGTH = 1024
MAX_SEGMENT_LENGTH = 120
COPY_CHUNK_BYTES = 1024 * 1024
WINDOWS_FORBIDDEN = set('<>:"\\|?*')
WINDOWS_RESERVED = {"CON", "PRN", "AUX", "NUL", *(f"COM{index}" for index in range(1, 10)), *(f"LPT{index}" for index in range(1, 10))}


class PreparationError(Exception):
    def __init__(self):
        super().__init__(ERROR_MESSAGE)


class PathValidationError(PreparationError):
    pass


def fail() -> None:
    raise PreparationError()


def is_link(path: Path) -> bool:
    return path.is_symlink() or bool(getattr(path, "is_junction", lambda: False)())


def has_hardlink(path: Path, info) -> bool:
    if info.st_nlink:
        return info.st_nlink > 1
    if os.name != "nt":
        return False
    try:
        import ctypes
        import msvcrt
        class StandardInfo(ctypes.Structure):
            _fields_ = [("allocation", ctypes.c_longlong), ("size", ctypes.c_longlong), ("links", ctypes.c_uint32), ("deleted", ctypes.c_ubyte), ("directory", ctypes.c_ubyte)]
        descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
        try:
            result = StandardInfo()
            if not ctypes.windll.kernel32.GetFileInformationByHandleEx(msvcrt.get_osfhandle(descriptor), 1, ctypes.byref(result), ctypes.sizeof(result)):
                fail()
            return result.links > 1
        finally:
            os.close(descriptor)
    except (OSError, AttributeError):
        fail()


def absolute_path(value) -> Path:
    if not isinstance(value, Path) or not value.is_absolute() or ".." in value.parts:
        fail()
    return value


def safe_directory(path: Path) -> Path:
    try:
        path = absolute_path(path)
        info, canonical = path.lstat(), path.resolve(strict=True)
    except (OSError, PreparationError):
        fail()
    if not stat.S_ISDIR(info.st_mode) or is_link(path) or os.path.normcase(str(path)) != os.path.normcase(str(canonical)):
        fail()
    return canonical


def regular_info(path: Path, maximum: int):
    try:
        path = absolute_path(path)
        info = path.lstat()
    except (OSError, PreparationError):
        fail()
    if not stat.S_ISREG(info.st_mode) or has_hardlink(path, info) or info.st_size < 0 or info.st_size > maximum:
        fail()
    return info


def _same_identity(before, after) -> bool:
    if before.st_dev and before.st_ino and after.st_dev and after.st_ino:
        return before.st_dev == after.st_dev and before.st_ino == after.st_ino
    return before.st_size == after.st_size and stat.S_IFMT(before.st_mode) == stat.S_IFMT(after.st_mode)


def hash_regular_file(path: Path, maximum: int, after_open=None) -> bytes:
    before = regular_info(path, maximum)
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_BINARY", 0)
    try:
        descriptor = os.open(path, flags)
    except OSError:
        fail()
    try:
        opened = os.fstat(descriptor)
        if not stat.S_ISREG(opened.st_mode) or has_hardlink(path, opened) or opened.st_size != before.st_size or not _same_identity(before, opened):
            fail()
        if after_open is not None:
            after_open(descriptor)
        digest, total = hashlib.sha256(), 0
        while True:
            chunk = os.read(descriptor, min(COPY_CHUNK_BYTES, maximum + 1 - total))
            if not chunk:
                break
            total += len(chunk)
            if total > maximum:
                fail()
            digest.update(chunk)
        closed = os.fstat(descriptor)
        if total != opened.st_size or not _same_identity(opened, closed) or closed.st_size != opened.st_size:
            fail()
        return digest.digest()
    except PreparationError:
        raise
    except OSError:
        fail()
    finally:
        os.close(descriptor)


def read_regular_file(path: Path, maximum: int) -> bytes:
    before = regular_info(path, maximum)
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_BINARY", 0)
    try:
        descriptor = os.open(path, flags)
    except OSError:
        fail()
    try:
        opened, pieces, total = os.fstat(descriptor), [], 0
        if has_hardlink(path, opened) or not _same_identity(before, opened) or opened.st_size != before.st_size:
            fail()
        while True:
            chunk = os.read(descriptor, min(COPY_CHUNK_BYTES, maximum + 1 - total))
            if not chunk:
                break
            total += len(chunk)
            if total > maximum:
                fail()
            pieces.append(chunk)
        if total != opened.st_size or not _same_identity(opened, os.fstat(descriptor)):
            fail()
        return b"".join(pieces)
    except PreparationError:
        raise
    except OSError:
        fail()
    finally:
        os.close(descriptor)


def _valid_segment(segment: str) -> bool:
    normalized = unicodedata.normalize("NFC", segment)
    base = normalized.split(".", 1)[0].upper()
    return (
        normalized == segment
        and 1 <= len(segment) <= MAX_SEGMENT_LENGTH
        and segment not in {".", ".."}
        and not segment.endswith((".", " "))
        and base not in WINDOWS_RESERVED
        and segment.isascii()
        and not any(char in WINDOWS_FORBIDDEN or ord(char) < 32 for char in segment)
    )


def validate_archive_path(name: str, seen: set[str]) -> PurePosixPath:
    if not isinstance(name, str) or not name or len(name) > MAX_PATH_LENGTH or "\\" in name:
        raise PathValidationError()
    parts = name.split("/")
    if any(not _valid_segment(part) for part in parts):
        raise PathValidationError()
    key = "/".join(unicodedata.normalize("NFC", part).casefold() for part in parts)
    if key in seen:
        raise PathValidationError()
    seen.add(key)
    return PurePosixPath(*parts)
