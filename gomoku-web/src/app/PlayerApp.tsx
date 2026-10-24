import { Component, Suspense, lazy, type ReactNode } from "react";
import { matchRoutes, Navigate, Route, Routes, useLocation } from "react-router-dom";

import { CloudAuthController, CloudSessionController } from "../cloud/CloudSessionController";
import { HomeRoute } from "../routes/HomeRoute";
import { LocalPersistenceNotice } from "../profile/LocalPersistenceNotice";

import styles from "./App.module.css";

const standaloneRoutes = [{ path: "/replay/shared" }, { path: "/player" }];

const LocalMatchRoute = lazy(async () => ({
  default: (await import("../routes/LocalMatchRoute")).LocalMatchRoute,
}));
const ProfileRoute = lazy(async () => ({
  default: (await import("../routes/ProfileRoute")).ProfileRoute,
}));
const ReplayRoute = lazy(async () => ({
  default: (await import("../routes/ReplayRoute")).ReplayRoute,
}));
const SharedReplayRoute = lazy(async () => ({
  default: (await import("../routes/SharedReplayRoute")).SharedReplayRoute,
}));
const PublicPlayerRoute = lazy(async () => ({
  default: (await import("../routes/PublicPlayerRoute")).PublicPlayerRoute,
}));
const SettingsRoute = lazy(async () => ({
  default: (await import("../routes/SettingsRoute")).SettingsRoute,
}));


const RulesRoute = lazy(async () => ({
  default: (await import("../routes/RulesRoute")).RulesRoute,
}));
const GuideRoute = lazy(async () => ({
  default: (await import("../routes/GuideRoute")).GuideRoute,
}));
const PolicyRoute = lazy(async () => ({
  default: (await import("../routes/PolicyRoute")).PolicyRoute,
}));

class PageRecovery extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false };

  static getDerivedStateFromError() {
    return { failed: true };
  }

  render() {
    if (!this.state.failed) return this.props.children;
    return (
      <main className={styles.recovery}>
        <h1>Page unavailable</h1>
        <p>Reload to try again, or return Home.</p>
        <div>
          <button className="uiAction uiActionPrimary" onClick={() => window.location.reload()} type="button">Reload</button>
          <a className="uiAction uiActionNeutral" href={import.meta.env.BASE_URL}>Home</a>
        </div>
      </main>
    );
  }
}

export function PlayerApp() {
  const { pathname } = useLocation();
  const standalone = matchRoutes(standaloneRoutes, { pathname }) !== null;
  return (
    <div className={styles.app}>
      <CloudAuthController />
      {!standalone && <CloudSessionController />}
      <LocalPersistenceNotice />
      <div className={styles.content}>
        <PageRecovery>
          <Routes>
            <Route path="/" element={<HomeRoute />} />
            <Route
              path="/match/local"
              element={
                <Suspense fallback={<main className={styles.loading}>Loading match…</main>}>
                  <LocalMatchRoute />
                </Suspense>
              }
            />
            <Route
              path="/settings"
              element={
                <Suspense fallback={<main className={styles.loading}>Loading settings…</main>}>
                  <SettingsRoute />
                </Suspense>
              }
            />
            <Route
              path="/profile"
              element={
                <Suspense fallback={<main className={styles.loading}>Loading profile…</main>}>
                  <ProfileRoute />
                </Suspense>
              }
            />
            <Route
              path="/player"
              element={
                <Suspense fallback={<main className={styles.loading}>Loading public profile…</main>}>
                  <PublicPlayerRoute />
                </Suspense>
              }
            />
            <Route
              path="/replay/shared"
              element={
                <Suspense fallback={<main className={styles.loading}>Loading shared replay…</main>}>
                  <SharedReplayRoute />
                </Suspense>
              }
            />
            <Route
              path="/replay/:matchId"
              element={
                <Suspense fallback={<main className={styles.loading}>Loading replay…</main>}>
                  <ReplayRoute />
                </Suspense>
              }
            />


            <Route
              path="/rules/*"
              element={
                <Suspense fallback={<main className={styles.loading}>Loading page…</main>}>
                  <RulesRoute />
                </Suspense>
              }
            />
            <Route
              path="/guide/*"
              element={
                <Suspense fallback={<main className={styles.loading}>Loading page…</main>}>
                  <GuideRoute />
                </Suspense>
              }
            />
            <Route
              path="/privacy/*"
              element={
                <Suspense fallback={<main className={styles.loading}>Loading page…</main>}>
                  <PolicyRoute kind="privacy" />
                </Suspense>
              }
            />
            <Route
              path="/terms/*"
              element={
                <Suspense fallback={<main className={styles.loading}>Loading page…</main>}>
                  <PolicyRoute kind="terms" />
                </Suspense>
              }
            />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </PageRecovery>
      </div>
    </div>
  );
}
