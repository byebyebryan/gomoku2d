import { useStore } from "zustand";

import { localPersistenceStore } from "./local_profile_store";
import styles from "./LocalPersistenceNotice.module.css";

export function LocalPersistenceNotice() {
  const available = useStore(localPersistenceStore, (state) => state.available);
  if (available) return null;

  return (
    <p className={styles.notice} role="status">
      Local saves are unavailable. You can keep playing, but new history and
      settings will only stay for this session unless synced to your account.
    </p>
  );
}
