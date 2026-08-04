import hashlib
import io
import os
import shutil
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
)


class ZeroStream:
    def __init__(self, size):
        self.remaining = size

    def read(self, size):
        chunk = min(size, self.remaining)
        self.remaining -= chunk
        return b"\0" * chunk


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
        with self.assertRaises(PreparationError):
            safe_extract(self.archive(many), self.temp / "many")

    def test_refuses_a_symlink_parent_in_destination(self):
        linked = self.temp / "linked"
        linked.mkdir()
        try:
            os.symlink(linked, self.destination / "source", target_is_directory=True)
        except OSError as error:
            if os.name == "nt" and error.winerror in {5, 1314}:
                self.skipTest("symbolic links are unavailable")
            raise
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


if __name__ == "__main__":
    unittest.main()
