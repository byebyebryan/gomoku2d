// A separate port range lets acceptance own its processes without disturbing
// a manual preview. The default preserves the existing CI environment.
const rawOffset = process.env.GOMOKU_CLOUD_PORT_OFFSET?.trim() || "0";
if (!/^\d+$/.test(rawOffset) || !Number.isSafeInteger(Number(rawOffset))) {
  throw new Error("GOMOKU_CLOUD_PORT_OFFSET must be a nonnegative integer.");
}
export const cloudPortOffset = Number(rawOffset);
export const cloudTestPorts = Object.freeze({
  preview: 8013 + cloudPortOffset,
  auth: 19099 + cloudPortOffset,
  firestore: 18081 + cloudPortOffset,
  websocket: 19151 + cloudPortOffset,
  hub: 14401 + cloudPortOffset,
  logging: 14501 + cloudPortOffset,
});
if (Object.values(cloudTestPorts).some((port) => port < 1024 || port > 65535)) {
  throw new Error("GOMOKU_CLOUD_PORT_OFFSET puts an acceptance port outside 1024–65535.");
}
export const cloudTestOrigins = Object.freeze({
  preview: `http://127.0.0.1:${cloudTestPorts.preview}`,
  auth: `http://127.0.0.1:${cloudTestPorts.auth}`,
  firestore: `http://127.0.0.1:${cloudTestPorts.firestore}`,
});
