import type { IconName } from "./icons";
import { iconSvg } from "./icons";

type IconProps = {
  className?: string;
  name: IconName;
};

export function Icon({ className = "", name }: IconProps) {
  return (
    <span
      aria-hidden="true"
      className={["uiIcon", className].filter(Boolean).join(" ")}
      dangerouslySetInnerHTML={{ __html: iconSvg[name] }}
    />
  );
}
