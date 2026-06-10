export type ToastKind = "info" | "warning" | "error";

export interface ToastItem {
  id: string;
  message: string;
  kind: ToastKind;
}

let queue: ToastItem[] = [];
let version = 0;
const listeners = new Set<() => void>();

function notify(): void {
  version += 1;
  for (const listener of listeners) listener();
}

export function subscribeToasts(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function getToasts(): ToastItem[] {
  return queue;
}

export function getToastVersion(): number {
  return version;
}

export function pushToast(
  message: string,
  kind: ToastKind = "info",
  durationMs = 5200,
): void {
  const id = crypto.randomUUID();
  queue = [...queue, { id, message, kind }];
  notify();
  window.setTimeout(() => dismissToast(id), durationMs);
}

export function dismissToast(id: string): void {
  const next = queue.filter((item) => item.id !== id);
  if (next.length === queue.length) return;
  queue = next;
  notify();
}
