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
