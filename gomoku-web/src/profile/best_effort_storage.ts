import type { StateStorage } from "zustand/middleware";

export interface BrowserStorage {
  getItem: (name: string) => string | null;
  setItem: (name: string, value: string) => void;
  removeItem: (name: string) => void;
}

// Once a read or write fails, keep this store in memory for the session. In
// particular, never replace unreadable persisted data with a fallback profile.
export function bestEffortStorage(backing: () => BrowserStorage, onUnavailable?: () => void): BrowserStorage;
export function bestEffortStorage(backing: () => StateStorage, onUnavailable?: () => void): StateStorage;
export function bestEffortStorage(
  backing: () => StateStorage,
  onUnavailable: () => void = () => undefined,
): StateStorage {
  const memory = new Map<string, string>();
  let memoryOnly = false;
  const unavailable = () => {
    if (memoryOnly) return;
    memoryOnly = true;
    onUnavailable();
  };

  return {
    getItem: (name) => {
      if (!memoryOnly) {
        try {
          const value = backing().getItem(name);
          if (value !== null && typeof value !== "string") {
            return Promise.resolve(value).then((resolved) => {
              if (resolved === null) memory.delete(name);
              else memory.set(name, resolved);
              return resolved;
            }).catch(() => {
              unavailable();
              return memory.get(name) ?? null;
            });
          }
          if (value !== null) memory.set(name, value);
          else memory.delete(name);
          return value;
        } catch {
          unavailable();
        }
      }
      return memory.get(name) ?? null;
    },
    setItem: (name, value) => {
      memory.set(name, value);
      if (!memoryOnly) {
        try {
          return Promise.resolve(backing().setItem(name, value)).catch(unavailable);
        } catch {
          unavailable();
        }
      }
    },
    removeItem: (name) => {
      memory.delete(name);
      if (!memoryOnly) {
        try {
          return Promise.resolve(backing().removeItem(name)).catch(unavailable);
        } catch {
          unavailable();
        }
      }
    },
  };
}
