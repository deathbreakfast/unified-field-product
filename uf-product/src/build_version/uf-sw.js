// Unified Field pkg cache.
//
// Serves a release's stamped build files (`/pkg-<stamp>/...`) cache first,
// one cache per build (`uf-pkg-<build>`), and downloads a new build ahead of
// a reload when the page asks. It only handles same-origin GETs under
// `/pkg-`: HTML, server functions, `/api`, auth and everything else go
// straight to the network. It keeps at most two builds.
"use strict";

const CACHE_PREFIX = "uf-pkg-";
const BUILD_RE = /^pkg(-[A-Za-z0-9-]+)?$/;

self.addEventListener("install", () => {
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(prune(null).then(() => self.clients.claim()));
});

function stampedBuildOf(pathname) {
  const segment = pathname.split("/")[1] || "";
  return segment.startsWith("pkg-") && BUILD_RE.test(segment) ? segment : null;
}

// Build files only: a host fallback can answer an unknown `/pkg-…` path with
// a 200 HTML page (possibly showing the signed-in user), which must never be
// cached.
function cacheable(response) {
  const contentType = response.headers.get("content-type") || "";
  return (
    response.ok &&
    response.status === 200 &&
    response.type === "basic" &&
    !contentType.toLowerCase().startsWith("text/html")
  );
}

self.addEventListener("fetch", (event) => {
  const request = event.request;
  if (request.method !== "GET" || request.headers.has("range")) return;
  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;
  const build = stampedBuildOf(url.pathname);
  if (!build) return;
  event.respondWith(cacheFirst(request, build));
});

async function cacheFirst(request, build) {
  const cache = await caches.open(CACHE_PREFIX + build);
  const hit = await cache.match(request);
  if (hit) return hit;
  const response = await fetch(request);
  if (cacheable(response)) {
    await cache.put(request, response.clone()).catch(() => {});
  }
  return response;
}

self.addEventListener("message", (event) => {
  const data = event.data;
  if (!data || data.type !== "prefetch") return;
  const port = event.ports && event.ports[0];
  const reply = (type) => {
    if (port) port.postMessage({ type });
  };
  event.waitUntil(
    prefetch(data).then(
      () => reply("prefetch-done"),
      () => reply("prefetch-failed"),
    ),
  );
});

async function prefetch({ build, running, assets }) {
  if (typeof build !== "string" || !BUILD_RE.test(build) || !Array.isArray(assets)) {
    throw new Error("invalid prefetch request");
  }
  // Only files inside the build's own dir, on this origin.
  const prefix = "/" + build + "/";
  const urls = assets.map((asset) => {
    if (typeof asset !== "string") throw new Error("invalid asset");
    const url = new URL(asset, self.location.origin);
    if (url.origin !== self.location.origin || !url.pathname.startsWith(prefix) || url.search) {
      throw new Error("invalid asset");
    }
    return url.href;
  });
  const cache = await caches.open(CACHE_PREFIX + build);
  for (const url of urls) {
    if (await cache.match(url)) continue;
    const response = await fetch(url, { credentials: "same-origin", cache: "no-cache" });
    if (!cacheable(response)) throw new Error("asset download failed");
    await cache.put(url, response);
  }
  const keep = [build];
  if (typeof running === "string" && BUILD_RE.test(running)) keep.push(running);
  await prune(keep);
}

// Delete every pkg cache except `keep`, or, with no list, all but the two
// newest (cache keys come back in creation order).
async function prune(keep) {
  const names = (await caches.keys()).filter((name) => name.startsWith(CACHE_PREFIX));
  const keepNames = keep
    ? new Set(keep.map((build) => CACHE_PREFIX + build))
    : new Set(names.slice(-2));
  await Promise.all(
    names.filter((name) => !keepNames.has(name)).map((name) => caches.delete(name)),
  );
}
