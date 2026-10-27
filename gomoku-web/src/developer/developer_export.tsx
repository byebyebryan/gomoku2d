import { createRoot } from "react-dom/client";
import { HashRouter, Navigate, Route, Routes } from "react-router-dom";
import "../app/global.css";
import { DeveloperApp } from "./DeveloperApp";

const root = document.getElementById("root");
if (!root) throw new Error("Developer preview root is missing");

createRoot(root).render(
  <HashRouter>
    <Routes>
      <Route path="/" element={<Navigate to="/dev/reviews/" replace />} />
      <Route path="*" element={<DeveloperApp standalone />} />
    </Routes>
  </HashRouter>,
);
