def resolve_torch_device(requested):
    if requested == "cpu":
        return "cpu"
    try:
        import torch

        if torch.cuda.is_available():
            return "cuda"
        if hasattr(torch.backends, "mps") and torch.backends.mps.is_available():
            return "mps"
    except Exception:
        return "cpu"
    return "cpu"


def transformers_device_map(requested):
    return resolve_torch_device(requested)


def move_model(model, requested):
    if hasattr(model, "to"):
        return model.to(resolve_torch_device(requested))
    return model


def move_tensor(tensor, requested):
    if hasattr(tensor, "to"):
        return tensor.to(resolve_torch_device(requested))
    return tensor
