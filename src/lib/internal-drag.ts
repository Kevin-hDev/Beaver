let _active = false;

export function setInternalDrag(active: boolean) {
  _active = active;
}

export function isInternalDrag(): boolean {
  return _active;
}
