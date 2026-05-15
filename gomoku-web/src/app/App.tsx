import { Suspense, lazy } from "react";
import { Navigate, Route, Routes } from "react-router-dom";

import { HomeRoute } from "../routes/HomeRoute";

import styles from "./App.module.css";

const LocalMatchRoute = lazy(async () => ({
  default: (await import("../routes/LocalMatchRoute")).LocalMatchRoute,
}));
const ProfileRoute = lazy(async () => ({
  default: (await import("../routes/ProfileRoute")).ProfileRoute,
}));
const ReplayRoute = lazy(async () => ({
  default: (await import("../routes/ReplayRoute")).ReplayRoute,
}));
const SettingsRoute = lazy(async () => ({
  default: (await import("../routes/SettingsRoute")).SettingsRoute,
}));

export function App() {
  return (
    <div className={styles.app}>
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
          path="/replay/:matchId"
          element={
            <Suspense fallback={<main className={styles.loading}>Loading replay…</main>}>
              <ReplayRoute />
            </Suspense>
          }
        />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </div>
  );
}
