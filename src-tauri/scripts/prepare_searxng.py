import os
import shutil
import subprocess
import sys
from pathlib import Path, PurePosixPath

from searxng_archive import copy_member, is_metadata, safe_extract
from searxng_bundle import BundleLock, bundle_valid, publish_bundle, recover_bundle, requirements_hash, temporary_directory, validate_source
from searxng_safety import ERROR_MESSAGE, PreparationError, fail, safe_directory

MAX_ARCHIVE_ENTRIES = 4096
MAX_MEMBER_BYTES = 64 * 1024 * 1024
MAX_WHEEL_BYTES = 150 * 1024 * 1024
MAX_WHEEL_FILES = 512
STAMP_NAME = ".requirements.sha256"


def _cli_root(value: object) -> Path:
    if not isinstance(value, str) or not value or len(value) > 4096 or any(char in value for char in ("\0", "\r", "\n")):
        fail()
    if ".." in value.replace("\\", "/").split("/"):
        fail()
    root = Path(value)
    if not root.is_absolute():
        fail()
    return root


def _run_download(run_process, requirements: Path, temporary: Path) -> None:
    for arguments in (["-r", str(requirements)], ["setuptools", "wheel"]):
        run_process(
            [sys.executable, "-m", "pip", "download", "--only-binary=:all:", "--dest", str(temporary), *arguments],
            check=True,
            shell=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )


def prepare(root: Path, run_process=subprocess.run) -> None:
    temporary_source = temporary_wheels = None
    try:
        sidecar = safe_directory(safe_directory(root) / "resources" / "searxng-sidecar")
        with BundleLock(sidecar):
            source = sidecar / "source"
            if not source.exists():
                temporary_source = temporary_directory(sidecar, "source-")
                safe_extract(sidecar / "source.tar.gz", temporary_source)
                source = temporary_source / "source"
            requirements, setup = validate_source(source)
            stamp = requirements_hash(requirements, setup)
            recover_bundle(sidecar, stamp)
            if bundle_valid(sidecar / "wheels", stamp):
                return
            temporary_wheels = temporary_directory(sidecar, "wheels-new-")
            _run_download(run_process, requirements, temporary_wheels)
            if not bundle_valid(temporary_wheels, stamp, False):
                fail()
            (temporary_wheels / STAMP_NAME).write_text(stamp, encoding="ascii")
            if not bundle_valid(temporary_wheels, stamp):
                fail()
            publish_bundle(sidecar, temporary_wheels, stamp)
            temporary_wheels = None
    except PreparationError:
        raise
    except Exception:
        fail()
    finally:
        for directory in (temporary_source, temporary_wheels):
            if directory is not None:
                shutil.rmtree(directory, ignore_errors=True)


def main(arguments=None) -> int:
    values = sys.argv[1:] if arguments is None else arguments
    try:
        if not isinstance(values, list) or len(values) != 2 or values[0] != "--root":
            fail()
        prepare(_cli_root(values[1]))
        return 0
    except Exception:
        print(ERROR_MESSAGE, file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
