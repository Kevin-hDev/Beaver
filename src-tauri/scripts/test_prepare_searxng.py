import hashlib
import io
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from prepare_searxng import main, prepare, safe_extract
from searxng_archive import MAX_ARCHIVE_ENTRIES, MAX_MEMBER_BYTES, preflight_tar
from searxng_bundle import (
    MAX_WHEEL_BYTES,
    bundle_valid,
    legacy_bundle_valid,
    temporary_directory,
)
from searxng_runtime_manifest import (
    MANIFEST_NAME,
    RuntimeIdentity,
    build_manifest,
    runtime_identity,
)
from searxng_safety import (
    PathValidationError,
    PreparationError,
    hash_regular_file,
    validate_archive_path,
)
from searxng_transaction import (
    cleanup_orphans,
    publish_bundle,
    recover_bundle,
)
from searxng_transaction_fs import BundleLock, durable_replace
from searxng_transaction_journal import JournalRecord, write_journal


class ZeroStream:
    def __init__(self, size):
        self.remaining = size

    def read(self, size):
        chunk = min(size, self.remaining)
        self.remaining -= chunk
        return b"\0" * chunk


def tar_header(name, size, typeflag=b"0", base256=False):
    header = bytearray(512)
    name_bytes = name.encode("ascii")
    header[: len(name_bytes)] = name_bytes
    if base256:
        encoded = size.to_bytes(12, "big")
        header[124:136] = bytes([encoded[0] | 0x80]) + encoded[1:]
    else:
        header[124:136] = f"{size:011o}\0".encode("ascii")
    header[136:148] = b"00000000000\0"
    header[148:156] = b"        "
    header[156:157] = typeflag
    header[257:263] = b"ustar\0"
    header[263:265] = b"00"
    header[148:156] = f"{sum(header):06o}\0 ".encode("ascii")
    return bytes(header)


def compact_tar(path, records):
    raw = b"".join(tar_header(*record) for record in records) + b"\0" * 1024
    import gzip

    path.write_bytes(gzip.compress(raw))


