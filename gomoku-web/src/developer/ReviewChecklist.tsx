export function ReviewChecklist() {
  return (
    <details className="dev-disclosure review-checklist">
      <summary>UI review checklist</summary>
      <div>
        <ul>
          <li>Keep the board dominant on match and replay screens.</li>
          <li>Keep controls reachable in portrait and short landscape layouts.</li>
          <li>Check that replay analysis leaves room for the board and its controls.</li>
          <li>Inspect stone colours, hint markers and overlays against the board.</li>
          <li>Compare typography, navigation, spacing and icons with the existing UI.</li>
          <li>Inspect long records and enlarged text for clipping or unreachable content.</li>
          <li>Check the capture's release, content and conditions before comparing it.</li>
        </ul>
        <a href="https://github.com/byebyebryan/gomoku2d/blob/main/docs/working/ui_screenshot_review.md">
          Screenshot review procedure
        </a>
      </div>
    </details>
  );
}
