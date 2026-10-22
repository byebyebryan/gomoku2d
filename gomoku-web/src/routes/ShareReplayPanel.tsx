import { useEffect, useId, useMemo, useRef, useState } from "react";

import type { SavedMatchV2 } from "../match/saved_match";
import { portableReplayFromSavedMatch, type PortableReplayV1 } from "../replay/portable_replay";
import { encodePortableReplayFragment } from "../replay/portable_replay_codec";
import { Icon } from "../ui/Icon";

import styles from "./ShareReplayPanel.module.css";

export function ShareReplayPanel({ match, moveIndex, source, open, onClose }: {
  match: SavedMatchV2;
  moveIndex: number;
  source?: PortableReplayV1;
  open: boolean;
  onClose: () => void;
}) {
  const [currentMoment, setCurrentMoment] = useState(true);
  const [genericNames, setGenericNames] = useState(false);
  const [includeProfiles, setIncludeProfiles] = useState(false);
  const [message, setMessage] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  const backRef = useRef<HTMLButtonElement>(null);
  const profileHintId = useId();

  useEffect(() => {
    if (!open) return;
    setMessage("");
    backRef.current?.focus({ preventScroll: true });
    backRef.current?.scrollIntoView?.({ block: "nearest" });
  }, [open]);
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
      const hasProfiles = [original.black, original.white].some((player) =>
        player.kind === "human" && Boolean(player.profileUid));
      return { replay, url: url.href, error: null, hasProfiles };
    } catch (error) {
      return { replay: null, url: "", error: error instanceof Error ? error.message : "This game cannot be shared.", hasProfiles: false };
    }
  }, [currentMoment, genericNames, includeProfiles, match, moveIndex, open, source]);

  return (
    <section className={styles.section} aria-label="Replay sharing" hidden={!open} onKeyDown={(event) => {
      if (event.key === "Escape") { event.preventDefault(); onClose(); }
    }}>
      <button className={`uiAction uiActionNeutral ${styles.back}`} type="button" ref={backRef} onClick={onClose}>
        <Icon name="back" /><span className="uiActionLabel">Back to replay</span>
      </button>
      <div className={styles.heading}>
        <Icon name="share" /><h2>Share replay</h2>
      </div>
      {open && <div className={styles.body}>
        <div className={styles.choice} role="group" aria-label="Replay opening position">
          <p className="uiSectionLabel">Open at</p>
          <div className={styles.segments}>
            <button className={currentMoment ? "uiSegment uiSegmentActive" : "uiSegment"} type="button" aria-pressed={currentMoment}
              onClick={() => { setCurrentMoment(true); setMessage(""); }}>Move {moveIndex}</button>
            <button className={!currentMoment ? "uiSegment uiSegmentActive" : "uiSegment"} type="button" aria-pressed={!currentMoment}
              onClick={() => { setCurrentMoment(false); setMessage(""); }}>Final move</button>
          </div>
        </div>
        <div className={styles.choice} role="group" aria-label="Player names">
          <p className="uiSectionLabel">Player names</p>
          <div className={styles.segments}>
            <button className={!genericNames ? "uiSegment uiSegmentActive" : "uiSegment"} type="button" aria-pressed={!genericNames}
              onClick={() => { setGenericNames(false); setMessage(""); }}>As played</button>
            <button className={genericNames ? "uiSegment uiSegmentActive" : "uiSegment"} type="button" aria-pressed={genericNames}
              onClick={() => { setGenericNames(true); setMessage(""); }}>Generic names</button>
          </div>
        </div>
        <label className={styles.profileOption}>
          <span className={styles.optionCopy}>
            <span className={styles.optionLabel}>Profile links</span>
            <span className={styles.optionState} id={profileHintId}>
              {genericNames ? "Off · Generic names" : preview?.hasProfiles ? includeProfiles ? "Included" : "Not included" : "None available"}
            </span>
          </span>
          <input className={styles.checkbox} type="checkbox" aria-label="Include available player profile links" aria-describedby={profileHintId}
            checked={includeProfiles && !genericNames && Boolean(preview?.hasProfiles)} disabled={genericNames || !preview?.hasProfiles}
            onChange={(event) => { setIncludeProfiles(event.target.checked); setMessage(""); }} />
        </label>
        {preview?.replay && <div className={styles.preview} role="group" aria-label="Link preview">
          <div className={styles.previewHeading}>
            <span className="uiSectionLabel">In the link</span>
            <span className={styles.moment}>Move {preview.replay.viewPly} / {match.move_count}</span>
          </div>
          {[preview.replay.black, preview.replay.white].map((player, index) => <div className={styles.player} key={index}>
            <span className={`${styles.stone} ${index === 0 ? styles.stoneBlack : styles.stoneWhite}`} aria-hidden="true" />
            <span className="uiSrOnly">{index === 0 ? "Black" : "White"}:</span>
            <span className={styles.playerName} title={player.name}>{player.name}</span>
            {player.kind === "human" && player.profileUid && <span className={styles.profileIcon} role="img" aria-label="Public profile link included"><Icon name="profile" /></span>}
          </div>)}
          <p className={styles.access}>Full game · Anyone with link</p>
        </div>}
        {preview?.error ? <p className={styles.error} role="alert">{preview.error}</p> : <>
          <button className={`uiAction uiActionPrimary ${styles.copyButton}`} type="button" onClick={async () => {
            if (!preview?.url) return;
            try {
              await navigator.clipboard.writeText(preview.url);
              setMessage("Link copied.");
            } catch {
              inputRef.current?.focus(); inputRef.current?.select();
              setMessage("Copy the selected link below.");
            }
          }}><Icon name="copy" />Copy link</button>
          <label className={styles.linkField}>
            <span className="uiSectionLabel">Replay link</span>
            <input ref={inputRef} className={`uiInput ${styles.linkInput}`} readOnly value={preview?.url ?? ""}
              onFocus={(event) => event.currentTarget.select()} spellCheck={false} />
          </label>
        </>}
        <p className={styles.message} role="status" aria-live="polite">{message}</p>
      </div>}
    </section>
  );
}
