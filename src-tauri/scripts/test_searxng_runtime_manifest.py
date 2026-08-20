import json
import sys
import unittest
from unittest.mock import patch

from searxng_runtime_manifest import (
    MAX_MANIFEST_BYTES,
    RuntimeIdentity,
    build_manifest,
    current_manifest,
    parse_manifest,
    runtime_identity,
)


class RuntimeManifestTests(unittest.TestCase):
    def test_round_trip_records_exact_cpython_minor_as_canonical_ascii(self):
        raw = build_manifest("cpython", 3, 14, "a" * 64)

        self.assertEqual(
            raw,
            b'{"schema_version":1,"implementation":"cpython","major":3,"minor":14,"requirements_sha256":"'
            + b"a" * 64
            + b'"}',
        )
        self.assertEqual(
            parse_manifest(raw),
            {
                "schema_version": 1,
                "implementation": "cpython",
                "major": 3,
                "minor": 14,
                "requirements_sha256": "a" * 64,
            },
        )

    def test_current_manifest_uses_the_running_interpreter_identity(self):
        implementation = type("Implementation", (), {"name": "cpython"})()
        version = type("Version", (), {"major": 3, "minor": 14})()
        with (
            patch.object(sys, "implementation", implementation),
            patch.object(sys, "version_info", version),
        ):
            self.assertEqual(
                current_manifest("b" * 64),
                build_manifest("cpython", 3, 14, "b" * 64),
            )

    def test_rejects_unknown_duplicate_missing_and_nonobject_documents(self):
        valid = {
            "schema_version": 1,
            "implementation": "cpython",
            "major": 3,
            "minor": 14,
            "requirements_sha256": "a" * 64,
        }
        invalid = [
            b'{"schema_version":1,"schema_version":1,"implementation":"cpython","major":3,"minor":14,"requirements_sha256":"'
            + b"a" * 64
            + b'"}',
            json.dumps({**valid, "extra": "blocked"}).encode("ascii"),
            json.dumps(
                {key: value for key, value in valid.items() if key != "minor"}
            ).encode("ascii"),
            b"[]",
            b"null",
        ]

        for body in invalid:
            with self.subTest(body=body[:32]), self.assertRaises(ValueError):
                parse_manifest(body)

    def test_rejects_invalid_types_values_and_json_constants(self):
        valid = {
            "schema_version": 1,
            "implementation": "cpython",
            "major": 3,
            "minor": 14,
            "requirements_sha256": "a" * 64,
        }
        invalid = [
            {**valid, "schema_version": True},
            {**valid, "major": True},
            {**valid, "minor": True},
            {**valid, "minor": 9},
            {**valid, "minor": 100},
            {**valid, "implementation": "pypy"},
            {**valid, "requirements_sha256": "A" * 64},
            {**valid, "requirements_sha256": "g" * 64},
        ]

        for document in invalid:
            with self.subTest(document=document), self.assertRaises(ValueError):
                parse_manifest(json.dumps(document).encode("ascii"))
        for body in (b'{"schema_version":NaN}', b'{"schema_version":Infinity}'):
            with self.subTest(body=body), self.assertRaises(ValueError):
                parse_manifest(body)

    def test_rejects_invalid_encoding_syntax_and_oversized_documents(self):
        for body in (b"\xff", b"{", b"x" * (MAX_MANIFEST_BYTES + 1)):
            with self.subTest(body=body[:8]), self.assertRaises(ValueError):
                parse_manifest(body)

    def test_build_manifest_rejects_an_invalid_interpreter_or_hash(self):
        invalid = (
            ("cpython", 3, True, "a" * 64),
            ("pypy", 3, 14, "a" * 64),
            ("cpython", 3, 14, "A" * 64),
        )
        for values in invalid:
            with self.subTest(values=values), self.assertRaises(ValueError):
                build_manifest(*values)

    def test_runtime_identity_requires_the_stamp_to_match_the_manifest_hash(self):
        manifest = build_manifest("cpython", 3, 14, "a" * 64)
        identity = runtime_identity("a" * 64, manifest)

        self.assertIsInstance(identity, RuntimeIdentity)
        self.assertEqual(identity.stamp, "a" * 64)
        self.assertEqual(identity.manifest, manifest)
        with self.assertRaises(ValueError):
            runtime_identity("b" * 64, manifest)


if __name__ == "__main__":
    unittest.main()
