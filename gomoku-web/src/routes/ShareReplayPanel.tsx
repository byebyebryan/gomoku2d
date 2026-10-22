import { useMemo, useRef, useState } from "react";

import type { SavedMatchV2 } from "../match/saved_match";
import { portableReplayFromSavedMatch, type PortableReplayV1 } from "../replay/portable_replay";
import { encodePortableReplayFragment } from "../replay/portable_replay_codec";

import styles from "./ReplayRoute.module.css";

export function ShareReplayPanel({ match, moveIndex, source }: {
  match: SavedMatchV2;
  moveIndex: number;
  source?: PortableReplayV1;
}) {
  const [open, setOpen] = useState(false);
  const [currentMoment, setCurrentMoment] = useState(true);
  const [genericNames, setGenericNames] = useState(false);
  const [includeProfiles, setIncludeProfiles] = useState(false);
  const [message, setMessage] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  const preview = useMemo(() => {
    if (!open) return null;
    try {
      const original = source ?? portableReplayFromSavedMatch(match).replay;
      const players = [original.black, original.white].map((player, index) => {
        if (player.kind === "bot") return player;
        return {
          kind: "human" as const,
          name: genericNames ? `Player ${index + 1}` : player.name,
          ...(!genericNames && includeProfiles && player.profileUid ? { profileUid: player.profileUid } : {}),
        };
      });
      const replay: PortableReplayV1 = {
        ...original,
        black: players[0]!,
        white: players[1]!,
        viewPly: currentMoment ? moveIndex : match.move_count,
      };
      const fragment = encodePortableReplayFragment(replay);
      const url = new URL(`${import.meta.env.BASE_URL}replay/shared/`, window.location.origin);
      url.hash = fragment;
      return { replay, url: url.href, error: null };
    } catch (error) {
      return { replay: null, url: "", error: error instanceof Error ? error.message : "This game cannot be shared." };
    }
  }, [currentMoment, genericNames, includeProfiles, match, moveIndex, open, source]);

  return (
    <section className={`${styles.deckSection} ${styles.sharingSection}`} aria-label="Replay sharing">
      <button className="uiAction uiActionSecondary" type="button" aria-expanded={open} onClick={() => {
        setOpen(!open); setMessage("");
      }}>Share replay</button>
      {open && <>
        <p className={styles.notFoundText}>The link contains a copy of this game and the player labels shown below.</p>
        <label><input type="checkbox" checked={currentMoment} onChange={(event) => {
          setCurrentMoment(event.target.checked); setMessage("");
        }} /> Open at current move ({moveIndex})</label>
        <label><input type="checkbox" checked={genericNames} onChange={(event) => {
          setGenericNames(event.target.checked); setMessage("");
        }} /> Use generic player names</label>
        <label><input type="checkbox" checked={includeProfiles} disabled={genericNames} onChange={(event) => {
          setIncludeProfiles(event.target.checked); setMessage("");
        }} /> Include available player profile links</label>
        {preview?.replay && <p>{preview.replay.black.name} · {preview.replay.white.name}</p>}
        {preview?.error ? <p role="alert">{preview.error}</p> : <>
          <label>Replay link
            <input ref={inputRef} className={styles.shareInput} readOnly value={preview?.url ?? ""}
              onFocus={(event) => event.currentTarget.select()} />
          </label>
          <button className="uiAction uiActionPrimary" type="button" onClick={async () => {
            if (!preview?.url) return;
            try {
              await navigator.clipboard.writeText(preview.url);
              setMessage("Link copied.");
            } catch {
              inputRef.current?.focus(); inputRef.current?.select();
              setMessage("Copy the selected link above.");
            }
          }}>Copy replay link</button>
        </>}
        <p role="status" aria-live="polite">{message}</p>
      </>}
    </section>
  );
}
