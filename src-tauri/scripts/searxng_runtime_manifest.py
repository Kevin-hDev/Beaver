import json
import re
import secrets
import sys
from dataclasses import dataclass

# Ce point d'entrée est l'unique générateur de la fixture partagée avec Rust ;
# le fichier golden ne doit jamais être édité à la main.

MANIFEST_NAME = ".runtime.json"
MAX_MANIFEST_BYTES = 512
EXPECTED_FIELDS = {
    "schema_version",
    "implementation",
    "major",
    "minor",
    "requirements_sha256",
}


@dataclass(frozen=True)
class RuntimeIdentity:
    stamp: str
    manifest: bytes


def strict_pairs(pairs: list[tuple[str, object]]) -> dict[str, object]:
    document: dict[str, object] = {}
    for key, value in pairs:
        if key in document:
            raise ValueError("invalid runtime manifest")
        document[key] = value
    return document


def _reject_constant(_value: str) -> None:
    raise ValueError("invalid runtime manifest")


def validate_manifest(document: object) -> None:
    if type(document) is not dict:
        raise ValueError("invalid runtime manifest")
    valid = (
        set(document) == EXPECTED_FIELDS
        and type(document["schema_version"]) is int
        and document["schema_version"] == 1
        and document["implementation"] == "cpython"
        and type(document["major"]) is int
        and document["major"] == 3
        and type(document["minor"]) is int
        and 10 <= document["minor"] <= 99
        and type(document["requirements_sha256"]) is str
        and re.fullmatch(r"[0-9a-f]{64}", document["requirements_sha256"]) is not None
    )
    if not valid:
        raise ValueError("invalid runtime manifest")


def build_manifest(
    implementation: str,
    major: int,
    minor: int,
    requirements_sha256: str,
) -> bytes:
    document = {
        "schema_version": 1,
        "implementation": implementation,
        "major": major,
        "minor": minor,
        "requirements_sha256": requirements_sha256,
    }
    validate_manifest(document)
    try:
        body = json.dumps(
            document,
            separators=(",", ":"),
            ensure_ascii=True,
        ).encode("ascii")
    except (TypeError, UnicodeError) as error:
        raise ValueError("invalid runtime manifest") from error
    parse_manifest(body)
    return body


def parse_manifest(body: object) -> dict[str, object]:
    if type(body) is not bytes or len(body) > MAX_MANIFEST_BYTES:
        raise ValueError("invalid runtime manifest")
    try:
        document = json.loads(
            body.decode("utf-8"),
            object_pairs_hook=strict_pairs,
            parse_constant=_reject_constant,
        )
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        raise ValueError("invalid runtime manifest") from error
    validate_manifest(document)
    return document


def current_manifest(requirements_sha256: str) -> bytes:
    return build_manifest(
        sys.implementation.name,
        sys.version_info.major,
        sys.version_info.minor,
        requirements_sha256,
    )


def runtime_identity(stamp: object, manifest: object) -> RuntimeIdentity:
    if type(stamp) is not str or re.fullmatch(r"[0-9a-f]{64}", stamp) is None:
        raise ValueError("invalid runtime manifest")
    document = parse_manifest(manifest)
    canonical = build_manifest(
        document["implementation"],
        document["major"],
        document["minor"],
        document["requirements_sha256"],
    )
    if not secrets.compare_digest(manifest, canonical):
        raise ValueError("invalid runtime manifest")
    if not secrets.compare_digest(document["requirements_sha256"], stamp):
        raise ValueError("invalid runtime manifest")
    return RuntimeIdentity(stamp, canonical)


def _emit_contract(arguments: list[str]) -> int:
    try:
        if len(arguments) != 2:
            raise ValueError("invalid runtime manifest")
        sys.stdout.buffer.write(current_manifest(arguments[1]))
        return 0
    except (OSError, ValueError):
        sys.stderr.write("invalid runtime manifest\n")
        return 1


if __name__ == "__main__":
    raise SystemExit(_emit_contract(sys.argv))
