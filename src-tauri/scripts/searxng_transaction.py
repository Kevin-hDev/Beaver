import os
import re
import secrets
import shutil
from enum import Enum
from pathlib import Path

from searxng_bundle import (
    STAMP_BYTES,
    STAMP_NAME,
    bundle_valid,
    legacy_bundle_valid,
    rollback_bundle_valid,
)
from searxng_runtime_manifest import RuntimeIdentity
from searxng_safety import (
    PreparationError,
    fail,
    is_link,
    read_regular_file,
    regular_info,
    safe_directory,
)
from searxng_transaction_fs import BundleLock, durable_replace
from searxng_transaction_journal import (
    BACKUP_PATTERN,
    JOURNAL_NAME,
    JournalRecord,
    read_journal,
    write_journal,
)

MAX_SCAN_ENTRIES = 1024
TEMPORARY = re.compile(r"(?:source-|wheels-new-)[0-9a-f]{32}")
JOURNAL_TEMP = re.compile(r"\.transaction-[0-9a-f]{32}\.tmp")


class RecoveryOutcome(Enum):
    COMMITTED_NEW = "committed-new"
    ROLLED_BACK_OLD = "rolled-back-old"


def _new_valid(path: Path, record: JournalRecord) -> bool:
    if record.identity is None:
        return legacy_bundle_valid(path, record.stamp)
    return bundle_valid(path, expected_identity=record.identity)


def _old_valid(path: Path, identity: RuntimeIdentity | None) -> bool:
    if identity is not None:
        return rollback_bundle_valid(path)
    try:
        stamp = read_regular_file(path / STAMP_NAME, STAMP_BYTES).decode("ascii")
        return legacy_bundle_valid(path, stamp)
    except (OSError, UnicodeError, PreparationError):
        return False


def _remove(path: Path) -> None:
    if path.exists():
        if is_link(path) or not path.is_dir():
            fail()
        shutil.rmtree(path)


def _recover_current(
    destination: Path,
    backup: Path,
    temporary: Path,
    record: JournalRecord,
) -> RecoveryOutcome:
    destination_new = _new_valid(destination, record) if destination.exists() else False
    temporary_new = _new_valid(temporary, record) if temporary.exists() else False
    destination_old = (
        _old_valid(destination, record.identity) if destination.exists() else False
    )
    backup_old = _old_valid(backup, record.identity) if backup.exists() else False

    if backup.exists():
        if temporary_new:
            durable_replace(temporary, destination)
            _remove(backup)
            return RecoveryOutcome.COMMITTED_NEW
        if destination_new and not temporary.exists():
            _remove(backup)
            return RecoveryOutcome.COMMITTED_NEW
        if backup_old:
            _remove(destination)
            durable_replace(backup, destination)
            _remove(temporary)
            return RecoveryOutcome.ROLLED_BACK_OLD
        fail()
    if destination_old:
        _remove(temporary)
        return RecoveryOutcome.ROLLED_BACK_OLD
    if temporary_new:
        durable_replace(temporary, destination)
        return RecoveryOutcome.COMMITTED_NEW
    if destination_new:
        _remove(temporary)
        return RecoveryOutcome.COMMITTED_NEW
    fail()


def recover_bundle(parent: Path) -> None:
    parent = safe_directory(parent)
    record = read_journal(parent)
    if record is None:
        return
    destination = parent / "wheels"
    backup = parent / record.backup
    temporary = parent / record.temporary
    outcome = _recover_current(destination, backup, temporary, record)
    valid = (
        _new_valid(destination, record)
        if outcome is RecoveryOutcome.COMMITTED_NEW
        else _old_valid(destination, record.identity)
    )
    if not valid:
        fail()
    (parent / JOURNAL_NAME).unlink(missing_ok=True)


def cleanup_orphans(parent: Path, lock) -> None:
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
            if (
                BACKUP_PATTERN.fullmatch(entry.name)
                and not (parent / JOURNAL_NAME).exists()
            ):
                fail()
            if TEMPORARY.fullmatch(entry.name):
                if is_link(path) or not entry.is_dir(follow_symlinks=False):
                    fail()
                _remove(path)
            elif JOURNAL_TEMP.fullmatch(entry.name):
                regular_info(path, 1024)
                path.unlink()


def publish_bundle(
    parent: Path, temporary: Path, identity: RuntimeIdentity, replace=durable_replace
) -> None:
    parent, temporary = safe_directory(parent), safe_directory(temporary)
    destination = parent / "wheels"
    backup = parent / f"wheels-backup-{secrets.token_hex(16)}"
    record = JournalRecord(backup.name, temporary.name, identity.stamp, identity)
    if backup.exists() or not _new_valid(temporary, record):
        fail()
    write_journal(
        parent,
        record,
        durable_replace,
    )
    try:
        if destination.exists():
            replace(destination, backup)
        replace(temporary, destination)
    except OSError:
        fail()
    recover_bundle(parent)
