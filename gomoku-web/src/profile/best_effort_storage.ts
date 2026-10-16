import type { StateStorage } from "zustand/middleware";

export interface BrowserStorage {
  getItem: (name: string) => string | null;
  setItem: (name: string, value: string) => void;
  removeItem: (name: string) => void;
}

// Once a read or write fails, keep this store in memory for the session. In
// particular, never replace unreadable persisted data with a fallback profile.
export function bestEffortStorage(backing: () => BrowserStorage, onUnavailable?: (error?: unknown) => void): BrowserStorage;
export function bestEffortStorage(backing: () => StateStorage, onUnavailable?: (error?: unknown) => void): StateStorage;
export function bestEffortStorage(
  backing: () => StateStorage,
  onUnavailable: (error?: unknown) => void = () => undefined,
): StateStorage {
  const memory = new Map<string, string>();
  let memoryOnly = false;
  const unavailable = (error?: unknown) => {
    if (memoryOnly) return;
    memoryOnly = true;
    onUnavailable(error);
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
            }).catch((error: unknown) => {
              unavailable(error);
              return memory.get(name) ?? null;
            });
          }
          if (value !== null) memory.set(name, value);
          else memory.delete(name);
          return value;
        } catch (error) {
          unavailable(error);
        }
      }
      return memory.get(name) ?? null;
    },
    setItem: (name, value) => {
      memory.set(name, value);
      if (!memoryOnly) {
        try {
          return Promise.resolve(backing().setItem(name, value)).catch(unavailable);
        } catch (error) {
          unavailable(error);
        }
      }
    },
    removeItem: (name) => {
      memory.delete(name);
      if (!memoryOnly) {
        try {
          return Promise.resolve(backing().removeItem(name)).catch(unavailable);
        } catch (error) {
          unavailable(error);
        }
      }
    },
  };
}
