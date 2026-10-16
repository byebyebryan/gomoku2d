import { useStore } from "zustand";

import { localPersistenceStore } from "./local_profile_store";
import styles from "./LocalPersistenceNotice.module.css";

export function LocalPersistenceNotice() {
  const available = useStore(localPersistenceStore, (state) => state.available);
  const reason = useStore(localPersistenceStore, (state) => state.reason);
  if (available) return null;

  return (
    <p className={styles.notice} role="status">
      {reason === "changed"
        ? "Saved data changed in another tab. New history and settings in this tab will only stay for this session unless synced to your account."
        : "Local saves are unavailable. You can keep playing, but new history and settings will only stay for this session unless synced to your account."}
    </p>
  );
}
