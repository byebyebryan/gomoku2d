import { useEffect, useLayoutEffect, useRef, useState, type ReactNode, type RefObject } from "react";

import {
  privateMetadataCodePointCount,
  validatePrivateReplayMetadata,
  type PrivateReplayMetadata,
} from "../replay/private_replay_metadata";
import { Icon } from "../ui/Icon";
import styles from "./ShareReplayPanel.module.css";
import detailsStyles from "./ReplayDetailsPanel.module.css";

export interface ReplayDetailsBinding {
  kind: "private" | "public";
  value: PrivateReplayMetadata;
  onSave: (value: PrivateReplayMetadata, previous: PrivateReplayMetadata) => Promise<string | void>;
  onCancel?: () => void;
  returnLabel?: string;
}

export interface ReplayDraftState {
  dirty: boolean;
  saving: boolean;
}

function sameDetails(left: PrivateReplayMetadata, right: PrivateReplayMetadata) {
  return left.title === right.title && left.note === right.note && left.opening_ply === right.opening_ply;
}

export function ReplayDetailsPanel({ binding, move, total, playback, draftState, onBack, onSaved }: {
  binding: ReplayDetailsBinding;
  move: number;
  total: number;
  playback: ReactNode;
  draftState: RefObject<ReplayDraftState | null>;
  onBack: () => void;
  onSaved: (message?: string) => void;
}) {
  const isPublic = binding.kind === "public";
  const [original] = useState(binding.value);
  const [draft, setDraft] = useState(original);
  const [saving, setSaving] = useState(false);
  const [failure, setFailure] = useState("");
  const backRef = useRef<HTMLButtonElement>(null);
  const mounted = useRef(false);
  const dirty = !sameDetails(original, draft);
  const defaultOpening = isPublic ? total : null;
  const titleLength = privateMetadataCodePointCount(draft.title.trim());
  const noteLength = privateMetadataCodePointCount(draft.note.replace(/\r\n?/g, "\n").trim());
  let validated: PrivateReplayMetadata | undefined;
  let validation = "";
  try {
    validated = validatePrivateReplayMetadata(draft, total);
    if (isPublic && !validated.title) validation = "Enter a public title.";
  } catch (error) {
    validation = error instanceof Error ? error.message : "Check your replay details.";
  }

  useLayoutEffect(() => {
    draftState.current = { dirty, saving };
    return () => { draftState.current = null; };
  }, [dirty, draftState, saving]);
  useEffect(() => {
    mounted.current = true;
    // Public editing initially opens at the top of the replay. Focusing the
    // control avoids jumping the mobile viewport down past the board.
    backRef.current?.focus({ preventScroll: true });
    if (!isPublic) backRef.current?.scrollIntoView?.({ block: "nearest" });
    return () => { mounted.current = false; };
  }, [isPublic]);

  return <section className={styles.section} aria-label={isPublic ? "Edit publication" : "Replay details"}
    onKeyDown={(event) => { if (event.key === "Escape") { event.preventDefault(); onBack(); } }}>
    <button className={`uiAction uiActionNeutral ${styles.back}`} type="button" ref={backRef} onClick={onBack} disabled={saving}>
      <Icon name="back" /><span className="uiActionLabel">{binding.returnLabel ?? "Back to replay"}</span>
    </button>
    <div className={styles.heading}>
      <Icon name={isPublic ? "publish" : "settings"} /><h2>{isPublic ? "Edit publication" : "Replay details"}</h2>
    </div>
    <form className={styles.body} onSubmit={async (event) => {
      event.preventDefault();
      if (!validated || validation || !dirty || saving) return;
      setSaving(true);
      setFailure("");
      try {
        const message = await binding.onSave(validated, original);
        if (mounted.current) onSaved(message || undefined);
      } catch (error) {
        if (mounted.current) setFailure(error instanceof Error ? error.message : "Replay details could not be saved.");
      } finally {
        if (mounted.current) setSaving(false);
      }
    }}>
      <label className={styles.choice}>
        <span className="uiSectionLabel">{isPublic ? "Public title" : "Private title"}</span>
        <input className="uiInput" value={draft.title} disabled={saving} placeholder={isPublic ? undefined : "Player names"}
          onChange={(event) => setDraft({ ...draft, title: event.currentTarget.value })} />
        {titleLength > 68 && <span className={detailsStyles.counter}>{titleLength} / 80 characters</span>}
      </label>
      {!isPublic && <label className={styles.choice}>
        <span className="uiSectionLabel">Private note</span>
        <textarea className={`uiInput ${detailsStyles.note}`} rows={3} value={draft.note} disabled={saving}
          onChange={(event) => setDraft({ ...draft, note: event.currentTarget.value })} />
        {noteLength > 240 && <span className={detailsStyles.counter}>{noteLength} / 280 characters</span>}
      </label>}
      <div className={styles.choice} role="group" aria-label="Replay opening position">
        <p className="uiSectionLabel">Open at</p>
        <p className={detailsStyles.opening} data-testid="replay-details-opening" aria-live="polite">
          {draft.opening_ply === null || draft.opening_ply === total ? "Final move" : `Move ${draft.opening_ply} / ${total}`}
        </p>
        <div className={detailsStyles.playback}>{playback}</div>
        <div className={styles.segments}>
          <button className="uiSegment" type="button" disabled={saving} onClick={() => setDraft({ ...draft, opening_ply: move })}>Use move {move}</button>
          <button className="uiSegment" type="button" disabled={saving || draft.opening_ply === defaultOpening}
            onClick={() => setDraft({ ...draft, opening_ply: defaultOpening })}>{isPublic ? "Final move" : "Use default"}</button>
        </div>
      </div>
      {(failure || (dirty && validation)) && <p className={styles.error} role="alert">{failure || validation}</p>}
      <button className={`uiAction uiActionPrimary ${styles.copyButton}`} type="submit" disabled={!dirty || saving || !!validation}>
        <Icon name="save" />{saving ? "Saving…" : isPublic ? "Save public changes" : "Save details"}
      </button>
    </form>
  </section>;
}
