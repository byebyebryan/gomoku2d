export function profileRecordScroller(target: HTMLElement | null): HTMLElement | null {
  let ancestor = target?.parentElement ?? null;
  while (ancestor && ancestor !== document.body) {
    const overflowY = getComputedStyle(ancestor).overflowY;
    if ((overflowY === "auto" || overflowY === "scroll") && ancestor.scrollHeight > ancestor.clientHeight + 1) {
      return ancestor;
    }
    ancestor = ancestor.parentElement;
  }
  return document.scrollingElement instanceof HTMLElement ? document.scrollingElement : null;
}
