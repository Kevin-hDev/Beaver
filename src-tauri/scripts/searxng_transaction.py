import os
import re
import secrets
import shutil
import stat
from pathlib import Path

from searxng_bundle import TEMP_ATTEMPTS, bundle_valid
from searxng_safety import PreparationError, fail, has_hardlink, is_link, read_regular_file, regular_info, safe_directory

MAX_SCAN_ENTRIES = 1024
JOURNAL_NAME = ".prepare.transaction"
BACKUP = re.compile(r"wheels-backup-[0-9a-f]{32}")
TEMPORARY = re.compile(r"(?:source-|wheels-new-)[0-9a-f]{32}")
JOURNAL_TEMP = re.compile(r"\.transaction-[0-9a-f]{32}\.tmp")


class BundleLock:
    def __init__(self, parent: Path):
        self.parent = safe_directory(parent)
        self.path = self.parent / ".prepare.lock"
        self.descriptor = None
        self.locked = False

    def __enter__(self):
        try:
            if self.path.is_symlink():
                fail()
            self.descriptor = os.open(self.path, os.O_RDWR | os.O_CREAT | getattr(os, "O_NOFOLLOW", 0), 0o600)
            opened, current = os.fstat(self.descriptor), self.path.lstat()
            if not stat.S_ISREG(opened.st_mode) or is_link(self.path) or has_hardlink(self.path, current):
                fail()
            if opened.st_dev and opened.st_ino and (opened.st_dev, opened.st_ino) != (current.st_dev, current.st_ino):
                fail()
            if os.name == "nt":
                import msvcrt
                os.ftruncate(self.descriptor, 1)
                os.lseek(self.descriptor, 0, os.SEEK_SET)
                msvcrt.locking(self.descriptor, msvcrt.LK_NBLCK, 1)
            else:
                import fcntl
                fcntl.flock(self.descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            self.locked = True
            return self
        except Exception:
            self.__exit__(None, None, None)
            fail()

    def __exit__(self, *_):
        if self.descriptor is None:
            return
        try:
            if self.locked and os.name == "nt":
                import msvcrt
                os.lseek(self.descriptor, 0, os.SEEK_SET)
                msvcrt.locking(self.descriptor, msvcrt.LK_UNLCK, 1)
            elif self.locked:
                import fcntl
                fcntl.flock(self.descriptor, fcntl.LOCK_UN)
        except OSError:
            pass
        finally:
            os.close(self.descriptor)
            self.descriptor = None
            self.locked = False


def _durable_replace(source: Path, destination: Path) -> None:
    try:
        if os.name == "nt":
            import ctypes
            if not ctypes.windll.kernel32.MoveFileExW(str(source), str(destination), 0x1 | 0x8):
                fail()
        else:
            os.replace(source, destination)
            descriptor = os.open(destination.parent, os.O_RDONLY)
            try:
                os.fsync(descriptor)
            finally:
                os.close(descriptor)
    except (OSError, AttributeError):
        fail()


def _write_journal(parent: Path, backup: Path, temporary: Path, stamp: str) -> None:
    content = f"{backup.name}\n{temporary.name}\n{stamp}\n".encode("ascii")
    if len(content) > 512:
        fail()
    for _ in range(TEMP_ATTEMPTS):
        candidate = parent / f".transaction-{secrets.token_hex(16)}.tmp"
        try:
            descriptor = os.open(candidate, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        except FileExistsError:
            continue
        except OSError:
            fail()
        try:
            with os.fdopen(descriptor, "wb") as output:
                output.write(content)
                output.flush()
                os.fsync(output.fileno())
            _durable_replace(candidate, parent / JOURNAL_NAME)
            return
        finally:
            candidate.unlink(missing_ok=True)
    fail()


def _record(parent: Path):
    journal = parent / JOURNAL_NAME
    if not journal.exists():
        return None
    try:
        data = read_regular_file(journal, 512)
        backup, temporary, stamp, trailing = data.decode("ascii").split("\n")
    except (UnicodeError, ValueError, PreparationError):
        fail()
    if trailing or not BACKUP.fullmatch(backup) or not re.fullmatch(r"wheels-new-[0-9a-f]{32}", temporary) or not re.fullmatch(r"[0-9a-f]{64}", stamp):
        fail()
    return parent / backup, parent / temporary, stamp


def _valid(path: Path, stamp=None) -> bool:
    if not path.exists() or is_link(path):
        return False
    if stamp is None:
        try:
            value = read_regular_file(path / ".requirements.sha256", 64).decode("ascii")
        except (PreparationError, UnicodeError):
            return False
        stamp = value
    return bundle_valid(path, stamp)


def _remove(path: Path) -> None:
    if path.exists():
        if is_link(path) or not path.is_dir():
            fail()
        shutil.rmtree(path)


def recover_bundle(parent: Path) -> None:
    parent = safe_directory(parent)
    record = _record(parent)
    if record is None:
        return
    backup, temporary, stamp = record
    destination = parent / "wheels"
    valid_old, valid_new = _valid(backup), _valid(temporary, stamp)
    if destination.exists() and not backup.exists():
        if not _valid(destination):
            fail()
        _remove(temporary)
    elif not destination.exists() and backup.exists():
        if valid_new:
            _durable_replace(temporary, destination)
            _remove(backup)
        elif valid_old:
            _durable_replace(backup, destination)
            _remove(temporary)
        else:
            fail()
    elif destination.exists() and backup.exists():
        if _valid(destination, stamp):
            _remove(backup)
            _remove(temporary)
        elif valid_old:
            if temporary.exists():
                fail()
            _durable_replace(destination, temporary)
            _durable_replace(backup, destination)
            _remove(temporary)
        else:
            fail()
    elif valid_new:
        _durable_replace(temporary, destination)
    else:
        fail()
    if not destination.exists() or not _valid(destination):
        fail()
    (parent / JOURNAL_NAME).unlink(missing_ok=True)


def cleanup_orphans(parent: Path, lock: BundleLock) -> None:
    parent = safe_directory(parent)
    if not isinstance(lock, BundleLock) or not lock.locked or lock.parent != parent:
        fail()
    count = 0
    with os.scandir(parent) as entries:
        for entry in entries:
            count += 1
            if count > MAX_SCAN_ENTRIES:
                fail()
            path = Path(entry.path)
            if BACKUP.fullmatch(entry.name) and not (parent / JOURNAL_NAME).exists():
                fail()
            if TEMPORARY.fullmatch(entry.name):
                if is_link(path) or not entry.is_dir(follow_symlinks=False):
                    fail()
                _remove(path)
            elif JOURNAL_TEMP.fullmatch(entry.name):
                regular_info(path, 512)
                path.unlink()


def publish_bundle(parent: Path, temporary: Path, stamp: str, replace=_durable_replace) -> None:
    parent, temporary = safe_directory(parent), safe_directory(temporary)
    destination = parent / "wheels"
    backup = parent / f"wheels-backup-{secrets.token_hex(16)}"
    if backup.exists() or not _valid(temporary, stamp):
        fail()
    _write_journal(parent, backup, temporary, stamp)
    try:
        if destination.exists():
            replace(destination, backup)
        replace(temporary, destination)
    except OSError:
        fail()
    recover_bundle(parent)
