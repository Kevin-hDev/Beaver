import gzip
import os
import stat
import tarfile
from pathlib import Path, PurePosixPath

from searxng_safety import COPY_CHUNK_BYTES, PreparationError, fail, is_link, regular_info, safe_directory, validate_archive_path

MAX_ARCHIVE_ENTRIES = 4096
MAX_MEMBER_BYTES = 64 * 1024 * 1024
MAX_ARCHIVE_TOTAL_BYTES = 150 * 1024 * 1024
BLOCK_BYTES = 512
MAX_DECOMPRESSED_BYTES = MAX_ARCHIVE_TOTAL_BYTES + MAX_ARCHIVE_ENTRIES * (2 * BLOCK_BYTES) + 2 * BLOCK_BYTES
IGNORED_NONPORTABLE_LINKS = {
    "source/utils/templates/etc/apache2": "httpd",
}
IGNORED_NONPORTABLE_FILES = frozenset({
    "source/utils/templates/etc/httpd/sites-available/searxng.conf:socket",
    "source/utils/templates/etc/nginx/default.apps-available/searxng.conf:socket",
    "source/utils/templates/etc/uwsgi/apps-archlinux/searxng.ini:socket",
    "source/utils/templates/etc/uwsgi/apps-available/searxng.ini:socket",
})


def is_metadata(path: PurePosixPath) -> bool:
    return any(part.startswith("._") or part in {".DS_Store", ".AppleDouble", "__MACOSX", ".py"} for part in path.parts)


def _ignore_exact_nonportable_member(member: tarfile.TarInfo, seen: set[str]) -> bool:
    expected_link = IGNORED_NONPORTABLE_LINKS.get(member.name)
    ignored_file = member.name in IGNORED_NONPORTABLE_FILES
    if expected_link is None and not ignored_file:
        return False
    key = member.name.casefold()
    if key in seen:
        fail()
    if ignored_file:
        if not member.isfile() or member.size < 0 or member.size > MAX_MEMBER_BYTES:
            fail()
    elif not member.issym() or member.linkname != expected_link or member.size != 0:
        fail()
    seen.add(key)
    return True


class BoundedReader:
    def __init__(self, source):
        self.source, self.total = source, 0

    def read(self, length: int) -> bytes:
        if not isinstance(length, int) or length < 0 or length > COPY_CHUNK_BYTES:
            fail()
        chunk = self.source.read(length)
        self.total += len(chunk)
        if self.total > MAX_DECOMPRESSED_BYTES:
            fail()
        return chunk


def _read_exact(stream, length: int) -> bytes:
    pieces, remaining = [], length
    while remaining:
        chunk = stream.read(min(COPY_CHUNK_BYTES, remaining))
        if not chunk:
            fail()
        pieces.append(chunk)
        remaining -= len(chunk)
    return b"".join(pieces)


def _tar_size(field: bytes) -> int:
    if field[0] & 0x80:
        value = int.from_bytes(bytes([field[0] & 0x7F]) + field[1:], "big")
    else:
        value_bytes = field.rstrip(b"\0 ")
        if value_bytes and any(byte < ord("0") or byte > ord("7") for byte in value_bytes):
            fail()
        value = int(value_bytes or b"0", 8)
    if value < 0:
        fail()
    return value


def _skip(stream, length: int) -> None:
    remaining = length
    while remaining:
        chunk = stream.read(min(COPY_CHUNK_BYTES, remaining))
        if not chunk:
            fail()
        remaining -= len(chunk)


def preflight_tar(archive: Path) -> None:
    try:
        regular_info(archive, MAX_ARCHIVE_TOTAL_BYTES)
        entries = total = 0
        with gzip.open(archive, "rb") as compressed:
            source = BoundedReader(compressed)
            while True:
                header = _read_exact(source, BLOCK_BYTES)
                if header == b"\0" * BLOCK_BYTES:
                    if _read_exact(source, BLOCK_BYTES) != b"\0" * BLOCK_BYTES:
                        fail()
                    while chunk := source.read(COPY_CHUNK_BYTES):
                        if any(chunk):
                            fail()
                    return
                entries += 1
                size = _tar_size(header[124:136])
                if entries > MAX_ARCHIVE_ENTRIES or size > MAX_MEMBER_BYTES:
                    fail()
                total += size
                if total > MAX_ARCHIVE_TOTAL_BYTES:
                    fail()
                _skip(source, size + (-size % BLOCK_BYTES))
    except PreparationError:
        raise
    except (OSError, EOFError):
        fail()


def _safe_child(directory: Path, parts: tuple[str, ...]) -> Path:
    current = directory
    for part in parts:
        current = current / part
        try:
            current.mkdir()
        except FileExistsError:
            pass
        except OSError:
            fail()
        info = current.lstat()
        canonical = current.resolve(strict=True)
        if (
            not stat.S_ISDIR(info.st_mode)
            or is_link(current)
            or os.path.normcase(str(current)) != os.path.normcase(str(canonical))
        ):
            fail()
    return current


def copy_member(source: tarfile.TarFile, member: tarfile.TarInfo, destination: Path, relative: PurePosixPath) -> None:
    try:
        if member.isdir():
            _safe_child(destination, relative.parts)
            return
        target = _safe_child(destination, relative.parts[:-1]) / relative.name
        if target.exists() or target.is_symlink():
            fail()
        content = source.extractfile(member)
        if content is None:
            fail()
        remaining = member.size
        with content, target.open("xb") as output:
            while remaining:
                chunk = content.read(min(COPY_CHUNK_BYTES, remaining))
                if not chunk:
                    fail()
                output.write(chunk)
                remaining -= len(chunk)
            if content.read(1):
                fail()
    except PreparationError:
        raise
    except Exception:
        fail()


def safe_extract(archive: Path, destination: Path) -> None:
    try:
        preflight_tar(archive)
        destination, seen, extracted = safe_directory(destination), set(), 0
        with tarfile.open(archive, "r:gz") as source:
            for member in source:
                path = PurePosixPath(member.name)
                if _ignore_exact_nonportable_member(member, seen):
                    continue
                relative = validate_archive_path(member.name, seen)
                if is_metadata(path):
                    continue
                if not (member.isdir() or member.isfile()) or member.size < 0 or member.size > MAX_MEMBER_BYTES:
                    fail()
                extracted += 1
                copy_member(source, member, destination, relative)
        if not extracted:
            fail()
    except PreparationError:
        raise
    except Exception:
        fail()
