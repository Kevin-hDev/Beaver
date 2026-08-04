import hashlib
import os
import re
import secrets
import shutil
import stat
from pathlib import Path, PurePosixPath

from searxng_archive import MAX_ARCHIVE_ENTRIES, MAX_MEMBER_BYTES, is_metadata
from searxng_safety import PreparationError, fail, has_hardlink, hash_regular_file, is_link, read_regular_file, regular_info, safe_directory

MAX_WHEEL_BYTES = 150 * 1024 * 1024
MAX_WHEEL_FILES = 512
STAMP_NAME = ".requirements.sha256"
JOURNAL_NAME = ".prepare.transaction"
TEMP_ATTEMPTS = 8


def temporary_directory(parent: Path, prefix: str, token=secrets.token_hex) -> Path:
    parent = safe_directory(parent)
    for _ in range(TEMP_ATTEMPTS):
        candidate = parent / f"{prefix}{token(16)}"
        try:
            candidate.mkdir(mode=0o700)
            return safe_directory(candidate)
        except FileExistsError:
            continue
        except OSError:
            fail()
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
                    if count > MAX_ARCHIVE_ENTRIES or is_metadata(PurePosixPath(entry.name)) or is_link(Path(entry.path)):
                        fail()
                    if stat.S_ISDIR(info.st_mode):
                        pending.append(Path(entry.path))
                    elif not stat.S_ISREG(info.st_mode) or has_hardlink(Path(entry.path), info):
                        fail()
        except OSError:
            fail()
    requirements, setup = source / "requirements.txt", source / "setup.py"
    regular_info(requirements, MAX_MEMBER_BYTES)
    regular_info(setup, MAX_MEMBER_BYTES)
    return requirements, setup


def requirements_hash(requirements: Path, setup: Path) -> str:
    outer = hashlib.sha256()
    outer.update(hash_regular_file(requirements, MAX_MEMBER_BYTES))
    outer.update(hash_regular_file(setup, MAX_MEMBER_BYTES))
    return outer.hexdigest()


def bundle_valid(directory: Path, expected_stamp: str, needs_stamp=True) -> bool:
    try:
        wheels, total, stamp = 0, 0, None
        with os.scandir(safe_directory(directory)) as entries:
            for entry in entries:
                path = Path(entry.path)
                if entry.name == STAMP_NAME:
                    stamp = read_regular_file(path, 64).decode("ascii")
                    if len(stamp) != 64:
                        return False
                    continue
                info = entry.stat(follow_symlinks=False)
                if not entry.name.endswith(".whl") or is_link(path) or not stat.S_ISREG(info.st_mode) or has_hardlink(path, info):
                    return False
                wheels, total = wheels + 1, total + info.st_size
                if wheels > MAX_WHEEL_FILES or total > MAX_WHEEL_BYTES:
                    return False
        return wheels > 0 and (not needs_stamp or stamp is not None and re.fullmatch(r"[0-9a-f]{64}", stamp) is not None and secrets.compare_digest(stamp, expected_stamp))
    except (OSError, UnicodeError, PreparationError):
        return False


class BundleLock:
    def __init__(self, parent: Path):
        self.path = safe_directory(parent) / ".prepare.lock"
        self.descriptor = None

    def __enter__(self):
        flags = os.O_RDWR | os.O_CREAT | getattr(os, "O_NOFOLLOW", 0)
        try:
            self.descriptor = os.open(self.path, flags, 0o600)
            if not stat.S_ISREG(os.fstat(self.descriptor).st_mode):
                fail()
            if os.name == "nt":
                import msvcrt
                os.lseek(self.descriptor, 0, os.SEEK_SET)
                msvcrt.locking(self.descriptor, msvcrt.LK_NBLCK, 1)
            else:
                import fcntl
                fcntl.flock(self.descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            return self
        except OSError:
            self.__exit__(None, None, None)
            fail()

    def __exit__(self, *_):
        if self.descriptor is None:
            return
        try:
            if os.name == "nt":
                import msvcrt
                os.lseek(self.descriptor, 0, os.SEEK_SET)
                msvcrt.locking(self.descriptor, msvcrt.LK_UNLCK, 1)
            else:
                import fcntl
                fcntl.flock(self.descriptor, fcntl.LOCK_UN)
        except OSError:
            pass
        finally:
            os.close(self.descriptor)
            self.descriptor = None


def _atomic_journal(parent: Path, backup: Path, temporary: Path, stamp: str) -> None:
    content = f"{backup.name}\n{temporary.name}\n{stamp}\n".encode("ascii")
    if len(content) > 256:
        fail()
    for _ in range(TEMP_ATTEMPTS):
        temp = parent / f".transaction-{secrets.token_hex(16)}.tmp"
        try:
            descriptor = os.open(temp, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        except FileExistsError:
            continue
        except OSError:
            fail()
        try:
            with os.fdopen(descriptor, "wb") as output:
                output.write(content)
                output.flush()
                os.fsync(output.fileno())
            os.replace(temp, parent / JOURNAL_NAME)
            return
        except OSError:
            fail()
        finally:
            if temp.exists():
                temp.unlink(missing_ok=True)
    fail()


def _read_journal(parent: Path):
    journal = parent / JOURNAL_NAME
    if not journal.exists():
        return None
    try:
        backup, temporary, stamp, empty = read_regular_file(journal, 256).decode("ascii").split("\n")
    except (UnicodeError, ValueError, PreparationError):
        fail()
    if empty or not re.fullmatch(r"wheels-backup-[0-9a-f]{32}", backup) or not re.fullmatch(r"wheels-new-[0-9a-f]{32}", temporary) or not re.fullmatch(r"[0-9a-f]{64}", stamp):
        fail()
    return parent / backup, parent / temporary, stamp


def _remove_directory(path: Path) -> None:
    if path.exists():
        safe_directory(path)
        shutil.rmtree(path)


def recover_bundle(parent: Path, expected_stamp: str) -> None:
    parent = safe_directory(parent)
    record = _read_journal(parent)
    if record is None:
        return
    backup, temporary, stamp = record
    destination = parent / "wheels"
    if not destination.exists() and backup.exists():
        os.replace(backup, destination)
    elif destination.exists() and backup.exists():
        if bundle_valid(destination, expected_stamp, True):
            _remove_directory(backup)
        else:
            _remove_directory(temporary)
            os.replace(destination, temporary)
            os.replace(backup, destination)
    elif not destination.exists():
        fail()
    if not bundle_valid(destination, expected_stamp, True):
        fail()
    _remove_directory(temporary)
    (parent / JOURNAL_NAME).unlink(missing_ok=True)


def publish_bundle(parent: Path, temporary: Path, stamp: str, replace=os.replace) -> None:
    parent, temporary = safe_directory(parent), safe_directory(temporary)
    destination = parent / "wheels"
    backup = parent / f"wheels-backup-{secrets.token_hex(16)}"
    if backup.exists() or backup.is_symlink() or not bundle_valid(temporary, stamp, True):
        fail()
    _atomic_journal(parent, backup, temporary, stamp)
    try:
        if destination.exists() or destination.is_symlink():
            safe_directory(destination)
            replace(destination, backup)
        replace(temporary, destination)
    except OSError:
        fail()
    recover_bundle(parent, stamp)
