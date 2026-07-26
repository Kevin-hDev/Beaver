export type ToastType = "success" | "error" | "warning" | "info" | "check";
type ToastFn = (message: string, type?: ToastType, duration?: number) => void;

let _show: ToastFn = () => {};

export function registerToast(fn: ToastFn) {
  _show = fn;
}

export function showToast(message: string, type: ToastType = "error", duration?: number) {
  _show(message, type, duration);
}