class PrepareSearxngTests(unittest.TestCase):
    def setUp(self):
        self.temp = Path(tempfile.mkdtemp(prefix="searxng-test-")).resolve()
        self.destination = self.temp / "destination"
        self.destination.mkdir()

    def tearDown(self):
        shutil.rmtree(self.temp, ignore_errors=True)

    def archive(self, entries):
        path = self.temp / f"archive-{len(list(self.temp.glob('archive-*')))}.tar.gz"
        with tarfile.open(path, "w:gz") as output:
            for name, content in entries.items():
                if isinstance(content, tarfile.TarInfo):
                    output.addfile(
                        content,
                        ZeroStream(content.size)
                        if content.isreg() and content.size
                        else None,
                    )
                    continue
                info = tarfile.TarInfo(name)
                info.size = len(content)
                output.addfile(info, io.BytesIO(content))
        return path

    def source_root(self):
        root = self.temp / "root" / "resources" / "searxng-sidecar" / "source"
        root.mkdir(parents=True)
        (root / "requirements.txt").write_bytes(b"searxng==1\n")
        (root / "setup.py").write_bytes(b"from setuptools import setup\n")
        return root.parents[2]

    def source_hash(self, source):
        outer = hashlib.sha256()
        for name in ("requirements.txt", "setup.py"):
            inner = hashlib.sha256((source / name).read_bytes()).digest()
            outer.update(inner)
        return outer.hexdigest()

    def downloader(self, wheels, calls):
        def run(args, **kwargs):
            calls.append((args, kwargs))
            if len(calls) == 2:
                Path(args[args.index("--dest") + 1], "ok.whl").write_bytes(b"wheel")

        return run

    def runtime_manifest(self, stamp, minor=None):
        return build_manifest(
            "cpython",
            3,
            sys.version_info.minor if minor is None else minor,
            stamp,
        )

    def identity(self, stamp, minor=None):
        return runtime_identity(stamp, self.runtime_manifest(stamp, minor))

    def write_bundle(self, directory, stamp, runtime=None):
        directory.mkdir()
        (directory / "ok.whl").write_bytes(b"wheel")
        (directory / ".requirements.sha256").write_text(stamp, encoding="ascii")
        if runtime is not None:
            (directory / MANIFEST_NAME).write_bytes(runtime)

    def write_current_journal(self, sidecar, backup, temporary, identity):
        write_journal(
            sidecar,
            JournalRecord(backup.name, temporary.name, identity.stamp, identity),
            durable_replace,
        )

    def write_legacy_journal(self, sidecar, backup, temporary, stamp):
        (sidecar / ".prepare.transaction").write_bytes(
            f"{backup.name}\n{temporary.name}\n{stamp}\n".encode("ascii")
        )

    def recovery_root(self, suffix):
        root = self.temp / suffix
        source = root / "resources" / "searxng-sidecar" / "source"
        source.mkdir(parents=True)
        (source / "requirements.txt").write_bytes(b"searxng==1\n")
        (source / "setup.py").write_bytes(b"from setuptools import setup\n")
        return root, source.parent, source

    def test_rejects_empty_and_unsafe_archive_members(self):
        with self.assertRaises(PreparationError):
            safe_extract(self.archive({}), self.destination)
        for name in ("../escape.txt", "/absolute.txt", "..\\escape.txt"):
            with self.subTest(name=name), self.assertRaises(PreparationError):
                safe_extract(self.archive({name: b"blocked"}), self.destination)

    def test_preflight_rejects_compact_pax_gnu_and_cumulative_tar_bombs(self):
        pax = self.temp / "pax.tar.gz"
        compact_tar(pax, [("pax", MAX_MEMBER_BYTES + 1, b"x", False)])
        with self.assertRaises(PreparationError):
            preflight_tar(pax)
        gnu = self.temp / "gnu.tar.gz"
        compact_tar(gnu, [("long", MAX_MEMBER_BYTES + 1, b"L", True)])
        with self.assertRaises(PreparationError):
            preflight_tar(gnu)
        many = self.temp / "many-raw.tar.gz"
        compact_tar(
            many,
            [(str(index), 0, b"0", False) for index in range(MAX_ARCHIVE_ENTRIES + 1)],
        )
        with self.assertRaises(PreparationError):
            preflight_tar(many)
        cumulative = self.archive({"one": b"123456", "two": b"123456"})
        with patch("searxng_archive.MAX_ARCHIVE_TOTAL_BYTES", 10):
            with self.assertRaises(PreparationError):
                preflight_tar(cumulative)

    def test_rejects_windows_aliases_ads_and_nonportable_segments(self):
        for name in (
            "source/file:stream",
            "source/C:evil",
            "source/CON.txt",
            "source/name. ",
            "source/a//b",
            "source/é.txt",
        ):
            with self.subTest(name=name):
                with self.assertRaises(PathValidationError):
                    validate_archive_path(name, set())
        seen = set()
        validate_archive_path("source/File.txt", seen)
        with self.assertRaises(PathValidationError):
            validate_archive_path("source/file.TXT", seen)

    def test_validates_metadata_paths_before_ignoring_them(self):
        for name in ("../._ignored", "__MACOSX/CON.txt", "__MACOSX/file:stream"):
            with self.subTest(name=name), self.assertRaises(PreparationError):
                safe_extract(self.archive({name: b"x"}), self.destination)

    def test_recovery_uses_the_transaction_stamp_before_current_requirements(self):
        root = self.source_root()
        sidecar = root / "resources" / "searxng-sidecar"
        transaction_stamp, current_stamp = "a" * 64, "b" * 64
        destination = sidecar / "wheels"
        destination.mkdir()
        (destination / "old.whl").write_bytes(b"old")
        (destination / ".requirements.sha256").write_text(
            current_stamp, encoding="ascii"
        )
        temporary = sidecar / ("wheels-new-" + "1" * 32)
        temporary.mkdir()
        (temporary / "new.whl").write_bytes(b"new")
        (temporary / ".requirements.sha256").write_text(
            transaction_stamp, encoding="ascii"
        )
        journal = f"wheels-backup-{'2' * 32}\n{temporary.name}\n{transaction_stamp}\n"
        (sidecar / ".prepare.transaction").write_bytes(journal.encode("ascii"))
        recover_bundle(sidecar)
        self.assertEqual((destination / "old.whl").read_bytes(), b"old")
        self.assertFalse(temporary.exists())
        self.assertFalse((sidecar / ".prepare.transaction").exists())

    def test_rejects_new_transaction_with_an_incompatible_runtime(self):
        root = self.source_root()
        sidecar = root / "resources" / "searxng-sidecar"
        stamp = self.source_hash(sidecar / "source")
        temporary = sidecar / ("wheels-new-" + "1" * 32)
        self.write_bundle(temporary, stamp, self.runtime_manifest(stamp, 13))

        with self.assertRaises(PreparationError):
            publish_bundle(
                sidecar,
                temporary,
                self.identity(stamp, 14),
            )

    def test_rejects_invalid_duplicate_and_oversized_runtime_journals(self):
        root = self.source_root()
        sidecar = root / "resources" / "searxng-sidecar"
        stamp = self.source_hash(sidecar / "source")
        prefix = f"wheels-backup-{'a' * 32}\nwheels-new-{'b' * 32}\n{stamp}\n"
        duplicate = (
            b'{"schema_version":1,"schema_version":1,"implementation":"cpython",'
            b'"major":3,"minor":14,"requirements_sha256":"' + b"a" * 64 + b'"}'
        )
        invalid = b"{}"
        oversized = b"x" * 1025

        for runtime in (duplicate, invalid, oversized):
            with self.subTest(runtime=runtime[:8]):
                (sidecar / ".prepare.transaction").write_bytes(
                    prefix.encode("ascii") + runtime + b"\n"
                )
                with self.assertRaises(PreparationError):
                    recover_bundle(sidecar)

    def test_legacy_journal_recovers_but_its_bundle_is_rebuilt_before_reuse(self):
        root = self.source_root()
        sidecar = root / "resources" / "searxng-sidecar"
        source = sidecar / "source"
        stamp = self.source_hash(source)
        destination = sidecar / "wheels"
        self.write_bundle(destination, stamp)
        backup = sidecar / ("wheels-backup-" + "c" * 32)
        temporary = sidecar / ("wheels-new-" + "d" * 32)
        os.replace(destination, backup)
        temporary.mkdir()
        (sidecar / ".prepare.transaction").write_bytes(
            f"{backup.name}\n{temporary.name}\n{stamp}\n".encode("ascii")
        )

        recover_bundle(sidecar)

        self.assertFalse((destination / MANIFEST_NAME).exists())
        self.assertFalse((sidecar / ".prepare.transaction").exists())
        calls = []
        prepare(root, self.downloader(destination, calls))
        self.assertEqual(len(calls), 2)
        self.assertTrue((destination / MANIFEST_NAME).is_file())

    def test_rejects_manifest_bundles_under_a_runtime_free_journal(self):
        root = self.source_root()
        sidecar = root / "resources" / "searxng-sidecar"
        stamp = self.source_hash(sidecar / "source")
        runtime = self.runtime_manifest(stamp)
        destination = sidecar / "wheels"
        self.write_bundle(destination, stamp, runtime)
        backup = sidecar / ("wheels-backup-" + "e" * 32)
        temporary = sidecar / ("wheels-new-" + "f" * 32)
        os.replace(destination, backup)
        self.write_bundle(temporary, stamp, runtime)
        (sidecar / ".prepare.transaction").write_bytes(
            f"{backup.name}\n{temporary.name}\n{stamp}\n".encode("ascii")
        )

        with self.assertRaises(PreparationError):
            recover_bundle(sidecar)

    def test_legacy_recovery_binds_new_candidates_to_the_journal_stamp(self):
        cases = {
            "after-journal": ("old", "journal"),
            "after-backup": ("new", "backup"),
            "after-temp-to-destination": ("new", "destination"),
            "temporary-absent": ("old", "absent"),
            "temporary-corrupt": ("old", "corrupt"),
            "temporary-wrong-stamp": ("old", "wrong-stamp"),
            "no-old-temporary": ("new", "no-old"),
        }
        old_stamp, journal_stamp, wrong_stamp = "a" * 64, "b" * 64, "c" * 64
        for name, (expected, state) in cases.items():
            with self.subTest(name=name):
                _root, sidecar, _source = self.recovery_root(f"legacy-{name}")
                destination = sidecar / "wheels"
                backup = sidecar / ("wheels-backup-" + "1" * 32)
                temporary = sidecar / ("wheels-new-" + "2" * 32)
                self.write_bundle(destination, old_stamp)
                self.write_bundle(temporary, journal_stamp)
                self.write_legacy_journal(sidecar, backup, temporary, journal_stamp)

                if state in {
                    "backup",
                    "destination",
                    "absent",
                    "corrupt",
                    "wrong-stamp",
                }:
                    os.replace(destination, backup)
                if state == "destination":
                    os.replace(temporary, destination)
                elif state == "absent":
                    shutil.rmtree(temporary)
                elif state == "corrupt":
                    (temporary / "ok.whl").unlink()
                elif state == "wrong-stamp":
                    (temporary / ".requirements.sha256").write_text(
                        wrong_stamp,
                        encoding="ascii",
                    )
                elif state == "no-old":
                    shutil.rmtree(destination)

                recover_bundle(sidecar)

                expected_stamp = journal_stamp if expected == "new" else old_stamp
                self.assertEqual(
                    (destination / ".requirements.sha256").read_text("ascii"),
                    expected_stamp,
                )
                self.assertFalse((sidecar / ".prepare.transaction").exists())

    def test_legacy_bundle_requires_lowercase_hex_stamps(self):
        root, sidecar, _source = self.recovery_root("legacy-stamp")
        wheels = sidecar / "wheels"
        self.write_bundle(wheels, "A" * 64)

        self.assertFalse(legacy_bundle_valid(wheels, "A" * 64))

    def test_cleanup_orphans_is_bounded_and_never_touches_unrelated_paths(self):
        root = self.source_root()
        sidecar = root / "resources" / "searxng-sidecar"
        orphan = sidecar / ("wheels-new-" + "3" * 32)
        orphan.mkdir()
        journal_temp = sidecar / (".transaction-" + "4" * 32 + ".tmp")
        journal_temp.write_bytes(b"partial")
        keep = sidecar / "source"
        with BundleLock(sidecar) as bundle_lock:
            cleanup_orphans(sidecar, bundle_lock)
        self.assertFalse(orphan.exists())
        self.assertFalse(journal_temp.exists())
        self.assertTrue(keep.exists())

    def test_hash_rejects_growth_after_the_verified_open(self):
        path = self.temp / "requirements.txt"
        path.write_bytes(b"x")

        def grow(_fd):
            with path.open("ab") as output:
                output.write(b"changed")

        with self.assertRaises(PreparationError):
            hash_regular_file(path, MAX_MEMBER_BYTES, after_open=grow)

    def test_hash_reads_windows_crlf_as_binary(self):
        path = self.temp / "requirements-crlf.txt"
        content = b"first\r\nsecond\r\n"
        path.write_bytes(content)
        self.assertEqual(
            hash_regular_file(path, MAX_MEMBER_BYTES), hashlib.sha256(content).digest()
        )

    def test_rejects_links_special_members_and_oversized_file(self):
        for kind in (tarfile.SYMTYPE, tarfile.LNKTYPE, tarfile.FIFOTYPE):
            info = tarfile.TarInfo("source/unsafe")
            info.type = kind
            info.linkname = "target"
            with self.subTest(kind=kind), self.assertRaises(PreparationError):
                safe_extract(self.archive({"ignored": info}), self.destination)
        info = tarfile.TarInfo("source/large")
        info.size = MAX_MEMBER_BYTES + 1
        with self.assertRaises(PreparationError):
            safe_extract(self.archive({"ignored": info}), self.destination)

    def test_ignores_exact_nonportable_upstream_templates(self):
        apache = tarfile.TarInfo("source/utils/templates/etc/apache2")
        apache.type = tarfile.SYMTYPE
        apache.linkname = "httpd"
        socket = "source/utils/templates/etc/httpd/sites-available/searxng.conf:socket"
        archive = self.archive(
            {
                "source/requirements.txt": b"requirement\n",
                apache.name: apache,
                socket: b"socket template",
            }
        )

        safe_extract(archive, self.destination)

        self.assertEqual(
            (self.destination / "source" / "requirements.txt").read_bytes(),
            b"requirement\n",
        )
        self.assertFalse((self.destination / apache.name).exists())

        malicious = tarfile.TarInfo(apache.name)
        malicious.type = tarfile.SYMTYPE
        malicious.linkname = "../../escape"
        rejected = self.temp / "rejected"
        rejected.mkdir()
        with self.assertRaises(PreparationError):
            safe_extract(self.archive({malicious.name: malicious}), rejected)

    def test_rejects_unreadable_archive_content(self):
        archive = self.archive({"source/file": b"valid content"})
        archive.write_bytes(b"not a readable archive")
        with self.assertRaises(PreparationError):
            safe_extract(archive, self.destination)

    def test_ignores_macos_metadata_without_accepting_too_many_entries(self):
        archive = self.archive(
            {
                "source/requirements.txt": b"x",
                "__MACOSX/noise": b"x",
                "source/._hidden": b"x",
            }
        )
        safe_extract(archive, self.destination)
        self.assertEqual(
            (self.destination / "source" / "requirements.txt").read_bytes(), b"x"
        )
        self.assertFalse((self.destination / "__MACOSX").exists())
        many = {f"source/{index}": b"" for index in range(MAX_ARCHIVE_ENTRIES + 1)}
        many_destination = self.temp / "many"
        many_destination.mkdir()
        with self.assertRaises(PreparationError):
            safe_extract(self.archive(many), many_destination)

    def test_refuses_a_symlink_parent_in_destination(self):
        linked = self.temp / "linked"
        linked.mkdir()
        if os.name == "nt":
            node = shutil.which("node")
            self.assertIsNotNone(node)
            subprocess.run(
                [
                    node,
                    "-e",
                    "require('node:fs').symlinkSync("
                    "process.argv[1], process.argv[2], 'junction')",
                    str(linked),
                    str(self.destination / "source"),
                ],
                check=True,
                shell=False,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
        else:
            os.symlink(linked, self.destination / "source", target_is_directory=True)
        with self.assertRaises(PreparationError):
            safe_extract(self.archive({"source/file": b"x"}), self.destination)

    def test_reuses_only_a_valid_stamp_and_regular_wheels(self):
        root = self.source_root()
        source = root / "resources" / "searxng-sidecar" / "source"
        wheels = source.parent / "wheels"
        wheels.mkdir()
        (wheels / "ok.whl").write_bytes(b"wheel")
        stamp = self.source_hash(source)
        (wheels / ".requirements.sha256").write_text(stamp, encoding="ascii")
        (wheels / MANIFEST_NAME).write_bytes(self.runtime_manifest(stamp))
        calls = []
        prepare(root, lambda *_args, **_kwargs: self.fail("pip must not run"))
        self.assertEqual(calls, [])

    def test_rebuilds_wheels_from_a_different_cpython_minor(self):
        root = self.source_root()
        source = root / "resources" / "searxng-sidecar" / "source"
        wheels = source.parent / "wheels"
        wheels.mkdir()
        stamp = self.source_hash(source)
        expected = build_manifest("cpython", 3, 14, stamp)
        (wheels / "old.whl").write_bytes(b"old wheel")
        (wheels / ".requirements.sha256").write_text(stamp, encoding="ascii")
        (wheels / MANIFEST_NAME).write_bytes(build_manifest("cpython", 3, 13, stamp))
        calls = []

        with patch("prepare_searxng.current_manifest", return_value=expected):
            prepare(root, self.downloader(wheels, calls))

        self.assertEqual(len(calls), 2)
        self.assertEqual((wheels / MANIFEST_NAME).read_bytes(), expected)

    def test_bundle_requires_matching_canonical_regular_runtime_manifest(self):
        root = self.source_root()
        source = root / "resources" / "searxng-sidecar" / "source"
        wheels = source.parent / "wheels"
        wheels.mkdir()
        stamp = self.source_hash(source)
        expected = self.runtime_manifest(stamp)
        (wheels / "ok.whl").write_bytes(b"wheel")
        (wheels / ".requirements.sha256").write_text(stamp, encoding="ascii")
        (wheels / MANIFEST_NAME).write_bytes(expected)

        self.assertTrue(
            bundle_valid(
                wheels,
                expected_identity=self.identity(stamp),
            )
        )
        (wheels / MANIFEST_NAME).write_bytes(expected + b"\n")
        self.assertFalse(
            bundle_valid(
                wheels,
                expected_identity=self.identity(stamp),
            )
        )
        (wheels / MANIFEST_NAME).write_bytes(self.runtime_manifest(stamp, 13))
        self.assertFalse(
            bundle_valid(
                wheels,
                expected_identity=self.identity(stamp),
            )
        )
        (wheels / MANIFEST_NAME).unlink()
        self.assertFalse(
            bundle_valid(
                wheels,
                expected_identity=self.identity(stamp),
            )
        )

    def test_prepublication_bundle_rejects_a_downloaded_stamp(self):
        root = self.source_root()
        source = root / "resources" / "searxng-sidecar" / "source"
        wheels = source.parent / "wheels"
        wheels.mkdir()
        stamp = self.source_hash(source)
        expected = self.runtime_manifest(stamp)
        (wheels / "ok.whl").write_bytes(b"wheel")
        (wheels / MANIFEST_NAME).write_bytes(expected)
        (wheels / ".requirements.sha256").write_text(stamp, encoding="ascii")

        self.assertFalse(
            bundle_valid(
                wheels,
                expected_identity=self.identity(stamp),
                needs_stamp=False,
            )
        )

    def test_valid_stamp_without_regular_wheels_rebuilds(self):
        root = self.source_root()
        source = root / "resources" / "searxng-sidecar" / "source"
        wheels = source.parent / "wheels"
        wheels.mkdir()
        (wheels / ".requirements.sha256").write_text(
            self.source_hash(source), encoding="ascii"
        )
        calls = []
        prepare(root, self.downloader(wheels, calls))
        self.assertEqual(len(calls), 2)
        self.assertTrue((wheels / "ok.whl").is_file())

    def test_downloads_with_argument_lists_and_rebuilds_an_invalid_stamp(self):
        root = self.source_root()
        wheels = root / "resources" / "searxng-sidecar" / "wheels"
        wheels.mkdir()
        (wheels / ".requirements.sha256").write_text("truncated", encoding="ascii")
        calls = []
        prepare(root, self.downloader(wheels, calls))
        self.assertEqual(len(calls), 2)
        self.assertEqual(
            calls[0][0][1:5], ["-m", "pip", "download", "--only-binary=:all:"]
        )
        self.assertIn("--dest", calls[0][0])
        self.assertEqual(calls[0][1]["check"], True)
        self.assertEqual(calls[0][1]["shell"], False)
        self.assertTrue((wheels / "ok.whl").is_file())
        self.assertRegex(
            (wheels / ".requirements.sha256").read_text(encoding="ascii"),
            r"^[0-9a-f]{64}$",
        )

    def test_extracts_the_source_archive_when_the_source_directory_is_absent(self):
        root = self.temp / "archive-root"
        sidecar = root / "resources" / "searxng-sidecar"
        sidecar.mkdir(parents=True)
        shutil.copyfile(
            self.archive(
                {
                    "source/requirements.txt": b"searxng==1\n",
                    "source/setup.py": b"from setuptools import setup\n",
                }
            ),
            sidecar / "source.tar.gz",
        )
        calls = []
        prepare(root, self.downloader(sidecar / "wheels", calls))
        self.assertEqual(len(calls), 2)
        self.assertTrue((sidecar / "wheels" / "ok.whl").is_file())

    def test_rejects_missing_required_files_and_preserves_wheels_on_pip_failure(self):
        root = self.source_root()
        source = root / "resources" / "searxng-sidecar" / "source"
        (source / "setup.py").unlink()
        with self.assertRaisesRegex(PreparationError, "^SearXNG preparation failed$"):
            prepare(root, lambda *_args, **_kwargs: None)
        (source / "setup.py").write_text("setup", encoding="ascii")
        wheels = source.parent / "wheels"
        wheels.mkdir()
        (wheels / "old.whl").write_bytes(b"old")
        (wheels / ".requirements.sha256").write_text("0" * 64, encoding="ascii")

        def fail_download(args, **_kwargs):
            Path(args[args.index("--dest") + 1], "partial.whl").write_bytes(b"partial")
            raise RuntimeError("pip detail must not escape")

        with self.assertRaisesRegex(PreparationError, "^SearXNG preparation failed$"):
            prepare(root, fail_download)
        self.assertEqual((wheels / "old.whl").read_bytes(), b"old")
        self.assertFalse(any(wheels.parent.glob("wheels-new-*")))

    def test_rejects_excessive_downloaded_wheels_before_replacing_existing_bundle(self):
        root = self.source_root()
        wheels = root / "resources" / "searxng-sidecar" / "wheels"
        wheels.mkdir()
        (wheels / "old.whl").write_bytes(b"old")
        (wheels / ".requirements.sha256").write_text("0" * 64, encoding="ascii")

        def download_many(args, **_kwargs):
            directory = Path(args[args.index("--dest") + 1])
            if args[-1] == "wheel":
                for index in range(513):
                    (directory / f"{index}.whl").write_bytes(b"")

        with self.assertRaises(PreparationError):
            prepare(root, download_many)
        self.assertEqual((wheels / "old.whl").read_bytes(), b"old")
        self.assertFalse(any(wheels.parent.glob("wheels-new-*")))

        def download_large(args, **_kwargs):
            directory = Path(args[args.index("--dest") + 1])
            if args[-1] == "wheel":
                with (directory / "too-large.whl").open("wb") as output:
                    output.truncate(MAX_WHEEL_BYTES + 1)

        with self.assertRaises(PreparationError):
            prepare(root, download_large)
        self.assertEqual((wheels / "old.whl").read_bytes(), b"old")

    def test_rejects_duplicate_archive_members_from_an_ordered_input(self):
        archive = self.temp / "duplicates.tar.gz"
        with tarfile.open(archive, "w:gz") as output:
            for content in (b"first", b"second"):
                info = tarfile.TarInfo("source/file")
                info.size = len(content)
                output.addfile(info, io.BytesIO(content))
        with self.assertRaises(PreparationError):
            safe_extract(archive, self.destination)

    def test_temp_collision_lock_and_recovery_preserve_the_old_bundle(self):
        root = self.source_root()
        sidecar = root / "resources" / "searxng-sidecar"
        source = sidecar / "source"
        stamp = self.source_hash(source)
        old_stamp = "0" * 64
        wheels = sidecar / "wheels"
        wheels.mkdir()
        (wheels / "old.whl").write_bytes(b"old")
        (wheels / ".requirements.sha256").write_text(old_stamp, encoding="ascii")
        (sidecar / ("wheels-new-" + "a" * 32)).mkdir()
        tokens = iter(["a" * 32, "b" * 32])
        temporary = temporary_directory(
            sidecar, "wheels-new-", lambda _size: next(tokens)
        )
        (temporary / "new.whl").write_bytes(b"new")
        (temporary / ".requirements.sha256").write_text(stamp, encoding="ascii")
        runtime = self.runtime_manifest(stamp)
        (temporary / MANIFEST_NAME).write_bytes(runtime)
        calls = []

        def interrupted(source_path, destination_path):
            calls.append((source_path, destination_path))
            if len(calls) == 2:
                raise OSError("interrupted")
            os.replace(source_path, destination_path)

        with self.assertRaises(PreparationError):
            publish_bundle(
                sidecar,
                temporary,
                self.identity(stamp),
                replace=interrupted,
            )
        self.assertFalse(wheels.exists())
        self.assertTrue(any(sidecar.glob("wheels-backup-*")))
        recover_bundle(sidecar)
        self.assertEqual((wheels / "new.whl").read_bytes(), b"new")
        self.assertFalse((sidecar / ".prepare.transaction").exists())
        with BundleLock(sidecar):
            with self.assertRaises(PreparationError):
                with BundleLock(sidecar):
                    pass

    def test_rejects_invalid_cli_roots_generically(self):
        for root in ("", "relative", "C:\\repo\\..\\escape", "bad\nroot", "x" * 4097):
            with self.subTest(root=root[:8]):
                self.assertEqual(main(["--root", root]), 1)

    def test_rejects_non_wheel_and_hardlinked_wheel_bundles(self):
        root = self.source_root()
        source = root / "resources" / "searxng-sidecar" / "source"
        stamp = self.source_hash(source)
        wheels = source.parent / "wheels"
        wheels.mkdir()
        (wheels / "note.txt").write_bytes(b"not a wheel")
        (wheels / ".requirements.sha256").write_text(stamp, encoding="ascii")
        self.assertFalse(
            bundle_valid(
                wheels,
                expected_identity=self.identity(stamp),
            )
        )
        (wheels / "note.txt").unlink()
        wheel = wheels / "one.whl"
        wheel.write_bytes(b"wheel")
        os.link(wheel, wheels / "two.whl")
        self.assertFalse(
            bundle_valid(
                wheels,
                expected_identity=self.identity(stamp),
            )
        )

    def test_recovery_crash_matrix_keeps_the_new_and_old_identities_separate(self):
        cases = {
            "after-journal": ("old", None),
            "after-backup-temp-missing": ("old", "missing-temp"),
            "after-backup-temp-corrupt": ("old", "corrupt-temp"),
            "after-temp-to-destination": ("new", "new-destination"),
            "new-destination-corrupt": ("old", "corrupt-destination"),
            "no-old-temporary": ("new", "no-old"),
        }
        for name, (expected, mutation) in cases.items():
            with self.subTest(name=name):
                _root, sidecar, _source = self.recovery_root(name)
                old_identity = self.identity("a" * 64, 13)
                new_identity = self.identity("b" * 64, 14)
                destination = sidecar / "wheels"
                backup = sidecar / ("wheels-backup-" + "1" * 32)
                temporary = sidecar / ("wheels-new-" + "2" * 32)
                self.write_bundle(
                    destination,
                    old_identity.stamp,
                    old_identity.manifest,
                )
                self.write_bundle(
                    temporary,
                    new_identity.stamp,
                    new_identity.manifest,
                )
                self.write_current_journal(sidecar, backup, temporary, new_identity)

                if mutation in {
                    "missing-temp",
                    "corrupt-temp",
                    "new-destination",
                    "corrupt-destination",
                }:
                    os.replace(destination, backup)
                if mutation == "missing-temp":
                    shutil.rmtree(temporary)
                elif mutation == "corrupt-temp":
                    (temporary / MANIFEST_NAME).write_bytes(b"invalid")
                elif mutation in {"new-destination", "corrupt-destination"}:
                    os.replace(temporary, destination)
                elif mutation == "no-old":
                    shutil.rmtree(destination)
                if mutation == "corrupt-destination":
                    (destination / MANIFEST_NAME).write_bytes(b"invalid")

                recover_bundle(sidecar)

                expected_identity = old_identity if expected == "old" else new_identity
                self.assertEqual(
                    (destination / MANIFEST_NAME).read_bytes(),
                    expected_identity.manifest,
                )
                self.assertFalse((sidecar / ".prepare.transaction").exists())

    def test_rejects_divergent_stamp_and_runtime_at_all_identity_boundaries(self):
        root, sidecar, _source = self.recovery_root("divergent")
        expected = self.identity("a" * 64, 14)
        divergent = self.identity("b" * 64, 14)
        temporary = sidecar / ("wheels-new-" + "3" * 32)
        self.write_bundle(temporary, divergent.stamp, divergent.manifest)

        self.assertFalse(bundle_valid(temporary, expected_identity=expected))
        with self.assertRaises(PreparationError):
            publish_bundle(sidecar, temporary, expected)
        invalid_identity = RuntimeIdentity(expected.stamp, divergent.manifest)
        with self.assertRaises(PreparationError):
            write_journal(
                sidecar,
                JournalRecord(
                    "wheels-backup-" + "4" * 32,
                    temporary.name,
                    expected.stamp,
                    invalid_identity,
                ),
                durable_replace,
            )
        with self.assertRaises(PreparationError):
            write_journal(
                sidecar,
                JournalRecord(
                    "wheels-backup-" + "4" * 32,
                    temporary.name,
                    divergent.stamp,
                    expected,
                ),
                durable_replace,
            )
        journal = sidecar / ".prepare.transaction"
        journal.write_bytes(
            (
                f"wheels-backup-{'4' * 32}\n{temporary.name}\n"
                f"{expected.stamp}\n{divergent.manifest.decode('ascii')}\n"
            ).encode("ascii")
        )
        with self.assertRaises(PreparationError):
            recover_bundle(sidecar)

    def test_manifest_symlink_is_rejected_without_reading_its_target(self):
        root, sidecar, _source = self.recovery_root("manifest-link")
        identity = self.identity("a" * 64)
        wheels = sidecar / "wheels"
        wheels.mkdir()
        victim = self.temp / "manifest-victim"
        victim.write_bytes(identity.manifest)
        (wheels / "ok.whl").write_bytes(b"wheel")
        (wheels / ".requirements.sha256").write_text(identity.stamp, encoding="ascii")
        os.symlink(victim, wheels / MANIFEST_NAME)

        self.assertFalse(bundle_valid(wheels, expected_identity=identity))
        self.assertEqual(victim.read_bytes(), identity.manifest)

    def test_metadata_creation_refuses_preexisting_untrusted_entries(self):
        kinds = (
            "regular",
            "empty-regular",
            "directory",
            "symlink",
            "broken-symlink",
            "hardlink",
        )
        for metadata_name in (MANIFEST_NAME, ".requirements.sha256"):
            for kind in kinds:
                with self.subTest(metadata_name=metadata_name, kind=kind):
                    root, sidecar, _source = self.recovery_root(
                        f"metadata-{metadata_name[1:]}-{kind}"
                    )
                    victim = self.temp / f"victim-{metadata_name[1:]}-{kind}"
                    victim.write_bytes(b"SAFE")

                    def download(arguments, **_kwargs):
                        if arguments[-1] != "wheel":
                            return
                        directory = Path(arguments[arguments.index("--dest") + 1])
                        (directory / "ok.whl").write_bytes(b"wheel")
                        target = directory / metadata_name
                        if kind == "regular":
                            target.write_bytes(b"PRESENT")
                        elif kind == "empty-regular":
                            target.touch()
                        elif kind == "directory":
                            target.mkdir()
                        elif kind == "symlink":
                            os.symlink(victim, target)
                        elif kind == "broken-symlink":
                            os.symlink(self.temp / "missing-target", target)
                        else:
                            os.link(victim, target)

                    with self.assertRaises(PreparationError):
                        prepare(root, download)
                    self.assertEqual(victim.read_bytes(), b"SAFE")


if __name__ == "__main__":
    unittest.main()
