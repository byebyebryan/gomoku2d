#!/usr/bin/env python3
"""Mine local Codex session logs for Gomoku2D process-story evidence.

This is intentionally a private review tool. It emits normalized evidence into
ignored output files so the public story can be written from curated facts
instead of raw chat dumps.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from collections import Counter, defaultdict
from dataclasses import dataclass, field
from datetime import date, datetime
from pathlib import Path
from typing import Any


REPO_NAME = "gomoku2d"
REPO_ROOT = Path(__file__).resolve().parents[1]
REPO_PATH = str(REPO_ROOT)

SKIP_TEXT_MARKERS = (
    "<permissions instructions>",
    "<collaboration_mode>",
    "<apps_instructions>",
    "<skills_instructions>",
    "<environment_context>",
    "<INSTRUCTIONS>",
    "========= MEMORY_SUMMARY BEGINS =========",
    "encrypted_content",
)

SELF_REFERENCE_REGEX = re.compile(
    r"process[- ]story|extract_process_story|evidence_events\.jsonl|"
    r"conversation_arcs\.(jsonl|md)|quote_candidates\.md|process_outline\.md|"
    r"session_index\.json|git_chronology\.json",
    re.IGNORECASE,
)

TOPIC_PATTERNS: dict[str, tuple[str, ...]] = {
    "revival_stack": (
        r"\bold\b.*\bproject\b",
        r"\brevive|revival|revived\b",
        r"\bdecade\b",
        r"\bRust\b",
        r"\bWasm|WebAssembly\b",
        r"\bPhaser\b",
        r"\bReact\b",
        r"\bmodern .* stack\b",
    ),
    "product_foundation": (
        r"\blocal[- ]first\b",
        r"\bguest\b",
        r"\bprofile\b",
        r"\bhistory\b",
        r"\bcloud\b",
        r"\bFirebase\b",
        r"\bFirestore\b",
        r"\bsign[- ]in\b",
        r"\btrusted\b",
    ),
    "bot_lab_reports": (
        r"\bbot lab\b",
        r"\bbot report\b",
        r"\blab report\b",
        r"\btournament\b",
        r"\bgauntlet\b",
        r"\bbenchmark\b",
        r"\banchor\b",
        r"\bsearch[- ]d\d\b",
    ),
    "corridor_analysis": (
        r"\bcorridor\b",
        r"\bforced corridor\b",
        r"\bsetup corridor\b",
        r"\blast escape\b",
        r"\bescape\b",
        r"\bproof\b",
        r"\bforced lose\b",
    ),
    "rolling_frontier": (
        r"\brolling frontier\b",
        r"\bfrontier\b",
        r"\bthreat view\b",
        r"\bscan\b",
        r"\bshadow\b",
    ),
    "renju_correctness": (
        r"\bRenju\b",
        r"\bforbidden\b",
        r"\bdouble[- ]three\b",
        r"\bdouble[- ]four\b",
        r"\brecursive\b",
        r"\bRenju\.net\b",
        r"\bPiskvork\b",
    ),
    "replay_analysis": (
        r"\breplay analysis\b",
        r"\banaly[sz]er\b",
        r"\banalysis report\b",
        r"\blethal onset\b",
        r"\bmistake\b",
        r"\bmissed response\b",
        r"\bmissed escape\b",
    ),
    "ai_process": (
        r"\bAI\b",
        r"\bagent\b",
        r"\bagents\b",
        r"\bsubagent\b",
        r"\bGPT\b",
        r"\bworkflow\b",
        r"\bprocess\b",
        r"\bone developer\b",
        r"\bindie\b",
    ),
    "public_release": (
        r"\bpublic release\b",
        r"\bpublic[- ]readiness\b",
        r"\b0\.5\b",
        r"\bitch\.io\b",
        r"\bdev[- ]log\b",
        r"\bpolish\b",
        r"\brelease prep\b",
    ),
}

TOPIC_REGEX = {
    topic: tuple(re.compile(pattern, re.IGNORECASE) for pattern in patterns)
    for topic, patterns in TOPIC_PATTERNS.items()
}

PROJECT_REGEX = re.compile(
    rf"{re.escape(REPO_NAME)}|{re.escape(REPO_PATH)}|gomoku-bot-lab|gomoku-web|gomoku-analysis",
    re.IGNORECASE,
)

DOMAIN_REGEX = re.compile(
    r"gomoku|renju|forbidden move|open three|broken three|closed four|"
    r"lethal onset|threat corridor|corridor search|forced corridor|setup corridor|"
    r"replay analysis|rolling frontier|"
    r"threat view|tactical[- ]cap|search-d\d|five[- ]in[- ]a[- ]row",
    re.IGNORECASE,
)

OTHER_PROJECT_REGEX = re.compile(
    r"pylander|powered-descent-lab|pd-lab|home-lab|lazy-serializable|cubey|comfy",
    re.IGNORECASE,
)

GOMOKU_SPECIFIC_REGEX = re.compile(
    rf"{re.escape(REPO_PATH)}|gomoku-(web|core|wasm|bot-lab|analysis)|"
    r"renju|forbidden|open three|broken three|closed four|"
    r"lethal onset|corridor|tactical[- ]cap|search-d\d|replay analysis",
    re.IGNORECASE,
)

SIGNIFICANT_TOOL_REGEX = re.compile(
    r"\b("
    r"git\s+(commit|tag|push)|"
    r"scripts/release\.sh|"
    r"cargo\s+run\b.*\b(tournament|analyze-report-replays|report-json|renju-rules)|"
    r"npm\s+run\s+build|"
    r"firebase\s+deploy|"
    r"gh\s+release"
    r")\b",
    re.IGNORECASE,
)

HIGH_SIGNAL_USER_REGEX = re.compile(
    r"\b("
    r"plan|map out|what do you think|execute|implement|review|commit|release|"
    r"regen|rerun|publish|bug|issue|fix|cleanup|refactor|docs|roadmap|"
    r"analy[sz]er|analysis|corridor|renju|frontier|lab report|screenshot|"
    r"public release|ready|why|how|should|what about"
    r")\b",
    re.IGNORECASE,
)

ASSISTANT_DECISION_REGEX = re.compile(
    r"\b("
    r"recommend|should|I think|we should|the right shape|tradeoff|"
    r"plan|approach|direction|design|decision|root cause|because"
    r")\b",
    re.IGNORECASE,
)

ASSISTANT_OUTCOME_REGEX = re.compile(
    r"\b("
    r"implemented|fixed|patched|committed|verified|ran|passed|failed|"
    r"findings|changed|regressed|improved|published|released"
    r")\b",
    re.IGNORECASE,
)

ARC_WINDOW_BEFORE = 2
ARC_WINDOW_AFTER = 6
MAX_ARC_MESSAGES = 32
MAX_ARC_EXCERPTS = 8
MAX_ARC_EXCERPT_CHARS = 520
MAX_ARC_KEY_ITEMS = 3
ARC_TITLE_CHARS = 90
CONVERSATION_ARCS_PER_CHUNK = 100


@dataclass
class SessionSummary:
    source: str
    session_id: str | None = None
    session_cwd: str | None = None
    first_timestamp: str | None = None
    last_timestamp: str | None = None
    line_count: int = 0
    relevant_count: int = 0
    role_counts: Counter[str] = field(default_factory=Counter)
    topic_counts: Counter[str] = field(default_factory=Counter)
    include_reasons: Counter[str] = field(default_factory=Counter)


@dataclass
class ConversationMessage:
    source: str
    line: int
    session_id: str | None
    timestamp: str | None
    role: str
    event_kind: str
    text: str
    topics: list[str]
    release_band: str
    include_reasons: list[str]


def normalize_text(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip()


def truncate(text: str, max_chars: int) -> str:
    normalized = normalize_text(text)
    if len(normalized) <= max_chars:
        return normalized
    return normalized[: max_chars - 3].rstrip() + "..."


def should_skip_text(text: str) -> bool:
    return any(marker in text for marker in SKIP_TEXT_MARKERS) or bool(SELF_REFERENCE_REGEX.search(text))


def looks_like_other_project_reference(text: str) -> bool:
    """Drop adjacent-project comparison chatter unless it contains Gomoku-specific work."""
    return bool(OTHER_PROJECT_REGEX.search(text)) and not bool(GOMOKU_SPECIFIC_REGEX.search(text))


def should_keep_timestamp(
    timestamp: str | None,
    since_date: date | None,
    before_date: date | None,
) -> bool:
    day = timestamp_date(timestamp)
    if day is None:
        return True
    if since_date is not None and day < since_date:
        return False
    return before_date is None or day < before_date


def detect_topics(text: str) -> list[str]:
    if should_skip_text(text):
        return []
    topics = [
        topic
        for topic, regexes in TOPIC_REGEX.items()
        if any(regex.search(text) for regex in regexes)
    ]
    return topics


def release_band(timestamp: str | None, text: str) -> str:
    if re.search(r"\b0\.5\b", text):
        return "v0.5"
    if re.search(r"\b0\.4\b|corridor|rolling frontier|lethal onset", text, re.IGNORECASE):
        return "v0.4"
    if re.search(r"\b0\.3\b|Firebase|Firestore|cloud", text, re.IGNORECASE):
        return "v0.3"
    if re.search(r"\b0\.2\b|local[- ]first|profile|replay", text, re.IGNORECASE):
        return "v0.2"
    if not timestamp:
        return "unknown"
    try:
        day = datetime.fromisoformat(timestamp.replace("Z", "+00:00")).date()
    except ValueError:
        return "unknown"
    if day >= datetime.fromisoformat("2026-05-22").date():
        return "v0.5"
    if day >= datetime.fromisoformat("2026-05-01").date():
        return "v0.4"
    if day >= datetime.fromisoformat("2026-04-25").date():
        return "v0.3"
    return "v0.2"


def payload_text(payload: dict[str, Any]) -> str | None:
    payload_type = payload.get("type")
    if payload_type == "message":
        role = payload.get("role")
        if role in {"system", "developer"}:
            return None
        parts = []
        for item in payload.get("content", []):
            text = item.get("text") or item.get("input_text") or item.get("output_text")
            if text:
                parts.append(text)
        return "\n".join(parts) if parts else None

    if payload_type == "function_call":
        name = payload.get("name", "")
        if name != "exec_command":
            return None
        args = parse_json_object(payload.get("arguments"))
        workdir = str(args.get("workdir", ""))
        cmd = str(args.get("cmd", ""))
        if REPO_PATH not in workdir and REPO_PATH not in cmd and REPO_NAME not in cmd:
            return None
        if not SIGNIFICANT_TOOL_REGEX.search(cmd):
            return None
        return f"$ {cmd}"

    return None


def event_text(payload: dict[str, Any]) -> tuple[str | None, str]:
    event_type = payload.get("type", "")
    if event_type in {"agent_message", "subagent_notification", "user_message"}:
        return payload.get("message"), event_type
    return None, event_type


def parse_json_object(raw: Any) -> dict[str, Any]:
    if isinstance(raw, dict):
        return raw
    if not isinstance(raw, str):
        return {}
    try:
        decoded = json.loads(raw)
    except json.JSONDecodeError:
        return {}
    return decoded if isinstance(decoded, dict) else {}


def iter_session_files(root: Path) -> list[Path]:
    return sorted(root.rglob("rollout-*.jsonl"))


def source_label(path: Path, root: Path) -> str:
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)


def timestamp_date(timestamp: str | None) -> date | None:
    if not timestamp:
        return None
    try:
        return datetime.fromisoformat(timestamp.replace("Z", "+00:00")).date()
    except ValueError:
        return None


def output_dir_is_safe(output_dir: Path, repo_root: Path) -> bool:
    resolved = output_dir.resolve()
    repo = repo_root.resolve()
    try:
        relative = resolved.relative_to(repo)
    except ValueError:
        return True

    probe = relative / ".process-story-output-probe"
    result = subprocess.run(
        ["git", "check-ignore", "-q", "--no-index", str(probe)],
        cwd=repo,
        check=False,
    )
    return result.returncode == 0


def extract_sessions(
    sessions_root: Path,
    since_date: date | None,
    before_date: date | None,
    max_snippet_chars: int,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    events: list[dict[str, Any]] = []
    summaries: dict[str, SessionSummary] = {}

    for path in iter_session_files(sessions_root):
        label = source_label(path, sessions_root)
        summary = SessionSummary(source=label)
        summaries[label] = summary

        try:
            lines = path.open("r", encoding="utf-8")
        except OSError:
            continue

        with lines:
            pending_events: list[dict[str, Any]] = []
            has_project_session_marker = False

            for line_no, line in enumerate(lines, start=1):
                summary.line_count = line_no
                try:
                    record = json.loads(line)
                except json.JSONDecodeError:
                    continue

                timestamp = record.get("timestamp")
                if timestamp:
                    summary.first_timestamp = summary.first_timestamp or timestamp
                    summary.last_timestamp = timestamp

                first_day = timestamp_date(summary.first_timestamp)
                if first_day is not None:
                    if since_date is not None and first_day < since_date:
                        break
                    if before_date is not None and first_day >= before_date:
                        break

                if not should_keep_timestamp(timestamp, since_date, before_date):
                    continue

                record_type = record.get("type")
                payload = record.get("payload", {})
                if record_type == "session_meta":
                    summary.session_id = payload.get("id")
                    summary.session_cwd = payload.get("cwd")
                    if REPO_PATH in str(summary.session_cwd):
                        has_project_session_marker = True
                    continue

                text: str | None = None
                role = "unknown"
                event_kind = record_type or "unknown"
                reasons: list[str] = []

                if record_type == "response_item":
                    text = payload_text(payload)
                    event_kind = payload.get("type", event_kind)
                    role = payload.get("role") or payload.get("name") or "unknown"
                    if payload.get("type") == "function_call":
                        reasons.append("repo_tool_call")
                elif record_type == "event_msg":
                    text, event_kind = event_text(payload)
                    role = payload.get("type", "event")
                elif record_type == "turn_context":
                    cwd = str(payload.get("cwd", ""))
                    if REPO_PATH in cwd:
                        text = f"turn cwd: {cwd}"
                        event_kind = "turn_context"
                        role = "context"
                        reasons.append("repo_turn_context")

                if not text or should_skip_text(text) or looks_like_other_project_reference(text):
                    continue

                topics = detect_topics(text)
                has_project_reference = bool(PROJECT_REGEX.search(text))
                has_domain_reference = bool(DOMAIN_REGEX.search(text))
                if has_project_reference:
                    reasons.append("project_reference")
                    has_project_session_marker = True
                if has_domain_reference:
                    reasons.append("domain_reference")

                if event_kind == "function_call":
                    if not reasons:
                        continue
                elif not has_project_reference and not has_domain_reference:
                    continue

                snippet = truncate(text, max_snippet_chars)
                if not snippet:
                    continue

                pending_events.append(
                    {
                        "source": label,
                        "line": line_no,
                        "session_id": summary.session_id,
                        "timestamp": timestamp,
                        "release_band": release_band(timestamp, text),
                        "role": role,
                        "event_kind": event_kind,
                        "topics": topics,
                        "include_reasons": reasons or ["topic_match"],
                        "snippet": snippet,
                    }
                )

            if not has_project_session_marker:
                continue

            pending_events = [
                event
                for event in pending_events
                if event["event_kind"] != "turn_context" or len(pending_events) > 1
            ]
            for event in pending_events:
                summary.relevant_count += 1
                summary.role_counts[event["role"]] += 1
                summary.topic_counts.update(event["topics"] or ["project_reference"])
                summary.include_reasons.update(event["include_reasons"])
            events.extend(pending_events)

    session_index = [
        {
            "source": summary.source,
            "session_id": summary.session_id,
            "session_cwd": summary.session_cwd,
            "first_timestamp": summary.first_timestamp,
            "last_timestamp": summary.last_timestamp,
            "line_count": summary.line_count,
            "relevant_count": summary.relevant_count,
            "role_counts": dict(summary.role_counts),
            "topic_counts": dict(summary.topic_counts),
            "include_reasons": dict(summary.include_reasons),
        }
        for summary in summaries.values()
        if summary.relevant_count > 0
    ]
    session_index.sort(key=lambda item: (item.get("first_timestamp") or "", item["source"]))
    return events, session_index


def conversation_text(payload: dict[str, Any]) -> tuple[str | None, str | None, str]:
    payload_type = payload.get("type")
    if payload_type != "message":
        return None, None, payload_type or "unknown"

    role = payload.get("role")
    if role not in {"user", "assistant"}:
        return None, None, payload_type

    parts = []
    for item in payload.get("content", []):
        text = item.get("text") or item.get("input_text") or item.get("output_text")
        if text:
            parts.append(text)
    return ("\n".join(parts) if parts else None), role, payload_type


def extract_conversation_messages(
    sessions_root: Path,
    since_date: date | None,
    before_date: date | None,
) -> list[ConversationMessage]:
    messages: list[ConversationMessage] = []

    for path in iter_session_files(sessions_root):
        label = source_label(path, sessions_root)
        session_id: str | None = None
        session_first_timestamp: str | None = None
        has_project_session_marker = False
        has_gomoku_content = False
        session_messages: list[ConversationMessage] = []

        try:
            lines = path.open("r", encoding="utf-8")
        except OSError:
            continue

        with lines:
            for line_no, line in enumerate(lines, start=1):
                try:
                    record = json.loads(line)
                except json.JSONDecodeError:
                    continue

                timestamp = record.get("timestamp")
                if timestamp:
                    session_first_timestamp = session_first_timestamp or timestamp

                first_day = timestamp_date(session_first_timestamp)
                if first_day is not None:
                    if since_date is not None and first_day < since_date:
                        break
                    if before_date is not None and first_day >= before_date:
                        break

                if not should_keep_timestamp(timestamp, since_date, before_date):
                    continue

                record_type = record.get("type")
                payload = record.get("payload", {})

                if record_type == "session_meta":
                    session_id = payload.get("id")
                    if REPO_PATH in str(payload.get("cwd", "")):
                        has_project_session_marker = True
                    continue

                if record_type == "turn_context":
                    if REPO_PATH in str(payload.get("cwd", "")):
                        has_project_session_marker = True
                    continue

                text: str | None = None
                role: str | None = None
                event_kind = record_type or "unknown"

                if record_type == "response_item":
                    text, role, event_kind = conversation_text(payload)
                elif record_type == "event_msg":
                    text, event_kind = event_text(payload)
                    if event_kind == "user_message":
                        role = "user"
                    elif event_kind in {"agent_message", "subagent_notification"}:
                        role = "assistant"

                if not text or not role:
                    continue
                if should_skip_text(text) or looks_like_other_project_reference(text):
                    continue

                topics = detect_topics(text)
                has_project_reference = bool(PROJECT_REGEX.search(text))
                has_domain_reference = bool(DOMAIN_REGEX.search(text))
                reasons: list[str] = []
                if has_project_reference:
                    reasons.append("project_reference")
                    has_project_session_marker = True
                if has_domain_reference:
                    reasons.append("domain_reference")
                if topics:
                    reasons.append("topic_match")

                if reasons:
                    has_gomoku_content = True

                normalized = normalize_text(text)
                if not normalized:
                    continue

                session_messages.append(
                    ConversationMessage(
                        source=label,
                        line=line_no,
                        session_id=session_id,
                        timestamp=timestamp,
                        role=role,
                        event_kind=event_kind,
                        text=normalized,
                        topics=topics,
                        release_band=release_band(timestamp, text),
                        include_reasons=reasons or ["context_window"],
                    )
                )

        if has_project_session_marker or has_gomoku_content:
            messages.extend(session_messages)

    messages.sort(key=lambda item: (item.source, item.line))
    return messages


def has_gomoku_signal(message: ConversationMessage) -> bool:
    return any(
        reason in {"project_reference", "domain_reference", "topic_match"}
        for reason in message.include_reasons
    )


def message_is_seed(messages: list[ConversationMessage], index: int) -> bool:
    message = messages[index]
    if message.role != "user":
        return False
    if not HIGH_SIGNAL_USER_REGEX.search(message.text):
        return False

    if has_gomoku_signal(message):
        return True

    start = max(0, index - 2)
    end = min(len(messages), index + 3)
    return any(has_gomoku_signal(candidate) for candidate in messages[start:end])


def excerpt_dict(message: ConversationMessage, max_chars: int = MAX_ARC_EXCERPT_CHARS) -> dict[str, Any]:
    return {
        "source": message.source,
        "line": message.line,
        "timestamp": message.timestamp,
        "role": message.role,
        "event_kind": message.event_kind,
        "topics": message.topics,
        "text": truncate(message.text, max_chars),
    }


def first_user_text(messages: list[ConversationMessage]) -> str:
    for message in messages:
        if message.role == "user":
            return message.text
    return messages[0].text if messages else ""


def arc_title(messages: list[ConversationMessage], seed_messages: list[ConversationMessage]) -> str:
    text = first_user_text(seed_messages or messages)
    text = re.sub(r"^(okay|ok|so|next|also|btw|actually)[,\s]+", "", text, flags=re.IGNORECASE)
    return truncate(text.rstrip("?."), ARC_TITLE_CHARS)


def arc_summary(
    main_topic: str,
    messages: list[ConversationMessage],
    seed_messages: list[ConversationMessage],
) -> str:
    user_text = first_user_text(seed_messages or messages)
    return f"{topic_title(main_topic)} discussion centered on: {truncate(user_text, 180)}"


def select_key_items(
    messages: list[ConversationMessage],
    role: str,
    regex: re.Pattern[str] | None = None,
) -> list[dict[str, Any]]:
    selected: list[dict[str, Any]] = []
    seen: set[str] = set()
    for message in messages:
        if message.role != role:
            continue
        if regex is not None and not regex.search(message.text):
            continue
        excerpt = truncate(message.text, MAX_ARC_EXCERPT_CHARS)
        key = excerpt.lower()
        if key in seen:
            continue
        seen.add(key)
        selected.append(excerpt_dict(message))
        if len(selected) >= MAX_ARC_KEY_ITEMS:
            break
    return selected


def select_arc_excerpts(
    messages: list[ConversationMessage],
    seed_lines: set[int],
) -> list[dict[str, Any]]:
    selected: list[dict[str, Any]] = []
    seen: set[str] = set()

    for message in messages:
        is_seed = message.line in seed_lines
        is_assistant_signal = (
            message.role == "assistant"
            and (ASSISTANT_DECISION_REGEX.search(message.text) or ASSISTANT_OUTCOME_REGEX.search(message.text))
        )
        is_user_signal = message.role == "user" and (is_seed or HIGH_SIGNAL_USER_REGEX.search(message.text))
        is_topic_signal = bool(message.topics)
        if not (is_seed or is_assistant_signal or is_user_signal or is_topic_signal):
            continue

        excerpt = truncate(message.text, MAX_ARC_EXCERPT_CHARS)
        key = f"{message.role}:{excerpt}".lower()
        if key in seen:
            continue
        seen.add(key)
        selected.append(excerpt_dict(message))
        if len(selected) >= MAX_ARC_EXCERPTS:
            return selected

    if selected:
        return selected

    for message in messages[:MAX_ARC_EXCERPTS]:
        selected.append(excerpt_dict(message))
    return selected


def build_conversation_arcs(messages: list[ConversationMessage]) -> list[dict[str, Any]]:
    by_source: dict[str, list[ConversationMessage]] = defaultdict(list)
    for message in messages:
        by_source[message.source].append(message)

    arcs: list[dict[str, Any]] = []

    for source in sorted(by_source):
        source_messages = sorted(by_source[source], key=lambda item: item.line)
        seed_indices = [
            index
            for index in range(len(source_messages))
            if message_is_seed(source_messages, index)
        ]
        if not seed_indices:
            continue

        windows: list[tuple[int, int]] = []
        for index in seed_indices:
            start = max(0, index - ARC_WINDOW_BEFORE)
            end = min(len(source_messages) - 1, index + ARC_WINDOW_AFTER)
            if not windows:
                windows.append((start, end))
                continue

            last_start, last_end = windows[-1]
            if start <= last_end + 1 and end - last_start + 1 <= MAX_ARC_MESSAGES:
                windows[-1] = (last_start, max(last_end, end))
            else:
                windows.append((start, end))

        for start, end in windows:
            window_messages = source_messages[start : end + 1]
            seed_messages = [
                message
                for index, message in enumerate(source_messages[start : end + 1], start=start)
                if message_is_seed(source_messages, index)
            ]
            topic_counts = Counter()
            release_counts = Counter()
            for message in window_messages:
                topic_counts.update(message.topics or [])
                release_counts[message.release_band] += 1
            main_topic = topic_counts.most_common(1)[0][0] if topic_counts else "project_reference"
            main_release = release_counts.most_common(1)[0][0] if release_counts else "unknown"
            seed_lines = {message.line for message in seed_messages}

            arcs.append(
                {
                    "arc_id": "",
                    "title": arc_title(window_messages, seed_messages),
                    "summary": arc_summary(main_topic, window_messages, seed_messages),
                    "source": source,
                    "source_start_line": window_messages[0].line,
                    "source_end_line": window_messages[-1].line,
                    "session_id": window_messages[0].session_id,
                    "start_timestamp": window_messages[0].timestamp,
                    "end_timestamp": window_messages[-1].timestamp,
                    "release_band": main_release,
                    "topics": [topic for topic, _count in topic_counts.most_common()],
                    "key_user_asks": select_key_items(seed_messages or window_messages, "user"),
                    "key_decisions": select_key_items(window_messages, "assistant", ASSISTANT_DECISION_REGEX),
                    "outcome_hints": select_key_items(window_messages, "assistant", ASSISTANT_OUTCOME_REGEX),
                    "excerpts": select_arc_excerpts(window_messages, seed_lines),
                }
            )

    arcs.sort(
        key=lambda arc: (
            arc.get("start_timestamp") or "",
            arc["source"],
            arc["source_start_line"],
        )
    )
    for index, arc in enumerate(arcs, start=1):
        arc["arc_id"] = f"arc_{index:04d}"
    return arcs


def git_chronology(repo_root: Path) -> list[dict[str, str]]:
    try:
        output = subprocess.check_output(
            [
                "git",
                "log",
                "--reverse",
                "--format=%H%x09%h%x09%ad%x09%s",
                "--date=short",
                "--since=2026-04-01",
            ],
            cwd=repo_root,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return []

    commits = []
    for line in output.splitlines():
        full, short, date, subject = line.split("\t", 3)
        commits.append({"hash": full, "short": short, "date": date, "subject": subject})
    return commits


def topic_title(topic: str) -> str:
    return topic.replace("_", " ").title()


def write_json(path: Path, data: Any) -> None:
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_jsonl(path: Path, rows: list[dict[str, Any]]) -> None:
    with path.open("w", encoding="utf-8") as fh:
        for row in rows:
            fh.write(json.dumps(row, sort_keys=True) + "\n")


def write_quote_candidates(
    path: Path,
    events: list[dict[str, Any]],
    sessions_root: Path,
    max_per_topic: int,
) -> None:
    by_topic: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for event in events:
        if event["event_kind"] not in {"message", "user_message", "agent_message", "subagent_notification"}:
            continue
        if event["role"] not in {"user", "assistant", "agent_message", "subagent_notification"}:
            continue
        for topic in event["topics"] or ["project_reference"]:
            by_topic[topic].append(event)

    lines = [
        "# Process Story Quote Candidates",
        "",
        "Private review material. Keep quotes short and rewrite/paraphrase before public use.",
        "",
    ]
    for topic in sorted(by_topic):
        lines.extend([f"## {topic_title(topic)}", ""])
        seen: set[str] = set()
        emitted = 0
        for event in by_topic[topic]:
            key = event["snippet"].lower()
            if key in seen:
                continue
            seen.add(key)
            provenance = f"{sessions_root / event['source']}:{event['line']}"
            lines.append(f"- `{event['timestamp']}` `{event['role']}` {provenance}")
            lines.append(f"  > {event['snippet']}")
            emitted += 1
            if emitted >= max_per_topic:
                break
        if emitted == 0:
            lines.append("- No quote candidates.")
        lines.append("")
    path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def markdown_quote(text: str) -> list[str]:
    quoted = []
    for line in text.splitlines() or [text]:
        quoted.append(f"> {line}")
    return quoted


def chunked(rows: list[dict[str, Any]], size: int) -> list[list[dict[str, Any]]]:
    return [rows[index : index + size] for index in range(0, len(rows), size)]


def arc_markdown_lines(arc: dict[str, Any], sessions_root: Path) -> list[str]:
    topics = ", ".join(arc["topics"]) if arc["topics"] else "project_reference"
    provenance = (
        f"{sessions_root / arc['source']}:{arc['source_start_line']}"
        f"-{arc['source_end_line']}"
    )
    lines = [
        f"## {arc['arc_id']} - {arc['title']}",
        "",
        f"- Time: `{arc['start_timestamp']}` to `{arc['end_timestamp']}`",
        f"- Source: `{provenance}`",
        f"- Release band: `{arc['release_band']}`",
        f"- Topics: `{topics}`",
        "",
        arc["summary"],
        "",
    ]

    if arc["key_user_asks"]:
        lines.extend(["### Key User Asks", ""])
        for item in arc["key_user_asks"]:
            lines.append(f"- `{item['timestamp']}` `{item['source']}:{item['line']}`")
            lines.extend(markdown_quote(item["text"]))
        lines.append("")

    if arc["key_decisions"]:
        lines.extend(["### Key Decisions / Rationale", ""])
        for item in arc["key_decisions"]:
            lines.append(f"- `{item['timestamp']}` `{item['source']}:{item['line']}`")
            lines.extend(markdown_quote(item["text"]))
        lines.append("")

    if arc["outcome_hints"]:
        lines.extend(["### Outcome Hints", ""])
        for item in arc["outcome_hints"]:
            lines.append(f"- `{item['timestamp']}` `{item['source']}:{item['line']}`")
            lines.extend(markdown_quote(item["text"]))
        lines.append("")

    lines.extend(["### Short Excerpts", ""])
    for item in arc["excerpts"]:
        lines.append(
            f"- `{item['timestamp']}` `{item['role']}` "
            f"`{item['source']}:{item['line']}`"
        )
        lines.extend(markdown_quote(item["text"]))
    lines.append("")
    return lines


def write_conversation_arcs_markdown(path: Path, arcs: list[dict[str, Any]], sessions_root: Path) -> None:
    chunk_dir = path.with_suffix("")
    chunk_dir.mkdir(parents=True, exist_ok=True)
    for old_chunk in chunk_dir.glob("arcs_*.md"):
        old_chunk.unlink()

    chunks = chunked(arcs, CONVERSATION_ARCS_PER_CHUNK)
    chunk_rows = []
    for chunk in chunks:
        first = chunk[0]
        last = chunk[-1]
        filename = f"arcs_{first['arc_id'][-4:]}-{last['arc_id'][-4:]}.md"
        chunk_rows.append((filename, first, last, len(chunk)))
        lines = [
            f"# Conversation Arcs {first['arc_id']} - {last['arc_id']}",
            "",
            "[Back to conversation arc index](../conversation_arcs.md)",
            "",
            "Private review material. These are bounded summaries and short excerpts",
            "from raw Codex sessions, intended to recover narrative flow without",
            "publishing full transcripts.",
            "",
        ]
        for arc in chunk:
            lines.extend(arc_markdown_lines(arc, sessions_root))
        (chunk_dir / filename).write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")

    lines = [
        "# Process Story Conversation Arcs",
        "",
        "Private review material. This index points to smaller markdown chunks so",
        "the full conversation review does not have to open as one large file.",
        "",
        f"- Total arcs: `{len(arcs)}`",
        f"- Arcs per chunk: `{CONVERSATION_ARCS_PER_CHUNK}`",
        f"- Chunk directory: `{chunk_dir.name}/`",
        "",
        "Use `conversation_arcs.jsonl` for machine processing and the chunk files",
        "below for manual narrative review.",
        "",
        "## Chunks",
        "",
    ]

    if not arcs:
        lines.append("- No conversation arcs found.")
    else:
        for filename, first, last, count in chunk_rows:
            lines.append(
                f"- [{first['arc_id']} - {last['arc_id']}]"
                f"({chunk_dir.name}/{filename}) - `{count}` arcs, "
                f"starts `{first['start_timestamp']}` to `{last['start_timestamp']}`"
            )

    path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def write_outline(
    path: Path,
    events: list[dict[str, Any]],
    session_index: list[dict[str, Any]],
    commits: list[dict[str, str]],
    conversation_arcs: list[dict[str, Any]],
) -> None:
    topic_counts = Counter()
    release_counts = Counter()
    for event in events:
        topic_counts.update(event["topics"] or ["project_reference"])
        release_counts[event["release_band"]] += 1

    release_commits = [
        commit
        for commit in commits
        if commit["subject"].startswith("release:") or "release" in commit["subject"].lower()
    ]

    lines = [
        "# Process Story First-Pass Outline",
        "",
        "Generated private review artifact. Use this to decide what becomes public",
        "devlog or README copy; do not publish raw quotes without another review.",
        "",
        "## Extraction Summary",
        "",
        f"- Included sessions: `{len(session_index)}`",
        f"- Evidence events: `{len(events)}`",
        f"- Conversation arcs: `{len(conversation_arcs)}`",
        f"- Git commits scanned: `{len(commits)}`",
        "",
        "### Evidence By Release Band",
        "",
    ]
    for band, count in sorted(release_counts.items()):
        lines.append(f"- `{band}`: `{count}` events")

    lines.extend(["", "### Evidence By Topic", ""])
    for topic, count in topic_counts.most_common():
        lines.append(f"- `{topic}`: `{count}` events")

    lines.extend(
        [
            "",
            "## Narrative Spine",
            "",
            "1. Revival: an old Gomoku project becomes a modern Rust/Wasm/browser product.",
            "2. Product foundation: local-first play, profiles, replay, cloud continuity, and mobile polish make it a real app.",
            "3. Lab turn: bot tuning pushes the project toward tournaments, reports, and measurable evidence instead of vibes.",
            "4. Strategic pivot: corridor search becomes more valuable as replay-analysis vocabulary than as a raw strength shortcut.",
            "5. Hard lessons: corridor portals fail promotion, rolling frontier pays off, and Renju legality requires recursive proof.",
            "6. Productization: replay analysis, reports, rules, guide, visuals, and release flow turn lab work into public surfaces.",
            "7. Process thesis: one developer supplies taste and judgment; agents supply exploration, implementation, review, and evidence throughput.",
            "",
            "## Release Checkpoints",
            "",
        ]
    )
    if release_commits:
        for commit in release_commits:
            lines.append(f"- `{commit['date']}` `{commit['short']}` {commit['subject']}")
    else:
        lines.append("- No release commits found.")

    lines.extend(
        [
            "",
            "## Follow-Up Review Questions",
            "",
            "- Which moments are worth showing as public devlog evidence rather than private process notes?",
            "- Which failures should be named explicitly, and which should stay as internal engineering lessons?",
            "- Does the story lead with the game first and keep the AI process as production context?",
            "- Which screenshots/report excerpts should accompany the story?",
        ]
    )
    path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def write_readme(path: Path, sessions_root: Path, repo_root: Path) -> None:
    lines = [
        "# Process Story Extraction Outputs",
        "",
        "Ignored private review artifacts generated from local Codex sessions.",
        "",
        "Regenerate from repo root:",
        "",
        "```sh",
        "python3 scripts/extract_process_story.py --before-date YYYY-MM-DD",
        "```",
        "",
        "Use `--before-date` to keep the current mining session from ingesting itself.",
        "",
        "Inputs:",
        "",
        f"- sessions: `{sessions_root}`",
        f"- repo: `{repo_root}`",
        "",
        "Outputs:",
        "",
        "- `session_index.json` - candidate session files and inclusion reasons.",
        "- `evidence_events.jsonl` - normalized evidence snippets with provenance.",
        "- `conversation_arcs.jsonl` - bounded narrative arcs around high-signal turns.",
        "- `conversation_arcs.md` - small index for split conversation-arc chunks.",
        "- `conversation_arcs/` - readable summaries and short excerpts for arc review.",
        "- `git_chronology.json` - commit chronology for cross-checking milestones.",
        "- `quote_candidates.md` - short private-review quote candidates.",
        "- `process_outline.md` - generated outline and evidence counts.",
        "",
        "Do not publish raw output directly. Treat it as source material for",
        "curated docs, devlogs, or release copy.",
    ]
    path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    repo_root = REPO_ROOT
    parser.add_argument(
        "--sessions-root",
        type=Path,
        default=Path.home() / ".codex" / "sessions" / "2026",
        help="Root containing Codex rollout JSONL files.",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=repo_root / "gomoku-bot-lab" / "outputs" / "process-story",
        help="Ignored output directory for private review artifacts.",
    )
    parser.add_argument(
        "--since-date",
        default="2026-04-14",
        help="Only scan sessions whose first timestamp is on or after this date.",
    )
    parser.add_argument(
        "--before-date",
        default=None,
        help="Only scan sessions whose first timestamp is before this date.",
    )
    parser.add_argument(
        "--max-snippet-chars",
        type=int,
        default=240,
        help="Maximum snippet length stored in evidence outputs.",
    )
    parser.add_argument(
        "--max-quotes-per-topic",
        type=int,
        default=8,
        help="Maximum quote candidates emitted for each topic.",
    )
    args = parser.parse_args()

    if not args.sessions_root.exists():
        raise SystemExit(f"sessions root does not exist: {args.sessions_root}")

    if not output_dir_is_safe(args.output_dir, repo_root):
        raise SystemExit(
            "refusing to write raw process-story extracts into a trackable repo path. "
            "Use an ignored output directory, for example "
            f"{repo_root / 'gomoku-bot-lab' / 'outputs' / 'process-story'}; got {args.output_dir}"
        )

    args.output_dir.mkdir(parents=True, exist_ok=True)

    try:
        since_date = datetime.fromisoformat(args.since_date).date() if args.since_date else None
    except ValueError as error:
        raise SystemExit(f"invalid --since-date: {args.since_date}") from error
    try:
        before_date = datetime.fromisoformat(args.before_date).date() if args.before_date else None
    except ValueError as error:
        raise SystemExit(f"invalid --before-date: {args.before_date}") from error

    events, session_index = extract_sessions(
        args.sessions_root,
        since_date,
        before_date,
        args.max_snippet_chars,
    )
    conversation_messages = extract_conversation_messages(
        args.sessions_root,
        since_date,
        before_date,
    )
    conversation_arcs = build_conversation_arcs(conversation_messages)
    commits = git_chronology(repo_root)

    write_json(args.output_dir / "session_index.json", session_index)
    write_jsonl(args.output_dir / "evidence_events.jsonl", events)
    write_jsonl(args.output_dir / "conversation_arcs.jsonl", conversation_arcs)
    write_json(args.output_dir / "git_chronology.json", commits)
    write_conversation_arcs_markdown(
        args.output_dir / "conversation_arcs.md",
        conversation_arcs,
        args.sessions_root,
    )
    write_quote_candidates(
        args.output_dir / "quote_candidates.md",
        events,
        args.sessions_root,
        args.max_quotes_per_topic,
    )
    write_outline(
        args.output_dir / "process_outline.md",
        events,
        session_index,
        commits,
        conversation_arcs,
    )
    write_readme(args.output_dir / "README.md", args.sessions_root, repo_root)

    print(f"Wrote process-story extraction to {args.output_dir}")
    print(f"Included sessions: {len(session_index)}")
    print(f"Evidence events: {len(events)}")
    print(f"Conversation arcs: {len(conversation_arcs)}")
    print(f"Git commits: {len(commits)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
