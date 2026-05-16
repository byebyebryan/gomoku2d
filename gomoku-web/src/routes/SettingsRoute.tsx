import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { useStore } from "zustand";

import {
  customConfigForPracticeBot,
  practiceBotConfigSummary,
  practiceBotLabel,
  practiceBotPlayerName,
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
import {
  uiPreferencesStore,
  type ImmediateHintMode,
  type ImminentHintMode,
} from "../profile/ui_preferences_store";
import { variantLabel } from "../replay/local_replay";
import { Icon } from "../ui/Icon";

import styles from "./SettingsRoute.module.css";

const PRESET_IDS: PracticeBotPresetId[] = ["easy", "normal", "hard"];
const DEPTHS: PracticeBotDepth[] = [1, 3, 5, 7];
const WIDTHS: PracticeBotWidth[] = ["none", 8, 16];
const COMPACT_SETTINGS_QUERY = "(max-width: 760px)";
const MOBILE_TOUCH_QUERY =
  "(max-width: 720px) and (orientation: portrait) and (hover: none) and (pointer: coarse)";

function setupSummary(config: PracticeBotConfig): string {
  return practiceBotConfigSummary(config);
}

function settingsEqual(left: PracticeBotConfig, right: PracticeBotConfig): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

function HintModeRow<TMode extends string>({
  hint,
  label,
  onSelect,
  options,
  selected,
}: {
  hint: string;
  label: string;
  onSelect: (value: TMode) => void;
  options: Array<{ label: string; value: TMode }>;
  selected: TMode;
}) {
  return (
    <div className={styles.labRow}>
      <div className={styles.labCopy}>
        <p className={styles.labLabel}>{label}</p>
        <p className={styles.labHint}>{hint}</p>
      </div>
      <div aria-label={`${label} hints`} className={styles.segmentGridThree} role="group">
        {options.map((option) => (
          <button
            className={selected === option.value ? "uiSegment uiSegmentActive" : "uiSegment"}
            key={option.value}
            onClick={() => onSelect(option.value)}
            type="button"
          >
            {option.label}
          </button>
        ))}
      </div>
    </div>
  );
}

const IMMEDIATE_HINT_OPTIONS: Array<{ label: string; value: ImmediateHintMode }> = [
  { label: "Off", value: "off" },
  { label: "Win", value: "win" },
  { label: "+ Lose", value: "win_threat" },
];

const IMMINENT_HINT_OPTIONS: Array<{ label: string; value: ImminentHintMode }> = [
  { label: "Off", value: "off" },
  { label: "Threat", value: "threat" },
  { label: "+ Counter", value: "threat_counter" },
];

export function SettingsRoute() {
  const navigate = useNavigate();
  const [compactSettingsLayout, setCompactSettingsLayout] = useState(false);
  const [showTouchControls, setShowTouchControls] = useState(false);
  const settings = useStore(localProfileStore, (state) => state.settings);
  const boardHints = useStore(uiPreferencesStore, (state) => state.boardHints);
  const touchControl = useStore(uiPreferencesStore, (state) => state.touchControl);
  const matchStore = useStore(localMatchSessionStore, (state) => state.matchStore);
  const activeMatch = matchStore?.getState() ?? null;
  const custom = customConfigForPracticeBot(settings.practiceBot);
  const currentBotLabel = practiceBotPlayerName(settings.practiceBot);
  const currentVariantLabel = variantLabel(settings.preferredVariant);
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

  useEffect(() => {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
      return undefined;
    }

    const compactQuery = window.matchMedia(COMPACT_SETTINGS_QUERY);
    const mediaQuery = window.matchMedia(MOBILE_TOUCH_QUERY);
    const syncCompact = () => setCompactSettingsLayout(compactQuery.matches);
    const syncTouch = () => setShowTouchControls(mediaQuery.matches);

    syncCompact();
    syncTouch();
    compactQuery.addEventListener("change", syncCompact);
    mediaQuery.addEventListener("change", syncTouch);
    return () => {
      compactQuery.removeEventListener("change", syncCompact);
      mediaQuery.removeEventListener("change", syncTouch);
    };
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
  const showCurrentSummary = !compactSettingsLayout;
  const showSummaryPanel = showCurrentSummary || activeSetupDiffers;

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <div className={styles.headerCopy}>
          <p className="uiPageEyebrow">Play setup</p>
          <h1 className={styles.title}>Settings</h1>
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
        {showSummaryPanel ? (
          <aside className={styles.summaryPanel}>
            {showCurrentSummary ? (
              <div className={styles.summaryCurrent}>
                <p className="uiSectionLabel">Current settings</p>
                <div className={styles.summaryStack} aria-label={`${currentVariantLabel} ${currentBotLabel}`} role="group">
                  <section className={styles.summaryGroup}>
                    <p className={styles.summaryKicker}>Game</p>
                    <h2 className={styles.summaryTitle}>{currentVariantLabel}</h2>
                  </section>
                  <section className={styles.summaryGroup}>
                    <p className={styles.summaryKicker}>Bot</p>
                    <h2 className={styles.summaryTitle}>{currentBotLabel}</h2>
                    <p className={styles.summaryText}>{setupSummary(settings.practiceBot)}</p>
                  </section>
                </div>
              </div>
            ) : null}

            {activeSetupDiffers ? (
              <div className={styles.applyPanel}>
                <p className={styles.applyTitle}>Saved settings apply next game.</p>
                <p className={styles.applyText}>
                  Keep playing the current game, or start fresh with this setup now.
                </p>
                <div className={styles.applyActions}>
                  <Link className="uiAction uiActionPrimary" to="/match/local">
                    <span className="uiActionLabel">Back to Game</span>
                  </Link>
                  <button
                    className="uiAction uiActionAccent"
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
        ) : null}

        <section className={styles.controlsPanel}>
          <section className={styles.controlSection}>
            <div className={styles.sectionHeader}>
              <p className="uiSectionLabel">Game</p>
            </div>
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

          {showTouchControls ? (
            <>
              <section className={styles.controlSection}>
                <div className={styles.sectionHeader}>
                  <p className="uiSectionLabel">Controls</p>
                </div>
                <div className={styles.labRow}>
                  <div className={styles.labCopy}>
                    <p className={styles.labLabel}>Touch control</p>
                    <p className={styles.labHint}>How mobile taps move the board cursor.</p>
                  </div>
                  <div className={styles.segmentGrid}>
                    <button
                      className={touchControl === "pointer" ? "uiSegment uiSegmentActive" : "uiSegment"}
                      onClick={() => uiPreferencesStore.getState().setTouchControl("pointer")}
                      type="button"
                    >
                      Pointer
                    </button>
                    <button
                      className={touchControl === "touchpad" ? "uiSegment uiSegmentActive" : "uiSegment"}
                      onClick={() => uiPreferencesStore.getState().setTouchControl("touchpad")}
                      type="button"
                    >
                      Touchpad
                    </button>
                  </div>
                </div>
              </section>

              <div className="uiDivider" />
            </>
          ) : null}

          <section className={styles.controlSection}>
            <div className={styles.sectionHeader}>
              <p className="uiSectionLabel">Hints</p>
            </div>
            <div className={styles.labRows}>
              <HintModeRow
                hint="One-move wins and urgent blocks."
                label="Immediate"
                onSelect={(immediate) => uiPreferencesStore.getState().setBoardHints({ immediate })}
                options={IMMEDIATE_HINT_OPTIONS}
                selected={boardHints.immediate}
              />
              <HintModeRow
                hint="Open/broken-three replies and counter threats."
                label="Imminent"
                onSelect={(imminent) => uiPreferencesStore.getState().setBoardHints({ imminent })}
                options={IMMINENT_HINT_OPTIONS}
                selected={boardHints.imminent}
              />
            </div>
          </section>

          <div className="uiDivider" />

          <section className={styles.controlSection}>
            <div className={styles.sectionHeader}>
              <p className="uiSectionLabel">Bot</p>
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
              <p className="uiSectionLabel">Advanced Controls</p>
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
                      {width === "none" ? "full" : `W${width}`}
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
                    Simple
                  </button>
                  <button
                    className={custom.patternScoring ? "uiSegment uiSegmentActive" : "uiSegment"}
                    onClick={() => updateCustomBot({ patternScoring: true })}
                    type="button"
                  >
                    Threat
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
