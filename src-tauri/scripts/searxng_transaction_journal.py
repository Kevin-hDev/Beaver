import os
import re
import secrets
from dataclasses import dataclass
from pathlib import Path

from searxng_bundle import TEMP_ATTEMPTS
from searxng_runtime_manifest import RuntimeIdentity, runtime_identity
from searxng_safety import PreparationError, fail, read_regular_file

MAX_JOURNAL_BYTES = 1024
JOURNAL_NAME = ".prepare.transaction"
BACKUP_PATTERN = re.compile(r"wheels-backup-[0-9a-f]{32}")
WHEELS_TEMP_PATTERN = re.compile(r"wheels-new-[0-9a-f]{32}")


@dataclass(frozen=True)
class JournalRecord:
    backup: str
    temporary: str
    stamp: str
    identity: RuntimeIdentity | None

    @property
    def is_legacy(self) -> bool:
        return self.identity is None


def _record(fields: list[str]) -> JournalRecord:
    if len(fields) == 4:
        backup, temporary, stamp, trailing = fields
        identity = None
    elif len(fields) == 5:
        backup, temporary, stamp, runtime_text, trailing = fields
        identity = runtime_identity(stamp, runtime_text.encode("ascii"))
    else:
        raise ValueError("invalid transaction journal")
    if (
        trailing
        or not BACKUP_PATTERN.fullmatch(backup)
        or not WHEELS_TEMP_PATTERN.fullmatch(temporary)
        or re.fullmatch(r"[0-9a-f]{64}", stamp) is None
    ):
        raise ValueError("invalid transaction journal")
    return JournalRecord(backup, temporary, stamp, identity)


def read_journal(parent: Path) -> JournalRecord | None:
    journal = parent / JOURNAL_NAME
    if not journal.exists():
        return None
    try:
        content = read_regular_file(journal, MAX_JOURNAL_BYTES).decode("ascii")
        fields = content.split("\n")
        return _record(fields)
    except (AttributeError, UnicodeError, ValueError, PreparationError):
        fail()


def write_journal(parent: Path, record: JournalRecord, replace) -> None:
    if not isinstance(record, JournalRecord) or record.is_legacy:
        fail()
    try:
        if not BACKUP_PATTERN.fullmatch(
            record.backup
        ) or not WHEELS_TEMP_PATTERN.fullmatch(record.temporary):
            fail()
        identity = runtime_identity(record.identity.stamp, record.identity.manifest)
        if (
            type(record.stamp) is not str
            or re.fullmatch(r"[0-9a-f]{64}", record.stamp) is None
            or not secrets.compare_digest(record.stamp, identity.stamp)
        ):
            fail()
        runtime = identity.manifest.decode("ascii")
        content = (
            f"{record.backup}\n{record.temporary}\n{identity.stamp}\n{runtime}\n"
        ).encode("ascii")
        checked = _record(content.decode("ascii").split("\n"))
        if checked != JournalRecord(
            record.backup,
            record.temporary,
            identity.stamp,
            identity,
        ):
            fail()
    except (AttributeError, UnicodeError, ValueError, PreparationError):
        fail()
    if len(content) > MAX_JOURNAL_BYTES:
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
            replace(candidate, parent / JOURNAL_NAME)
            return
        finally:
            candidate.unlink(missing_ok=True)
    fail()
