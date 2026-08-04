import hashlib
import os
import re
import secrets
import shutil
import stat
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path, PurePosixPath

MAX_ARCHIVE_ENTRIES = 4096
MAX_MEMBER_BYTES = 64 * 1024 * 1024
MAX_WHEEL_BYTES = 150 * 1024 * 1024
MAX_WHEEL_FILES = 512
COPY_CHUNK_BYTES = 1024 * 1024
STAMP_NAME = ".requirements.sha256"
ERROR_MESSAGE = "SearXNG preparation failed"

class PreparationError(Exception):
    pass

def fail() -> None:
    raise PreparationError(ERROR_MESSAGE)

def is_metadata(path: PurePosixPath) -> bool:
    return any(part.startswith("._") or part in {".DS_Store", ".AppleDouble", "__MACOSX", ".py"} for part in path.parts)

def regular_file(path: Path):
    try:
        if not path.is_absolute() or ".." in path.parts:
            fail()
        info = path.lstat()
    except OSError:
        fail()
    if not stat.S_ISREG(info.st_mode) or info.st_nlink > 1:
        fail()
    return info

def safe_directory(path: Path) -> Path:
    try:
        if not path.is_absolute() or ".." in path.parts:
            fail()
        info, canonical = path.lstat(), path.resolve(strict=True)
    except OSError:
        fail()
    if not stat.S_ISDIR(info.st_mode) or path.is_symlink() or os.path.normcase(str(path)) != os.path.normcase(str(canonical)):
        fail()
    return canonical

def safe_child(directory: Path, parts: tuple[str, ...]) -> Path:
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
        if not stat.S_ISDIR(info.st_mode) or current.is_symlink():
            fail()
    return current

def copy_member(source: tarfile.TarFile, member: tarfile.TarInfo, destination: Path, relative: PurePosixPath) -> None:
    try:
        if member.isdir():
            safe_child(destination, relative.parts)
            return
        target = safe_child(destination, relative.parts[:-1]) / relative.name
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
    except Exception:
        fail()

def safe_extract(archive: Path, destination: Path) -> None:
    try:
        regular_file(archive)
        names, count = set(), 0
        with tarfile.open(archive, "r:gz") as source:
            for member in source:
                count += 1
                relative = PurePosixPath(member.name)
                if count > MAX_ARCHIVE_ENTRIES or not member.name or "\\" in member.name or relative.is_absolute() or ".." in relative.parts:
                    fail()
                if is_metadata(relative):
                    continue
                if relative in names or not (member.isdir() or member.isfile()) or not 0 <= member.size <= MAX_MEMBER_BYTES:
                    fail()
                names.add(relative)
                copy_member(source, member, safe_directory(destination), relative)
        if not names:
            fail()
    except Exception:
        fail()

def validate_source(source: Path) -> tuple[Path, Path]:
    source, pending, count = safe_directory(source), [safe_directory(source)], 0
    while pending:
        current = pending.pop()
        try:
            with os.scandir(current) as entries:
                for entry in entries:
                    count += 1
                    info = entry.stat(follow_symlinks=False)
                    if count > MAX_ARCHIVE_ENTRIES or is_metadata(PurePosixPath(entry.name)) or entry.is_symlink():
                        fail()
                    if stat.S_ISDIR(info.st_mode):
                        pending.append(Path(entry.path))
                    elif not stat.S_ISREG(info.st_mode) or info.st_nlink > 1:
                        fail()
        except OSError:
            fail()
    requirements, setup = source / "requirements.txt", source / "setup.py"
    for path in requirements, setup:
        if regular_file(path).st_size > MAX_MEMBER_BYTES:
            fail()
    return requirements, setup

def requirements_hash(requirements: Path, setup: Path) -> str:
    outer = hashlib.sha256()
    for path in requirements, setup:
        digest = hashlib.sha256()
        with path.open("rb") as content:
            while chunk := content.read(COPY_CHUNK_BYTES):
                digest.update(chunk)
        outer.update(digest.digest())
    return outer.hexdigest()

def bundle_valid(directory: Path, expected_stamp: str, needs_stamp: bool) -> bool:
    try:
        wheels, total, stamp = 0, 0, None
        with os.scandir(safe_directory(directory)) as entries:
            for entry in entries:
                if entry.name == STAMP_NAME:
                    info = regular_file(Path(entry.path))
                    if info.st_size != 64:
                        return False
                    stamp = Path(entry.path).read_text(encoding="ascii")
                    continue
                info = entry.stat(follow_symlinks=False)
                if not entry.name.endswith(".whl") or entry.is_symlink() or not stat.S_ISREG(info.st_mode) or info.st_nlink > 1:
                    return False
                wheels, total = wheels + 1, total + info.st_size
                if wheels > MAX_WHEEL_FILES or total > MAX_WHEEL_BYTES:
                    return False
        return wheels > 0 and (not needs_stamp or stamp is not None and re.fullmatch(r"[0-9a-f]{64}", stamp) is not None and secrets.compare_digest(stamp, expected_stamp))
    except (OSError, UnicodeError, PreparationError):
        return False

def replace_bundle(parent: Path, temporary: Path) -> None:
    destination = parent / "wheels"
    backup = parent / f"wheels-backup-{secrets.token_hex(16)}"
    moved = completed = False
    try:
        if backup.exists() or backup.is_symlink():
            fail()
        if destination.exists() or destination.is_symlink():
            safe_directory(destination)
            os.replace(destination, backup)
            moved = True
        os.replace(temporary, destination)
        completed = True
    except OSError:
        if moved:
            try:
                if not destination.exists():
                    os.replace(backup, destination)
            except OSError:
                pass
        fail()
    finally:
        if completed and backup.exists():
            shutil.rmtree(backup, ignore_errors=True)

def prepare(root: Path, run_process=subprocess.run) -> None:
    temporary_source = temporary_wheels = None
    try:
        sidecar = safe_directory(safe_directory(root) / "resources" / "searxng-sidecar")
        source = sidecar / "source"
        if not source.exists():
            temporary_source = safe_directory(Path(tempfile.mkdtemp(prefix="source-", dir=sidecar)))
            safe_extract(sidecar / "source.tar.gz", temporary_source)
            source = temporary_source / "source"
        requirements, setup = validate_source(source)
        stamp = requirements_hash(requirements, setup)
        if bundle_valid(sidecar / "wheels", stamp, True):
            return
        temporary_wheels = safe_directory(Path(tempfile.mkdtemp(prefix="wheels-new-", dir=sidecar)))
        for arguments in (["-r", str(requirements)], ["setuptools", "wheel"]):
            run_process([sys.executable, "-m", "pip", "download", "--only-binary=:all:", "--dest", str(temporary_wheels), *arguments], check=True, shell=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        if not bundle_valid(temporary_wheels, stamp, False):
            fail()
        (temporary_wheels / STAMP_NAME).write_text(stamp, encoding="ascii")
        if not bundle_valid(temporary_wheels, stamp, True):
            fail()
        replace_bundle(sidecar, temporary_wheels)
        temporary_wheels = None
    except Exception:
        fail()
    finally:
        for directory in temporary_source, temporary_wheels:
            if directory is not None:
                shutil.rmtree(directory, ignore_errors=True)

if __name__ == "__main__":
    try:
        if len(sys.argv) != 3 or sys.argv[1] != "--root":
            fail()
        prepare(Path(sys.argv[2]))
    except Exception:
        print(ERROR_MESSAGE, file=sys.stderr)
        raise SystemExit(1)
