import botSvgRaw from "../../assets/icons/bot.svg?raw";
import backSvgRaw from "../../assets/icons/back.svg?raw";
import copySvgRaw from "../../assets/icons/copy.svg?raw";
import doubleNextSvgRaw from "../../assets/icons/double_next.svg?raw";
import doublePrevSvgRaw from "../../assets/icons/double_prev.svg?raw";
import exitSvgRaw from "../../assets/icons/exit.svg?raw";
import homeSvgRaw from "../../assets/icons/home.svg?raw";
import helpSvgRaw from "../../assets/icons/help.svg?raw";
import humanSvgRaw from "../../assets/icons/human.svg?raw";
import minusSvgRaw from "../../assets/icons/minus.svg?raw";
import playSvgRaw from "../../assets/icons/play.svg?raw";
import plusSvgRaw from "../../assets/icons/plus.svg?raw";
import prevSvgRaw from "../../assets/icons/prev.svg?raw";
import nextSvgRaw from "../../assets/icons/next.svg?raw";
import pauseSvgRaw from "../../assets/icons/pause.svg?raw";
import profileSvgRaw from "../../assets/icons/profile.svg?raw";
import publishSvgRaw from "../../assets/icons/publish.svg?raw";
import replaySvgRaw from "../../assets/icons/replay.svg?raw";
import resetSvgRaw from "../../assets/icons/reset.svg?raw";
import saveSvgRaw from "../../assets/icons/save.svg?raw";
import settingsSvgRaw from "../../assets/icons/settings.svg?raw";
import shareSvgRaw from "../../assets/icons/share.svg?raw";
import signInSvgRaw from "../../assets/icons/sign_in.svg?raw";
import signOutSvgRaw from "../../assets/icons/sign_out.svg?raw";
import syncSvgRaw from "../../assets/icons/sync.svg?raw";
import trashSvgRaw from "../../assets/icons/trash.svg?raw";
import undoSvgRaw from "../../assets/icons/undo.svg?raw";

function sanitize(svg: string): string {
  return svg.replace(/<title>.*?<\/title>\s*/s, "");
}

export const iconSvg = {
  back: sanitize(backSvgRaw),
  bot: sanitize(botSvgRaw),
  copy: sanitize(copySvgRaw),
  doubleNext: sanitize(doubleNextSvgRaw),
  doublePrev: sanitize(doublePrevSvgRaw),
  exit: sanitize(exitSvgRaw),
  home: sanitize(homeSvgRaw),
  help: sanitize(helpSvgRaw),
  human: sanitize(humanSvgRaw),
  minus: sanitize(minusSvgRaw),
  next: sanitize(nextSvgRaw),
  pause: sanitize(pauseSvgRaw),
  play: sanitize(playSvgRaw),
  plus: sanitize(plusSvgRaw),
  prev: sanitize(prevSvgRaw),
  profile: sanitize(profileSvgRaw),
  publish: sanitize(publishSvgRaw),
  replay: sanitize(replaySvgRaw),
  reset: sanitize(resetSvgRaw),
  save: sanitize(saveSvgRaw),
  settings: sanitize(settingsSvgRaw),
  share: sanitize(shareSvgRaw),
  signIn: sanitize(signInSvgRaw),
  signOut: sanitize(signOutSvgRaw),
  sync: sanitize(syncSvgRaw),
  trash: sanitize(trashSvgRaw),
  undo: sanitize(undoSvgRaw),
} as const;

export type IconName = keyof typeof iconSvg;
