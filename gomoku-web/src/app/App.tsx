import { Suspense, lazy } from "react";
import { matchRoutes, useLocation } from "react-router-dom";

const PlayerApp = lazy(async () => ({ default: (await import("./PlayerApp")).PlayerApp }));
const DeveloperApp = lazy(async () => ({ default: (await import("../developer/DeveloperApp")).DeveloperApp }));
const publicRoutes = [{ path: "/dev/*" }, { path: "/lab/*" }, { path: "/visuals/*" }];

// Keep this entry free of game/cloud/profile imports. Unmounting the player UI
// unsubscribes controllers; their singleton stores and in-flight promises survive.
// Returning remounts the controllers, which re-evaluate pending synchronization.
export function App() {
  const { pathname } = useLocation();
  const developer = matchRoutes(publicRoutes, { pathname }) !== null;
  return <Suspense fallback={<main aria-busy="true">Loading…</main>}>
    {developer ? <DeveloperApp /> : <PlayerApp />}
  </Suspense>;
}
