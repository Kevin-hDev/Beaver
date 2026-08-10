export type ToastType = "success" | "error" | "warning" | "info" | "check";
export interface ToastAction {
  label: string;
  onClick: () => void;
}

export interface ToastOptions {
  action?: ToastAction;
  dismissLabel?: string;
}

type ToastFn = (
  message: string,
  type?: ToastType,
  duration?: number,
  options?: ToastOptions,
) => void;

let _show: ToastFn = () => {};

export function registerToast(fn: ToastFn) {
  _show = fn;
}

export function showToast(
  message: string,
  type: ToastType = "error",
  duration?: number,
  options?: ToastOptions,
) {
  _show(message, type, duration, options);
}
