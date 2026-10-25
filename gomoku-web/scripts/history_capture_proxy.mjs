// Operator-only legacy Firebase transport. Every upstream socket is loopback.
import http from "node:http";
import https from "node:https";
import net from "node:net";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const ownerRoot = resolve(process.argv[2] ?? "");
const marker = JSON.parse(
  readFileSync(resolve(ownerRoot, "owner.json"), "utf8"),
);
if (
  marker.owner !== "gomoku-history-implementation" ||
  marker.root !== ownerRoot ||
  marker.repo !== resolve(import.meta.dirname, "../..")
)
  throw new Error("Expected this goal's owned capture root");
const hosts = new Set([
  "firestore.googleapis.com",
  "identitytoolkit.googleapis.com",
  "securetoken.googleapis.com",
]);
const probeHost = "history-capture.invalid";
/** @type {Set<import('node:net').Socket>} */
const sockets = new Set();
/** @type {Array<{host:string,path:string,method:string,status?:number,error?:string}>} */
const events = [];
let denied = 0;
const tls = https.createServer(
  {
    key: readFileSync(resolve(ownerRoot, "emulators/proxy-key.pem")),
    cert: readFileSync(resolve(ownerRoot, "emulators/proxy-cert.pem")),
    ALPNProtocols: ["http/1.1"],
  },
  (request, response) => {
    const host = request.headers.host?.split(":")[0] ?? "";
    const url = new URL(request.url ?? "/", "https://" + (host || "invalid"));
    if (
      host === probeHost &&
      request.method === "GET" &&
      url.pathname === "/__history_transport_probe__"
    ) {
      response.writeHead(200, { "content-type": "application/json" });
      response.end(
        JSON.stringify({
          owner: marker.owner,
          ownerRoot,
          project: "demo-gomoku-history",
          route: "loopback-only",
        }),
      );
      return;
    }
    const allowedPath =
      host === "firestore.googleapis.com"
        ? url.pathname.startsWith("/google.firestore.v1.Firestore/") ||
          url.pathname.startsWith("/v1/projects/demo-gomoku-history/")
        : host === "identitytoolkit.googleapis.com"
          ? url.pathname.startsWith("/v1/accounts:")
          : url.pathname === "/v1/token";
    if (
      !hosts.has(host) ||
      !allowedPath ||
      (url.searchParams.has("database") &&
        url.searchParams.get("database") !==
          "projects/demo-gomoku-history/databases/(default)")
    ) {
      denied++;
      response.writeHead(403).end();
      return;
    }
    const port = host === "firestore.googleapis.com" ? 28081 : 29099;
    const path =
      host === "firestore.googleapis.com"
        ? request.url
        : "/" + host + request.url;
    const event = { host, path: url.pathname, method: request.method ?? "GET" };
    events.push(event);
    const upstream = http.request(
      {
        hostname: "127.0.0.1",
        port,
        path,
        method: request.method,
        headers: { ...request.headers, host: `127.0.0.1:${port}` },
      },
      (result) => {
        /** @type {typeof events[number]} */ (event).status = result.statusCode;
        response.writeHead(result.statusCode ?? 502, result.headers);
        result.pipe(response);
      },
    );
    upstream.on("error", (error) => {
      /** @type {typeof events[number]} */ (event).error = error.message;
      if (!response.headersSent) response.writeHead(502);
      response.end();
    });
    request.pipe(upstream);
    response.on("close", () => upstream.destroy());
  },
);
const proxy = http.createServer((request, response) => {
  if (
    request.method === "GET" &&
    request.url === "/__history_transport_health__"
  ) {
    response.writeHead(200, { "content-type": "application/json" });
    response.end(
      JSON.stringify({
        owner: marker.owner,
        ownerRoot,
        project: "demo-gomoku-history",
        authPort: 29099,
        firestorePort: 28081,
        tlsPort: 29443,
        proxyPort: 29444,
        denied,
        forwarding: "unchanged HTTP responses/streams from loopback emulators",
        events,
      }),
    );
  } else {
    denied++;
    response.writeHead(403).end();
  }
});
proxy.on("connect", (request, client, head) => {
  const authority = (request.url ?? "").split(":");
  const [host, port] = authority;
  if (
    authority.length !== 2 ||
    (!hosts.has(host) && host !== probeHost) ||
    port !== "443"
  ) {
    denied++;
    client.end("HTTP/1.1 403 Forbidden\r\n\r\n");
    return;
  }
  const upstream = net.connect(29443, "127.0.0.1", () => {
    client.write("HTTP/1.1 200 Connection Established\r\n\r\n");
    if (head.length) upstream.write(head);
    client.pipe(upstream);
    upstream.pipe(client);
  });
  upstream.on("error", () => client.destroy());
  client.on("error", () => upstream.destroy());
  client.on("close", () => upstream.destroy());
});
for (const server of [tls, proxy]) {
  server.on("connection", (socket) => {
    sockets.add(socket);
    socket.on("close", () => sockets.delete(socket));
  });
  server.on("error", (error) => {
    console.error(error.message);
    for (const socket of sockets) socket.destroy();
    process.exitCode = 1;
    tls.close();
    proxy.close();
  });
}
tls.listen(29443, "127.0.0.1");
proxy.listen(29444, "127.0.0.1", () =>
  console.log("Owned loopback Firebase history transport ready"),
);
for (const signal of ["SIGINT", "SIGTERM"])
  process.on(signal, () => {
    for (const socket of sockets) socket.destroy();
    tls.close();
    proxy.close();
  });
