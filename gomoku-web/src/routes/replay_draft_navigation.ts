let guard: { index: number; mayLeave: (delta: number) => boolean } | undefined;

// Register before BrowserRouter subscribes. Some browsers deliver the router's
// listener before a later window capture listener can stop the event.
export function installReplayDraftNavigation() {
  window.addEventListener("popstate", (event) => {
    if (!guard) return;
    const { index, mayLeave } = guard;
    const target: unknown = event.state?.idx;
    if (typeof target !== "number" || target === index || mayLeave(target - index)) return;
    event.stopImmediatePropagation();
    window.history.go(index - target);
  }, true);
}

export function protectReplayDraftNavigation(mayLeave: (delta: number) => boolean) {
  // BrowserRouter stores idx on each same-document entry. Cross-document
  // exits are handled by the editor's beforeunload listener.
  const index: unknown = window.history.state?.idx;
  if (typeof index !== "number") return () => {};
  const active = { index, mayLeave };
  guard = active;
  return () => { if (guard === active) guard = undefined; };
}
