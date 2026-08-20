import os
import stat
from pathlib import Path

from searxng_safety import fail, has_hardlink, is_link, safe_directory


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
            flags = os.O_RDWR | os.O_CREAT | getattr(os, "O_NOFOLLOW", 0)
            self.descriptor = os.open(self.path, flags, 0o600)
            opened, current = os.fstat(self.descriptor), self.path.lstat()
            if (
                not stat.S_ISREG(opened.st_mode)
                or is_link(self.path)
                or has_hardlink(self.path, current)
            ):
                fail()
            if (
                opened.st_dev
                and opened.st_ino
                and (opened.st_dev, opened.st_ino) != (current.st_dev, current.st_ino)
            ):
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
            fail()
        finally:
            os.close(self.descriptor)
            self.descriptor = None
            self.locked = False


def durable_replace(source: Path, destination: Path) -> None:
    try:
        if os.name == "nt":
            import ctypes

            if not ctypes.windll.kernel32.MoveFileExW(
                str(source), str(destination), 0x1 | 0x8
            ):
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


def create_metadata(directory: Path, name: str, content: bytes, maximum: int) -> None:
    parent = safe_directory(directory)
    if (
        name not in {".runtime.json", ".requirements.sha256"}
        or type(content) is not bytes
        or len(content) > maximum
        or os.path.lexists(parent / name)
    ):
        fail()
    path = parent / name
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0)
    try:
        descriptor = os.open(path, flags, 0o600)
    except OSError:
        fail()
    try:
        opened, current = os.fstat(descriptor), path.lstat()
        if (
            not stat.S_ISREG(opened.st_mode)
            or has_hardlink(path, current)
            or opened.st_size != 0
            or (opened.st_dev, opened.st_ino) != (current.st_dev, current.st_ino)
        ):
            fail()
        view = memoryview(content)
        while view:
            written = os.write(descriptor, view)
            if written <= 0:
                fail()
            view = view[written:]
        if os.fstat(descriptor).st_size != len(content):
            fail()
        os.fsync(descriptor)
        final = path.lstat()
        if (
            not stat.S_ISREG(final.st_mode)
            or has_hardlink(path, final)
            or final.st_size != len(content)
            or (opened.st_dev, opened.st_ino) != (final.st_dev, final.st_ino)
        ):
            fail()
    except OSError:
        fail()
    finally:
        os.close(descriptor)
