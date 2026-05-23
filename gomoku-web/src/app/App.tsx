import { Suspense, lazy } from "react";
import { Navigate, Route, Routes } from "react-router-dom";

import { CloudSessionController } from "../cloud/CloudSessionController";
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
const BotReportRoute = lazy(async () => ({
  default: (await import("../routes/BotReportRoute")).BotReportRoute,
}));
const AnalysisReportRoute = lazy(async () => ({
  default: (await import("../routes/AnalysisReportRoute")).AnalysisReportRoute,
}));

export function App() {
  return (
    <div className={styles.app}>
      <CloudSessionController />
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
        <Route
          path="/bot-report/*"
          element={
            <Suspense fallback={<main className={styles.loading}>Loading report…</main>}>
              <BotReportRoute />
            </Suspense>
          }
        />
        <Route
          path="/analysis-report/*"
          element={
            <Suspense fallback={<main className={styles.loading}>Loading report…</main>}>
              <AnalysisReportRoute />
            </Suspense>
          }
        />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </div>
  );
}
