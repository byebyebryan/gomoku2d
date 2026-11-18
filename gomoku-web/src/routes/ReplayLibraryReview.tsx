import { useEffect, useRef, useState } from "react";

import type { PrivateLibraryEntry } from "../cloud/sharing_private";
import { decodePortableReplay } from "../replay/portable_replay_codec";
import { Icon } from "../ui/Icon";
import styles from "./ReplayLibraryPanel.module.css";

export type ReplayReviewChoice = "local" | "incoming";

export interface ReplayReviewRow {
  fingerprint: string;
  title: string;
  local: PrivateLibraryEntry | null;
  incoming: PrivateLibraryEntry | null;
  conflict: boolean;
}

export function replayReviewTitle(entry: PrivateLibraryEntry): string {
  try {
    const { replay } = decodePortableReplay(entry.payload);
    return `${replay.black.name} · ${replay.white.name} · ${replay.moves.length} moves`;
  } catch {
    return "Replay unavailable";
  }
}

function Version({ entry, source, choice, selected, disabled, onChoose, title }: {
  entry: PrivateLibraryEntry;
  source: string;
  choice: ReplayReviewChoice;
  selected: boolean;
  disabled: boolean;
  onChoose: (choice: ReplayReviewChoice) => void;
  title: string;
}) {
  const { metadata } = entry;
  return <div className={`${styles.reviewVersion} ${selected ? styles.reviewSelected : ""}`}>
    <p className="uiSectionLabel">{source}</p>
    <strong>{metadata.title || "Untitled"}</strong>
    <p className={styles.reviewNote}>{metadata.note || "No note"}</p>
    <small>{metadata.opening_ply === null ? "Final move" : `Opens at move ${metadata.opening_ply}`}</small>
    <button className={`uiAction ${selected ? "uiActionAccent" : "uiActionNeutral"}`} type="button"
      aria-pressed={selected} aria-label={`Keep ${source.toLowerCase()} details for ${title}`}
      disabled={disabled} onClick={() => onChoose(choice)}>Keep {source.toLowerCase()} details</button>
  </div>;
}

export function ReplayLibraryReview({ kind, rows, busy, onCancel, onApply }: {
  kind: "merge" | "import";
  rows: readonly ReplayReviewRow[];
  busy: boolean;
  onCancel: () => void;
  onApply: (choices: Record<string, ReplayReviewChoice>) => void;
}) {
  const [choices, setChoices] = useState<Record<string, ReplayReviewChoice>>({});
  const heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => { heading.current?.focus(); }, []);
  const isImport = kind === "import";
  const localLabel = isImport ? "Account" : "Device";
  const incomingLabel = isImport ? "Guest" : "Cloud";
  const localOnly = rows.filter((row) => row.local && !row.incoming);
  const incomingOnly = rows.filter((row) => row.incoming && !row.local);
  const conflicts = rows.filter((row) => row.conflict);
  const disabled = busy || conflicts.some((row) => choices[row.fingerprint] === undefined);

  return <section className={styles.conflict} aria-label={isImport ? "Review guest copies" : "Review library changes"}>
    <h3 ref={heading} tabIndex={-1}>{isImport ? "Review guest copies" : "Review library changes"}</h3>
    {conflicts.length > 0 && <p>Choose which details to keep for each replay.</p>}
    {isImport && incomingOnly.length > 0 && <p>New guest copies will be added to your account.</p>}
    {isImport && incomingOnly.length === 0 && conflicts.length === 0 && <p>These replays are already in your account.</p>}
    {!isImport && (localOnly.length > 0 || incomingOnly.length > 0) && <p>Copies removed on one side may be restored by this merge.</p>}
    {(incomingOnly.length > 0 || (!isImport && localOnly.length > 0)) && <details>
      <summary>{isImport
        ? `New copies (${incomingOnly.length})`
        : `Device-only (${localOnly.length}) and cloud-only (${incomingOnly.length}) copies`}</summary>
      <ul className={styles.reviewAdditions}>
        {[...(!isImport ? localOnly : []), ...incomingOnly].map((row) =>
          <li key={row.fingerprint}>{row.title} · {row.local ? localLabel : incomingLabel}</li>)}
      </ul>
    </details>}
    {conflicts.map((row) => <article className={styles.reviewEntry} key={row.fingerprint}>
      <h4>{row.title}</h4>
      {row.local && row.incoming && <div className={styles.reviewSides}>
        <Version entry={row.local} source={localLabel} choice="local" title={row.title}
          selected={choices[row.fingerprint] === "local"} disabled={busy}
          onChoose={(choice) => setChoices((current) => ({ ...current, [row.fingerprint]: choice }))} />
        <Version entry={row.incoming} source={incomingLabel} choice="incoming" title={row.title}
          selected={choices[row.fingerprint] === "incoming"} disabled={busy}
          onChoose={(choice) => setChoices((current) => ({ ...current, [row.fingerprint]: choice }))} />
      </div>}
    </article>)}
    <div className={styles.actions}>
      <button className="uiAction uiActionPrimary" type="button" disabled={disabled} onClick={() => onApply(choices)}>
        <Icon name={isImport ? "copy" : "sync"} />{isImport ? "Copy reviewed replays" : "Merge reviewed replays"}
      </button>
      <button className="uiAction uiActionNeutral" type="button" disabled={busy} onClick={onCancel}>Cancel</button>
    </div>
  </section>;
}
