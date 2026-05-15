import { useEffect } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useStore } from "zustand";

import {
  customConfigForPracticeBot,
  labSpecForPracticeBot,
  practiceBotLabel,
  type PracticeBotConfig,
  type PracticeBotDepth,
  type PracticeBotPresetId,
  type PracticeBotWidth,
} from "../core/practice_bot_config";
import {
  applySavedLocalMatchSetup,
  localMatchSessionStore,
  startLocalMatchWithSavedSetup,
} from "../game/local_match_session";
import { localProfileStore } from "../profile/local_profile_store";
import { variantLabel } from "../replay/local_replay";
import { Icon } from "../ui/Icon";

import styles from "./SettingsRoute.module.css";

const PRESET_IDS: PracticeBotPresetId[] = ["easy", "normal", "hard"];
const DEPTHS: PracticeBotDepth[] = [1, 3, 5, 7];
const WIDTHS: PracticeBotWidth[] = ["none", 8, 16];

function setupSummary(config: PracticeBotConfig): string {
  const custom = customConfigForPracticeBot(config);
  return [
    `D${custom.depth}`,
    custom.width === "none" ? "full width" : `width ${custom.width}`,
    custom.patternScoring ? "pattern" : "line",
    custom.corridorProof ? "proof" : null,
  ].filter(Boolean).join(" · ");
}

