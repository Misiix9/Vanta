import { writable } from "svelte/store";
import type { ToastOptions } from "$lib/sdk/types";

export type ToastKind = "success" | "error" | "info";

export interface ToastEntry {
  id: number;
  title: string;
  message?: string;
  type: ToastKind;
  createdAt: number;
  durationMs: number;
}

const MAX_TOAST_HISTORY = 50;
const DEFAULT_DURATION_MS = 4200;

const toastsWritable = writable<ToastEntry[]>([]);
const historyWritable = writable<ToastEntry[]>([]);

let nextToastId = 1;
const dismissTimers = new Map<number, ReturnType<typeof setTimeout>>();

function normalizeType(type?: ToastOptions["type"]): ToastKind {
  if (type === "success" || type === "error") return type;
  return "info";
}

function scheduleDismiss(id: number, durationMs: number) {
  const timer = setTimeout(() => {
    dismissToast(id);
  }, durationMs);
  dismissTimers.set(id, timer);
}

export function addToast(options: ToastOptions, durationMs = DEFAULT_DURATION_MS): number {
  const entry: ToastEntry = {
    id: nextToastId++,
    title: options.title,
    message: options.message,
    type: normalizeType(options.type),
    createdAt: Date.now(),
    durationMs,
  };

  toastsWritable.update((current) => [...current, entry]);
  historyWritable.update((current) => {
    const next = [entry, ...current];
    return next.slice(0, MAX_TOAST_HISTORY);
  });

  scheduleDismiss(entry.id, durationMs);
  return entry.id;
}

export function dismissToast(id: number) {
  const timer = dismissTimers.get(id);
  if (timer) {
    clearTimeout(timer);
    dismissTimers.delete(id);
  }
  toastsWritable.update((current) => current.filter((item) => item.id !== id));
}

export function clearToasts() {
  for (const timer of dismissTimers.values()) clearTimeout(timer);
  dismissTimers.clear();
  toastsWritable.set([]);
}

export const toasts = {
  subscribe: toastsWritable.subscribe,
};

export const toastHistory = {
  subscribe: historyWritable.subscribe,
};
