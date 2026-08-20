import hashlib
import os
import re
import secrets
import stat
from pathlib import Path, PurePosixPath

from searxng_archive import MAX_ARCHIVE_ENTRIES, MAX_MEMBER_BYTES, is_metadata
from searxng_runtime_manifest import (
    MANIFEST_NAME,
    MAX_MANIFEST_BYTES,
    RuntimeIdentity,
    runtime_identity,
)
from searxng_safety import (
    PreparationError,
    fail,
    has_hardlink,
    hash_regular_file,
    is_link,
    read_regular_file,
    regular_info,
    safe_directory,
)

MAX_WHEEL_BYTES = 150 * 1024 * 1024
MAX_WHEEL_FILES = 512
STAMP_NAME = ".requirements.sha256"
STAMP_BYTES = 64
STAMP_PATTERN = re.compile(r"[0-9a-f]{64}")
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
                    if (
                        count > MAX_ARCHIVE_ENTRIES
                        or is_metadata(PurePosixPath(entry.name))
                        or is_link(Path(entry.path))
                    ):
                        fail()
                    if stat.S_ISDIR(info.st_mode):
                        pending.append(Path(entry.path))
                    elif not stat.S_ISREG(info.st_mode) or has_hardlink(
                        Path(entry.path), info
                    ):
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


def bundle_valid(
    directory: Path,
    *,
    expected_identity: RuntimeIdentity,
    needs_stamp: bool = True,
) -> bool:
    try:
        identity = runtime_identity(expected_identity.stamp, expected_identity.manifest)
    except (AttributeError, PreparationError, ValueError):
        return False
    return _bundle_valid(directory, identity, needs_stamp)


def legacy_bundle_valid(directory: Path, expected_stamp: str) -> bool:
    """Accept pre-manifest bundles only while recovering a legacy journal."""
    if (
        type(expected_stamp) is not str
        or STAMP_PATTERN.fullmatch(expected_stamp) is None
    ):
        return False
    return _bundle_valid(directory, None, True, expected_stamp)


def rollback_bundle_valid(directory: Path) -> bool:
    try:
        directory = safe_directory(directory)
        stamp = read_regular_file(directory / STAMP_NAME, STAMP_BYTES).decode("ascii")
        manifest = directory / MANIFEST_NAME
        if not os.path.lexists(manifest):
            return legacy_bundle_valid(directory, stamp)
        identity = runtime_identity(
            stamp, read_regular_file(manifest, MAX_MANIFEST_BYTES)
        )
        return bundle_valid(directory, expected_identity=identity)
    except (OSError, UnicodeError, PreparationError, ValueError):
        return False


def _bundle_valid(
    directory: Path,
    expected_identity: RuntimeIdentity | None,
    needs_stamp: bool,
    legacy_stamp: str | None = None,
) -> bool:
    try:
        wheels, total, stamp, runtime = 0, 0, None, None
        with os.scandir(safe_directory(directory)) as entries:
            for entry in entries:
                path = Path(entry.path)
                if entry.name == STAMP_NAME:
                    if not needs_stamp:
                        return False
                    stamp = read_regular_file(path, STAMP_BYTES).decode("ascii")
                    if STAMP_PATTERN.fullmatch(stamp) is None:
                        return False
                    continue
                if entry.name == MANIFEST_NAME:
                    if expected_identity is None:
                        return False
                    runtime = read_regular_file(path, MAX_MANIFEST_BYTES)
                    continue
                info = entry.stat(follow_symlinks=False)
                if (
                    not entry.name.endswith(".whl")
                    or is_link(path)
                    or not stat.S_ISREG(info.st_mode)
                    or has_hardlink(path, info)
                ):
                    return False
                wheels, total = wheels + 1, total + info.st_size
                if wheels > MAX_WHEEL_FILES or total > MAX_WHEEL_BYTES:
                    return False
        if expected_identity is not None:
            stamp_valid = stamp is not None and secrets.compare_digest(
                stamp, expected_identity.stamp
            )
            runtime_valid = runtime is not None and secrets.compare_digest(
                runtime_identity(expected_identity.stamp, runtime).manifest,
                expected_identity.manifest,
            )
        else:
            stamp_valid = stamp is not None and secrets.compare_digest(
                stamp, legacy_stamp
            )
            runtime_valid = runtime is None
        return wheels > 0 and runtime_valid and (not needs_stamp or stamp_valid)
    except (OSError, UnicodeError, PreparationError, ValueError):
        return False
