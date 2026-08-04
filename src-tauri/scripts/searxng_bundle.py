import hashlib
import os
import re
import secrets
import stat
from pathlib import Path, PurePosixPath

from searxng_archive import MAX_ARCHIVE_ENTRIES, MAX_MEMBER_BYTES, is_metadata
from searxng_safety import PreparationError, fail, has_hardlink, hash_regular_file, is_link, read_regular_file, regular_info, safe_directory

MAX_WHEEL_BYTES = 150 * 1024 * 1024
MAX_WHEEL_FILES = 512
STAMP_NAME = ".requirements.sha256"
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
