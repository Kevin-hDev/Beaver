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

SCRIPTS = Path(__file__).parent
sys.path.insert(0, str(SCRIPTS))

from prepare_searxng import (
    MAX_ARCHIVE_ENTRIES,
    MAX_MEMBER_BYTES,
    MAX_WHEEL_BYTES,
    PreparationError,
    prepare,
    safe_extract,
    main,
)
from searxng_archive import MAX_ARCHIVE_TOTAL_BYTES, preflight_tar
from searxng_bundle import BundleLock, bundle_valid, publish_bundle, recover_bundle, temporary_directory
from searxng_safety import PathValidationError, hash_regular_file, validate_archive_path


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
    header[:len(name_bytes)] = name_bytes
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
        self.temp = Path(tempfile.mkdtemp(prefix="searxng-test-"))
        self.destination = self.temp / "destination"
        self.destination.mkdir()

    def tearDown(self):
        shutil.rmtree(self.temp, ignore_errors=True)

    def archive(self, entries):
        path = self.temp / f"archive-{len(list(self.temp.glob('archive-*')))}.tar.gz"
        with tarfile.open(path, "w:gz") as output:
            for name, content in entries.items():
                if isinstance(content, tarfile.TarInfo):
                    output.addfile(content, ZeroStream(content.size) if content.isreg() and content.size else None)
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
        compact_tar(many, [(str(index), 0, b"0", False) for index in range(MAX_ARCHIVE_ENTRIES + 1)])
        with self.assertRaises(PreparationError):
            preflight_tar(many)
        total = self.temp / "total.tar.gz"
        compact_tar(total, [("one", MAX_ARCHIVE_TOTAL_BYTES // 2 + 1, b"0", False), ("two", MAX_ARCHIVE_TOTAL_BYTES // 2 + 1, b"0", False)])
        with self.assertRaises(PreparationError):
            preflight_tar(total)

    def test_rejects_windows_aliases_ads_and_nonportable_segments(self):
        for name in ("source/file:stream", "source/C:evil", "source/CON.txt", "source/name. ", "source/a//b", "source/é.txt"):
            with self.subTest(name=name):
                with self.assertRaises(PathValidationError):
                    validate_archive_path(name, set())
        seen = set()
        validate_archive_path("source/File.txt", seen)
        with self.assertRaises(PathValidationError):
            validate_archive_path("source/file.TXT", seen)

    def test_hash_rejects_growth_after_the_verified_open(self):
        path = self.temp / "requirements.txt"
        path.write_bytes(b"x")
        def grow(_fd):
            with path.open("ab") as output:
                output.write(b"changed")
        with self.assertRaises(PreparationError):
            hash_regular_file(path, MAX_MEMBER_BYTES, after_open=grow)

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

    def test_rejects_unreadable_archive_content(self):
        archive = self.archive({"source/file": b"valid content"})
        archive.write_bytes(b"not a readable archive")
        with self.assertRaises(PreparationError):
            safe_extract(archive, self.destination)

    def test_ignores_macos_metadata_without_accepting_too_many_entries(self):
        archive = self.archive({
            "source/requirements.txt": b"x",
            "__MACOSX/noise": b"x",
            "source/._hidden": b"x",
        })
        safe_extract(archive, self.destination)
        self.assertEqual((self.destination / "source" / "requirements.txt").read_bytes(), b"x")
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
            subprocess.run(
                ["cmd.exe", "/d", "/c", "mklink", "/J", str(self.destination / "source"), str(linked)],
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
        (wheels / ".requirements.sha256").write_text(self.source_hash(source), encoding="ascii")
        calls = []
        prepare(root, lambda *_args, **_kwargs: self.fail("pip must not run"))
        self.assertEqual(calls, [])

    def test_valid_stamp_without_regular_wheels_rebuilds(self):
        root = self.source_root()
        source = root / "resources" / "searxng-sidecar" / "source"
        wheels = source.parent / "wheels"
        wheels.mkdir()
        (wheels / ".requirements.sha256").write_text(self.source_hash(source), encoding="ascii")
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
        self.assertEqual(calls[0][0][1:5], ["-m", "pip", "download", "--only-binary=:all:"])
        self.assertIn("--dest", calls[0][0])
        self.assertEqual(calls[0][1]["check"], True)
        self.assertEqual(calls[0][1]["shell"], False)
        self.assertTrue((wheels / "ok.whl").is_file())
        self.assertRegex((wheels / ".requirements.sha256").read_text(encoding="ascii"), r"^[0-9a-f]{64}$")

    def test_extracts_the_source_archive_when_the_source_directory_is_absent(self):
        root = self.temp / "archive-root"
        sidecar = root / "resources" / "searxng-sidecar"
        sidecar.mkdir(parents=True)
        shutil.copyfile(self.archive({
            "source/requirements.txt": b"searxng==1\n",
            "source/setup.py": b"from setuptools import setup\n",
        }), sidecar / "source.tar.gz")
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
        self.assertFalse(any(wheels.parent.glob("wheels-new-*.tmp")))

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
        self.assertFalse(any(wheels.parent.glob("wheels-new-*.tmp")))
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
        wheels = sidecar / "wheels"
        wheels.mkdir()
        (wheels / "old.whl").write_bytes(b"old")
        (wheels / ".requirements.sha256").write_text(stamp, encoding="ascii")
        (sidecar / ("wheels-new-" + "a" * 32)).mkdir()
        tokens = iter(["a" * 32, "b" * 32])
        temporary = temporary_directory(sidecar, "wheels-new-", lambda _size: next(tokens))
        (temporary / "new.whl").write_bytes(b"new")
        (temporary / ".requirements.sha256").write_text(stamp, encoding="ascii")
        calls = []
        def interrupted(source_path, destination_path):
            calls.append((source_path, destination_path))
            if len(calls) == 2:
                raise OSError("interrupted")
            os.replace(source_path, destination_path)
        with self.assertRaises(PreparationError):
            publish_bundle(sidecar, temporary, stamp, replace=interrupted)
        self.assertFalse(wheels.exists())
        self.assertTrue(any(sidecar.glob("wheels-backup-*")))
        recover_bundle(sidecar, stamp)
        self.assertEqual((wheels / "old.whl").read_bytes(), b"old")
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
        self.assertFalse(bundle_valid(wheels, stamp))
        (wheels / "note.txt").unlink()
        wheel = wheels / "one.whl"
        wheel.write_bytes(b"wheel")
        os.link(wheel, wheels / "two.whl")
        self.assertFalse(bundle_valid(wheels, stamp))


if __name__ == "__main__":
    unittest.main()
