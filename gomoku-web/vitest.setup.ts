import "@testing-library/jest-dom/vitest";

const backing = new Map<string, string>();

Object.defineProperty(globalThis, "localStorage", {
  configurable: true,
  value: {
    clear: () => backing.clear(),
    getItem: (key: string) => backing.get(key) ?? null,
    key: (index: number) => Array.from(backing.keys())[index] ?? null,
    removeItem: (key: string) => backing.delete(key),
    setItem: (key: string, value: string) => {
      backing.set(key, value);
    },
    get length() {
      return backing.size;
    },
  },
});

// jsdom has no Web Locks. Keep unit writes serialized as they are in the
// browser. Tests of missing locks can inject a null lock manager.
if (!("locks" in navigator)) {
  const lockTails = new Map<string, Promise<void>>();
  Object.defineProperty(navigator, "locks", {
    configurable: true,
    value: {
      request<T>(name: string, _options: { mode: "exclusive" }, work: () => Promise<T>): Promise<T> {
        const result = (lockTails.get(name) ?? Promise.resolve()).then(work);
        const tail = result.then(() => undefined, () => undefined);
        lockTails.set(name, tail);
        void tail.then(() => { if (lockTails.get(name) === tail) lockTails.delete(name); });
        return result;
      },
    },
  });
}
