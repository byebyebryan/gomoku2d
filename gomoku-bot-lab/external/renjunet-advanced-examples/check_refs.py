#!/usr/bin/env python3
import json
import shutil
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
LAB = ROOT.parents[1]
FIXTURES = ROOT / "fixtures.json"
BUILD = LAB / "outputs" / "renjunet-advanced-examples-ref-check-build"

PISKVORK_REPO_URL = "https://github.com/wind23/piskvork_renju.git"
PISKVORK_COMMIT = "f76a43afb67861883c86f8bd22b1a4957c27f068"

REASON_NAMES = {
    0: "legal",
    1: "double-three",
    2: "double-four",
    3: "overline",
}


def run(cmd, cwd=None, check=True):
    result = subprocess.run(
        cmd,
        cwd=cwd,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if check and result.returncode != 0:
        raise RuntimeError(
            f"command failed ({result.returncode}): {' '.join(map(str, cmd))}\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )
    return result


def expected_forbidden(case):
    return not case["expected"].lower().startswith("not forbidden")


def cpp_string(value):
    return json.dumps(value)


def cpp_vec(values):
    return "{" + ", ".join(cpp_string(value) for value in values) + "}"


def write_cpp_cases(cases):
    rows = []
    for case in cases:
        rows.append(
            f"    {{{cpp_string(case['id'])}, {cpp_string(case['probe'])}, "
            f"{str(expected_forbidden(case)).lower()}, "
            f"{cpp_vec(case['black'])}, {cpp_vec(case['white'])}}},"
        )
    return "\n".join(rows)


def checker_cpp(cases):
    checker_body = """
    int board[N][N] = {};
    for (const auto& s : tc.black) { board[col(s)][row(s)] = 1; }
    for (const auto& s : tc.white) { board[col(s)][row(s)] = -1; }
    board[col(tc.probe)][row(tc.probe)] = 1;
    int reason = forbid(board, col(tc.probe) + row(tc.probe) * 15, 15);
"""
    piskvork_extern = "extern int forbid(int board_[N][N], int pos, int size);\n"
    return f"""#include <iostream>
#include <string>
#include <vector>
#include "game.h"
{piskvork_extern}
struct Case {{
  const char* id;
  const char* probe;
  bool expected_forbidden;
  std::vector<const char*> black;
  std::vector<const char*> white;
}};
static int col(const std::string& s) {{ return s[0] - 'A'; }}
static int row(const std::string& s) {{ return std::stoi(s.substr(1)) - 1; }}
int main() {{
  std::vector<Case> cases = {{
{write_cpp_cases(cases)}
  }};
  std::cout << "[";
  for (size_t i = 0; i < cases.size(); ++i) {{
    const auto& tc = cases[i];
{checker_body}
    bool actual_forbidden = reason != 0;
    bool passed = actual_forbidden == tc.expected_forbidden;
    if (i) std::cout << ",";
    std::cout << "\\n  {{\\\"id\\\":\\\"" << tc.id
      << "\\\",\\\"probe\\\":\\\"" << tc.probe
      << "\\\",\\\"expected_forbidden\\\":" << (tc.expected_forbidden ? "true" : "false")
      << ",\\\"actual_forbidden\\\":" << (actual_forbidden ? "true" : "false")
      << ",\\\"reason\\\":" << reason
      << ",\\\"passed\\\":" << (passed ? "true" : "false")
      << "}}";
  }}
  std::cout << "\\n]\\n";
}}
"""


def ensure_piskvork_src():
    checkout = BUILD / "external" / "piskvork_renju"
    if not (checkout / ".git").exists():
        if checkout.exists():
            shutil.rmtree(checkout)
        checkout.parent.mkdir(parents=True, exist_ok=True)
        run(
            [
                "git",
                "clone",
                "--quiet",
                "--filter=blob:none",
                PISKVORK_REPO_URL,
                str(checkout),
            ]
        )
    else:
        run(["git", "fetch", "--quiet", "origin"], cwd=checkout)

    run(["git", "checkout", "--quiet", PISKVORK_COMMIT], cwd=checkout)
    src = checkout / "renju"
    if not src.exists():
        raise RuntimeError(f"Piskvork source directory not found: {src}")
    return src


def build_piskvork(cases):
    piskvork_src = ensure_piskvork_src()
    out_dir = BUILD / "piskvork-check"
    out_dir.mkdir(parents=True, exist_ok=True)
    src = out_dir / "check.cpp"
    src.write_text(checker_cpp(cases), encoding="utf-8")
    binary = out_dir / "check"
    run(
        [
            "g++",
            "-std=c++17",
            "-O2",
            f"-I{piskvork_src}",
            str(src),
            str(piskvork_src / "Class_line.cpp"),
            str(piskvork_src / "Class_line4v.cpp"),
            str(piskvork_src / "global.cpp"),
            "-o",
            str(binary),
        ]
    )
    result = run([str(binary)], check=False)
    output = json.loads(result.stdout)
    (ROOT / "piskvork_check.json").write_text(json.dumps(output, indent=2), encoding="utf-8")
    return output


def by_id(rows):
    return {row["id"]: row for row in rows or []}


def outcome(row):
    if row is None:
        return "n/a"
    label = "forbidden" if row["actual_forbidden"] else "legal"
    reason = row.get("reason")
    if reason is None:
        return label
    return f"{label} ({REASON_NAMES.get(reason, f'reason {reason}')})"


def write_summary(cases, piskvork):
    pis = by_id(piskvork)
    md = [
        "# RenjuNet Advanced Reference Check",
        "",
        "Fixture cases are generated from `fixtures.json`. Expected labels are the current manual labels.",
        "",
        "Checkers:",
        f"- `piskvork`: C++ Renju foul checker cloned from `{PISKVORK_REPO_URL}` at `{PISKVORK_COMMIT}`.",
        "",
        "| Fixture | Probe | Expected | Piskvork | Ref verdict |",
        "| --- | --- | --- | --- | --- |",
    ]
    ref_matches_expected = 0
    ref_disagrees_expected = 0
    for case in cases:
        exp = expected_forbidden(case)
        ref_row = pis.get(case["id"])
        if ref_row is None:
            verdict = "ref unavailable"
        elif ref_row["actual_forbidden"] == exp:
            verdict = "ref matches expected"
            ref_matches_expected += 1
        else:
            verdict = "ref disagrees expected"
            ref_disagrees_expected += 1
        expected_label = "forbidden" if exp else "legal"
        md.append(
            f"| `{case['id']}` | `{case['probe']}` | {expected_label} | "
            f"{outcome(ref_row)} | {verdict} |"
        )
    total = len(cases)
    p_pass = sum(1 for row in piskvork or [] if row["passed"])
    md.extend(
        [
            "",
            "## Summary",
            "",
            f"- Fixtures: {total}",
            f"- Piskvork matches expected: {p_pass}/{total}",
            f"- External reference matches expected: {ref_matches_expected}/{total}",
            f"- External reference disagrees with expected: {ref_disagrees_expected}/{total}",
            "",
            "Piskvork is GPL-licensed external executable evidence. The wrapper compiles it outside the repo and does not copy reference code into Gomoku2D.",
        ]
    )
    (ROOT / "validation.md").write_text("\n".join(md) + "\n", encoding="utf-8")


def main():
    BUILD.mkdir(parents=True, exist_ok=True)
    cases = json.loads(FIXTURES.read_text(encoding="utf-8"))
    piskvork = build_piskvork(cases)
    write_summary(cases, piskvork)
    print(ROOT / "validation.md")


if __name__ == "__main__":
    main()
