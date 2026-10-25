"""Build an exact tagged source in a new owned worktree and record its inputs.

This operator tool never starts services, changes tracked source, reuses a tag's
node_modules, publishes, or removes an existing worktree. Run from a clean main
checkout; authoring changes there are allowed and are not build inputs.
"""
from pathlib import Path
import argparse
import datetime
import hashlib
import json
import os
import subprocess

REPO = Path(__file__).resolve().parents[2]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def output(args, cwd=None):
    return subprocess.check_output(["rtk", "proxy", *args], cwd=cwd, text=True).strip()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--owner-root", type=Path, required=True)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--cloud-mode", choices=["none", "legacy-proxy", "modern-emulators"], default="none")
    args = parser.parse_args()
    owner = args.owner_root.resolve(strict=True)
    marker = json.loads((owner / "owner.json").read_text())
    if marker.get("owner") != "gomoku-history-implementation" or marker.get("repo") != str(REPO) or marker.get("root") != str(owner):
        raise RuntimeError("Owner marker does not match this repository/workflow")
    tags = output(["git", "tag", "--list", "v0.[2-6].*"], REPO).splitlines()
    if args.tag not in tags:
        raise RuntimeError("Unknown tagged history source")
    commit = output(["git", "rev-parse", args.tag + "^{commit}"], REPO)
    worktree = owner / "sources" / args.tag.replace(".", "_")
    evidence = owner / "builds" / args.tag
    if worktree.exists() or evidence.exists():
        raise RuntimeError("Build worktree/evidence must be new; preserve previous failures")
    worktree.parent.mkdir(exist_ok=True)
    evidence.mkdir(parents=True)
    subprocess.run(["rtk", "proxy", "git", "worktree", "add", "--detach", str(worktree), commit], cwd=REPO, check=True)
    log_path = evidence / "build.log"
    env = {k: v for k, v in os.environ.items() if not k.startswith("VITE_") and k != "GOMOKU_BASE_PATH"}
    env["CARGO_TARGET_DIR"] = str(owner / "cargo-target")
    env["BROWSER"] = "none"
    env["GOMOKU_BASE_PATH"] = "/"
    if args.cloud_mode != "none":
        env.update({
            "VITE_FIREBASE_API_KEY": "history-key", "VITE_FIREBASE_APP_ID": "1:123:web:history",
            "VITE_FIREBASE_AUTH_DOMAIN": "127.0.0.1", "VITE_FIREBASE_MESSAGING_SENDER_ID": "123",
            "VITE_FIREBASE_PROJECT_ID": "demo-gomoku-history", "VITE_FIREBASE_STORAGE_BUCKET": "unused",
        })
    if args.cloud_mode == "modern-emulators":
        env.update({"VITE_FIREBASE_EMULATORS": "1", "VITE_FIREBASE_AUTH_EMULATOR_PORT": "29099", "VITE_FIREBASE_FIRESTORE_EMULATOR_PORT": "28081"})
    wasm_pack = Path("/home/bryan/.cargo/bin/wasm-pack")
    lock = worktree / "gomoku-bot-lab/Cargo.lock"
    tracked_lock = lock.is_file()
    commands = [
        ([str(wasm_pack), "build", str(worktree / "gomoku-bot-lab/gomoku-wasm"), "--target", "bundler", *(["--", "--locked"] if tracked_lock else [])], worktree),
        (["npm", "ci", "--no-audit", "--no-fund"], worktree / "gomoku-web"),
        (["npm", "run", "build"], worktree / "gomoku-web"),
    ]
    with log_path.open("w") as log:
        for command, cwd in commands:
            log.write(json.dumps({"command": command, "cwd": str(cwd)}) + "\n")
            log.flush()
            subprocess.run(["rtk", "proxy", *command], cwd=cwd, env=env, stdout=log, stderr=subprocess.STDOUT, check=True)
    cargo_lock_hash = sha(lock)
    if not tracked_lock:
        # Retain the actual resolver output, then restore the untracked-input
        # state. No tracked lock or other source file is removed.
        (evidence / "resolved-Cargo.lock").write_bytes(lock.read_bytes())
        lock.unlink()
    status = output(["git", "status", "--porcelain", "--untracked-files=all"], worktree)
    if status:
        raise RuntimeError("Tagged source changed during build: " + status)
    dist = worktree / "gomoku-web/dist"
    files = [{"path": p.relative_to(dist).as_posix(), "sha256": sha(p)} for p in sorted(dist.rglob("*")) if p.is_file()]
    report_inputs = []
    for prefix in ["gomoku-bot-lab/reports", "gomoku-bot-lab/analysis-reports", "reports/lab"]:
        for relative in output(["git", "ls-tree", "-r", "--name-only", "HEAD", "--", prefix], worktree).splitlines():
            path = worktree / relative
            if path.suffix in [".html", ".json"]:
                report_inputs.append({"path": relative, "sha256": sha(path)})
    receipt = {
        "schemaVersion": 1, "recordedAt": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "tag": args.tag, "sourceCommit": commit, "sourceTree": output(["git", "rev-parse", "HEAD^{tree}"], worktree),
        "applicationTree": output(["git", "rev-parse", "HEAD:gomoku-web/src"], worktree),
        "packageVersion": json.loads((worktree / "gomoku-web/package.json").read_text())["version"],
        "packageLockSha256": sha(worktree / "gomoku-web/package-lock.json"),
        "cargoLock": {"kind": "tracked" if tracked_lock else "generated-external", "sha256": cargo_lock_hash},
        "wasmSha256": sha(worktree / "gomoku-bot-lab/gomoku-wasm/pkg/gomoku_wasm_bg.wasm"),
        "distSha256": hashlib.sha256((json.dumps(files, separators=(",", ":"), ensure_ascii=False) + "\n").encode()).hexdigest(),
        "indexSha256": sha(dist / "index.html"), "files": files, "reportInputs": report_inputs,
        "environment": {
            "node": output(["node", "--version"]), "npm": output(["npm", "--version"]),
            "rustc": output(["rustc", "--version"]), "cargo": output(["cargo", "--version"]),
            "wasmPack": output([str(wasm_pack), "--version"]),
            "firebaseMode": args.cloud_mode,
            "firebaseModeDescription": {"none": "unconfigured-local-guest", "legacy-proxy": "synthetic-demo-loopback-proxy", "modern-emulators": "synthetic-demo-native-emulators"}[args.cloud_mode],
            "firebaseEnvironment": {k: v for k, v in env.items() if k.startswith("VITE_FIREBASE_")},
            "wasmBuild": "wasm-pack release --target bundler" + (" -- --locked" if tracked_lock else "; generated Cargo resolution recorded separately"),
            "npmInstall": "npm ci --no-audit --no-fund; host npm default install-script policy",
        },
        "commands": [{"args": command, "cwd": str(cwd)} for command, cwd in commands],
        "buildLogSha256": sha(log_path), "applicationInputsClean": True,
    }
    (evidence / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({"status": "built", "tag": args.tag, "worktree": str(worktree), "receipt": str(evidence / "receipt.json"), "files": len(files), "distSha256": receipt["distSha256"]}))

if __name__ == "__main__":
    main()
