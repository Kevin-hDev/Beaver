from pathlib import Path

ADAPTERS = {
    "chronos-bolt": (".chronos_adapter", "ChronosAdapter"),
    "chronos-2": (".chronos_adapter", "ChronosAdapter"),
    "timesfm-2-5": (".timesfm_adapter", "TimesFmAdapter"),
    "toto-2": (".toto_adapter", "TotoAdapter"),
    "moirai-2": (".moirai_adapter", "MoiraiAdapter"),
    "flowstate": (".flowstate_adapter", "FlowStateAdapter"),
    "tabpfn-ts": (".tabpfn_adapter", "TabPfnTsAdapter"),
    "tirex": (".tirex_adapter", "TiRexAdapter"),
    "kairos": (".kairos_adapter", "KairosAdapter"),
    "sundial": (".sundial_adapter", "SundialAdapter"),
}


def load_adapter(family_id, model_name, models_dir, device="gpu"):
    adapter_target = ADAPTERS.get(family_id)
    if adapter_target is None:
        raise SystemExit("missing_adapter")
    module_name, class_name = adapter_target
    from importlib import import_module

    adapter_cls = getattr(import_module(module_name, __package__), class_name)

    model_dir = Path(models_dir).joinpath(model_name)
    if not model_dir.exists():
        raise SystemExit("missing_model")
    return adapter_cls(family_id, model_name, model_dir, device)
