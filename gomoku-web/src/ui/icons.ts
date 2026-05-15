import backSvgRaw from "../../assets/icons/back.svg?raw";
import botSvgRaw from "../../assets/icons/bot.svg?raw";
import fastForwardSvgRaw from "../../assets/icons/fast_forward.svg?raw";
import fastRewindSvgRaw from "../../assets/icons/fast_rewind.svg?raw";
import firstSvgRaw from "../../assets/icons/first.svg?raw";
import homeSvgRaw from "../../assets/icons/home.svg?raw";
import humanSvgRaw from "../../assets/icons/human.svg?raw";
import lastSvgRaw from "../../assets/icons/last.svg?raw";
import playSvgRaw from "../../assets/icons/play.svg?raw";
import plusSvgRaw from "../../assets/icons/plus.svg?raw";
import prevSvgRaw from "../../assets/icons/prev.svg?raw";
import nextSvgRaw from "../../assets/icons/next.svg?raw";
import pauseSvgRaw from "../../assets/icons/pause.svg?raw";
import profileSvgRaw from "../../assets/icons/profile.svg?raw";
import replaySvgRaw from "../../assets/icons/replay.svg?raw";
import resetSvgRaw from "../../assets/icons/reset.svg?raw";
import settingsSvgRaw from "../../assets/icons/settings.svg?raw";
import undoSvgRaw from "../../assets/icons/undo.svg?raw";

function sanitize(svg: string): string {
  return svg.replace(/<title>.*?<\/title>\s*/s, "");
}

export const iconSvg = {
  back: sanitize(backSvgRaw),
  bot: sanitize(botSvgRaw),
  fastForward: sanitize(fastForwardSvgRaw),
  fastRewind: sanitize(fastRewindSvgRaw),
  first: sanitize(firstSvgRaw),
  home: sanitize(homeSvgRaw),
  human: sanitize(humanSvgRaw),
  last: sanitize(lastSvgRaw),
  next: sanitize(nextSvgRaw),
  pause: sanitize(pauseSvgRaw),
  play: sanitize(playSvgRaw),
  plus: sanitize(plusSvgRaw),
  prev: sanitize(prevSvgRaw),
  profile: sanitize(profileSvgRaw),
  replay: sanitize(replaySvgRaw),
  reset: sanitize(resetSvgRaw),
  settings: sanitize(settingsSvgRaw),
  undo: sanitize(undoSvgRaw),
} as const;

export type IconName = keyof typeof iconSvg;