function settingsEqual(left: PracticeBotConfig, right: PracticeBotConfig): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function SettingsRoute() {
  const navigate = useNavigate();
  const settings = useStore(localProfileStore, (state) => state.settings);
  const matchStore = useStore(localMatchSessionStore, (state) => state.matchStore);
  const activeMatch = matchStore?.getState() ?? null;
  const custom = customConfigForPracticeBot(settings.practiceBot);
  const activeSetupDiffers = Boolean(
    activeMatch
      && (
        activeMatch.currentVariant !== settings.preferredVariant
        || !settingsEqual(activeMatch.currentPracticeBot, settings.practiceBot)
      ),
  );

  useEffect(() => {
    localProfileStore.getState().ensureLocalProfile();
  }, []);

  const updateSetup = (patch: Partial<typeof settings>) => {
    localProfileStore.getState().updateSettings(patch);
    applySavedLocalMatchSetup();
  };

  const updateBot = (practiceBot: PracticeBotConfig) => {
    updateSetup({ practiceBot });
  };

  const updateCustomBot = (patch: Partial<Omit<ReturnType<typeof customConfigForPracticeBot>, "mode" | "version">>) => {
    updateBot({
      ...custom,
      ...patch,
      mode: "custom",
      version: 1,
    });
  };

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div className={styles.headerCopy}>
          <p className="uiPageEyebrow">Saved settings</p>
          <h1 className={styles.title}>Game Settings</h1>
        </div>
        <div className={styles.headerActions}>
          <Link className="uiAction uiActionPrimary" to="/match/local">
            <Icon className="uiIconDesktop" name="play" />
            <span className="uiActionLabel">Back to Game</span>
          </Link>
          <Link className="uiAction uiActionSecondary" to="/profile">
            <Icon className="uiIconDesktop" name="profile" />
            <span className="uiActionLabel">Profile</span>
          </Link>
          <Link className="uiAction uiActionNeutral" to="/">
            <Icon className="uiIconDesktop" name="home" />
            <span className="uiActionLabel">Home</span>
          </Link>
        </div>
      </header>

      <section className={styles.layout}>
        <aside className={styles.summaryPanel}>
          <p className="uiSectionLabel">Current settings</p>
          <h2 className={styles.summaryTitle}>
            {variantLabel(settings.preferredVariant)} · {practiceBotLabel(settings.practiceBot)}
          </h2>
          <p className={styles.summaryText}>{setupSummary(settings.practiceBot)}</p>
          <p className={styles.labSpec}>{labSpecForPracticeBot(settings.practiceBot)}</p>

          {activeSetupDiffers ? (
            <div className={styles.applyPanel}>
              <p className={styles.applyTitle}>Saved settings apply next game.</p>
              <p className={styles.applyText}>
                Keep playing the current game, or start fresh with this setup now.
              </p>
              <div className={styles.applyActions}>
                <Link className="uiAction uiActionSecondary" to="/match/local">
                  <span className="uiActionLabel">Back to Game</span>
                </Link>
                <button
                  className="uiAction uiActionPrimary"
                  onClick={() => {
                    startLocalMatchWithSavedSetup();
                    navigate("/match/local");
                  }}
                  type="button"
                >
                  <span className="uiActionLabel">Start New Game</span>
                </button>
              </div>
            </div>
          ) : null}
        </aside>

        <section className={styles.controlsPanel}>
          <section className={styles.controlSection}>
            <div className={styles.labRow}>
              <div className={styles.labCopy}>
                <p className={styles.labLabel}>Rule</p>
                <p className={styles.labHint}>Ruleset for new games.</p>
              </div>
              <div className={styles.segmentGrid}>
                {(["freestyle", "renju"] as const).map((variant) => (
                  <button
                    className={settings.preferredVariant === variant ? "uiSegment uiSegmentActive" : "uiSegment"}
                    key={variant}
                    onClick={() => updateSetup({ preferredVariant: variant })}
                    type="button"
                  >
                    {variantLabel(variant)}
                  </button>
                ))}
              </div>
            </div>
          </section>

          <div className="uiDivider" />

          <section className={styles.controlSection}>
            <div className={styles.sectionHeader}>
              <p className="uiSectionLabel">Bot</p>
              <p className={styles.sectionValue}>{practiceBotLabel(settings.practiceBot)}</p>
            </div>
            <div className={styles.presetGrid}>
              {PRESET_IDS.map((preset) => {
                const config: PracticeBotConfig = { mode: "preset", preset, version: 1 };
                return (
                  <button
                    className={
                      settings.practiceBot.mode === "preset" && settings.practiceBot.preset === preset
                        ? `${styles.presetCard} ${styles.presetCardActive}`
                        : styles.presetCard
                    }
                    key={preset}
                    onClick={() => updateBot(config)}
                    type="button"
                  >
                    <span className={styles.presetName}>{practiceBotLabel(config)}</span>
                    <span className={styles.presetDetails}>{setupSummary(config)}</span>
                  </button>
                );
              })}
              <button
                className={
                  settings.practiceBot.mode === "custom"
                    ? `${styles.presetCard} ${styles.presetCardActive}`
                    : styles.presetCard
                }
                onClick={() => {
                  updateBot({
                    ...custom,
                    mode: "custom",
                    version: 1,
                  });
                }}
                type="button"
              >
                <span className={styles.presetName}>Custom</span>
                <span className={styles.presetDetails}>Tune the search knobs</span>
              </button>
            </div>
          </section>

          <section className={`${styles.controlSection} ${styles.labSection}`}>
            <div className={styles.sectionHeader}>
              <p className="uiSectionLabel">Lab Controls</p>
              <p className={styles.sectionValue}>Custom bot</p>
            </div>

            <div className={styles.labRows}>
              <div className={styles.labRow}>
                <div className={styles.labCopy}>
                  <p className={styles.labLabel}>Depth</p>
                  <p className={styles.labHint}>How far the bot searches before scoring a position.</p>
                </div>
                <div className={styles.segmentGridFour}>
                  {DEPTHS.map((depth) => (
                    <button
                      className={custom.depth === depth ? "uiSegment uiSegmentActive" : "uiSegment"}
                      key={depth}
                      onClick={() => updateCustomBot({ depth })}
                      type="button"
                    >
                      D{depth}
                    </button>
                  ))}
                </div>
              </div>

              <div className={styles.labRow}>
                <div className={styles.labCopy}>
                  <p className={styles.labLabel}>Width</p>
                  <p className={styles.labHint}>Cap searched child moves after tactical ordering.</p>
                </div>
                <div className={styles.segmentGridThree}>
                  {WIDTHS.map((width) => (
                    <button
                      className={custom.width === width ? "uiSegment uiSegmentActive" : "uiSegment"}
                      key={width}
                      onClick={() => updateCustomBot({ width })}
                      type="button"
                    >
                      {width === "none" ? "Full" : `W${width}`}
                    </button>
                  ))}
                </div>
              </div>

              <div className={styles.labRow}>
                <div className={styles.labCopy}>
                  <p className={styles.labLabel}>Scoring</p>
                  <p className={styles.labHint}>Static evaluation used at search leaves.</p>
                </div>
                <div className={styles.segmentGrid}>
                  <button
                    className={!custom.patternScoring ? "uiSegment uiSegmentActive" : "uiSegment"}
                    onClick={() => updateCustomBot({ patternScoring: false })}
                    type="button"
                  >
                    Simple geometry
                  </button>
                  <button
                    className={custom.patternScoring ? "uiSegment uiSegmentActive" : "uiSegment"}
                    onClick={() => updateCustomBot({ patternScoring: true })}
                    type="button"
                  >
                    Threat pattern
                  </button>
                </div>
              </div>

              <div className={styles.labRow}>
                <div className={styles.labCopy}>
                  <p className={styles.labLabel}>Extra pass</p>
                  <p className={styles.labHint}>Optional forced-line proof after search.</p>
                </div>
                <div className={styles.segmentGrid}>
                  <button
                    className={!custom.corridorProof ? "uiSegment uiSegmentActive" : "uiSegment"}
                    onClick={() => updateCustomBot({ corridorProof: false })}
                    type="button"
                  >
                    None
                  </button>
                  <button
                    className={custom.corridorProof ? "uiSegment uiSegmentActive" : "uiSegment"}
                    onClick={() => updateCustomBot({ corridorProof: true })}
                    type="button"
                  >
                    Corridor proof
                  </button>
                </div>
              </div>
            </div>
          </section>
        </section>
      </section>
    </main>
  );
}
