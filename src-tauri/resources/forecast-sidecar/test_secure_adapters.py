import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import patch

from forecast_runtime.kairos_adapter import KairosAdapter
from forecast_runtime.sundial_adapter import SundialAdapter
from forecast_runtime.tirex_adapter import TiRexAdapter


class SecureAdapterLoadingTests(unittest.TestCase):
    def test_kairos_loads_only_the_installed_local_implementation(self):
        calls = []

        class FakeAutoModel:
            @classmethod
            def from_pretrained(cls, path, **kwargs):
                calls.append((path, kwargs))
                return object()

        modules = {
            "tsfm": types.ModuleType("tsfm"),
            "tsfm.model": types.ModuleType("tsfm.model"),
            "tsfm.model.kairos": types.SimpleNamespace(AutoModel=FakeAutoModel),
        }
        with patch.dict(sys.modules, modules):
            adapter = KairosAdapter("kairos", "kairos-10m", "/local/model", "cpu")
            adapter._load_model()

        self.assertEqual(calls[0][0], "/local/model")
        self.assertEqual(calls[0][1], {"local_files_only": True})

    def test_tirex_loads_the_exact_local_checkpoint(self):
        calls = []

        class FakeTiRex:
            @classmethod
            def from_pretrained(cls, path, **kwargs):
                calls.append((path, kwargs))
                return object()

        with tempfile.TemporaryDirectory() as directory:
            checkpoint = Path(directory).joinpath("model.ckpt")
            checkpoint.write_bytes(b"checkpoint")
            module = types.SimpleNamespace(TiRexZero=FakeTiRex)
            with patch.dict(sys.modules, {"tirex": module}):
                adapter = TiRexAdapter("tirex", "tirex-35m", Path(directory), "cpu")
                adapter._load_model()

        self.assertEqual(calls[0][0], str(checkpoint))
        self.assertEqual(calls[0][1], {"backend": "torch", "device": "cpu"})

    def test_sundial_remote_loader_is_pinned_to_local_files(self):
        calls = []

        class FakeAutoModel:
            @classmethod
            def from_pretrained(cls, path, **kwargs):
                calls.append((path, kwargs))
                return object()

        module = types.SimpleNamespace(AutoModelForCausalLM=FakeAutoModel)
        with patch.dict(sys.modules, {"transformers": module}):
            adapter = SundialAdapter("sundial", "sundial-128m", "/local/model", "cpu")
            adapter._load_model({"model_config": {}})

        self.assertEqual(calls[0][0], "/local/model")
        self.assertEqual(
            calls[0][1],
            {"trust_remote_code": True, "local_files_only": True},
        )


if __name__ == "__main__":
    unittest.main()
