#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import re
import shlex
import subprocess
import sys
import time
from pathlib import Path
from typing import Any, Sequence

SEMVER_PATTERN = re.compile(
    r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)"
    r"(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?"
    r"(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$"
)

GITHUB_REMOTE_PATTERNS = (
    re.compile(r"^https://github\.com/(?P<owner>[^/]+)/(?P<repo>[^/.]+?)(?:\.git)?/?$"),
    re.compile(r"^git@github\.com:(?P<owner>[^/]+)/(?P<repo>[^/.]+?)(?:\.git)?$"),
    re.compile(r"^ssh://git@github\.com/(?P<owner>[^/]+)/(?P<repo>[^/.]+?)(?:\.git)?/?$"),
)

VERSION_FILES = (
    "package.json",
    "package-lock.json",
    "src-tauri/tauri.conf.json",
    "Cargo.toml",
)

RELEASE_WORKFLOW_PATH = ".github/workflows/release.yml"


class ReleaseError(RuntimeError):
    pass


def repo_root() -> Path:
    return Path(__file__).resolve().parents[3]


def format_command(command: Sequence[str]) -> str:
    return " ".join(shlex.quote(part) for part in command)


def run_command(
    command: Sequence[str],
    *,
    cwd: Path,
    capture_output: bool = False,
) -> str:
    print(f"$ {format_command(command)}", flush=True)
    completed = subprocess.run(
        list(command),
        cwd=cwd,
        text=True,
        capture_output=capture_output,
        check=False,
    )

    if completed.returncode != 0:
        if capture_output:
            if completed.stdout:
                sys.stdout.write(completed.stdout)
            if completed.stderr:
                sys.stderr.write(completed.stderr)
        raise ReleaseError(f"Command failed ({completed.returncode}): {format_command(command)}")

    return completed.stdout if capture_output else ""


def load_project_version(root: Path) -> str:
    package_json = json.loads((root / "package.json").read_text(encoding="utf-8"))
    version = package_json.get("version")
    if not isinstance(version, str) or not SEMVER_PATTERN.fullmatch(version):
        raise ReleaseError("package.json does not contain a valid semver version")
    return version


def next_patch_version(version: str) -> str:
    match = SEMVER_PATTERN.fullmatch(version)
    if not match:
        raise ReleaseError(f"Invalid semver version: {version}")

    major, minor, patch, prerelease, build = match.groups()
    if prerelease or build:
        raise ReleaseError(
            "Automatic patch release requires a stable current version. Pass --version explicitly.",
        )

    return f"{major}.{minor}.{int(patch) + 1}"


def parse_github_repository(remote_url: str) -> tuple[str, str]:
    for pattern in GITHUB_REMOTE_PATTERNS:
        match = pattern.fullmatch(remote_url)
        if match:
            return match.group("owner"), match.group("repo")

    raise ReleaseError(f"Remote is not a supported GitHub repository URL: {remote_url}")


def ensure_clean_worktree(root: Path) -> None:
    status = run_command(["git", "status", "--short"], cwd=root, capture_output=True).strip()
    if status:
        raise ReleaseError("Git working tree is not clean. Commit or stash changes before releasing.")


def ensure_branch(root: Path, branch: str) -> None:
    current_branch = run_command(["git", "branch", "--show-current"], cwd=root, capture_output=True).strip()
    if current_branch != branch:
        raise ReleaseError(f"Release must run from {branch}. Current branch is {current_branch}.")


def release_run_matches(run: dict[str, Any], tag_name: str) -> bool:
    return (
        run.get("name") == "Release"
        and run.get("path") == RELEASE_WORKFLOW_PATH
        and run.get("event") == "push"
        and run.get("head_branch") == tag_name
    )


def gh_api_json(root: Path, path: str, *, method: str = "GET", fields: Sequence[str] | None = None) -> Any:
    command = ["gh", "api", path]
    if method != "GET":
        command.extend(["--method", method])
    for field in fields or ():
        command.extend(["-F", field])

    output = run_command(command, cwd=root, capture_output=True)
    return json.loads(output)


def wait_for_release_run(
    root: Path,
    owner: str,
    repo: str,
    tag_name: str,
    *,
    timeout_seconds: int,
    poll_interval_seconds: int,
) -> dict[str, Any]:
    deadline = time.time() + timeout_seconds
    last_state: tuple[str | None, str | None] | None = None

    while time.time() < deadline:
        payload = gh_api_json(root, f"repos/{owner}/{repo}/actions/runs?per_page=100")
        workflow_runs = payload.get("workflow_runs", [])
        run = next((item for item in workflow_runs if release_run_matches(item, tag_name)), None)

        if run is None:
            print(f"Waiting for release workflow run for {tag_name}...", flush=True)
            time.sleep(poll_interval_seconds)
            continue

        state = (run.get("status"), run.get("conclusion"))
        if state != last_state:
            print(
                f"Release workflow run {run.get('id')} status={run.get('status')} conclusion={run.get('conclusion')}",
                flush=True,
            )
            last_state = state

        if run.get("status") == "completed":
            if run.get("conclusion") != "success":
                raise ReleaseError(
                    f"Release workflow failed for {tag_name}: {run.get('html_url')}",
                )
            return run

        time.sleep(poll_interval_seconds)

    raise ReleaseError(f"Timed out waiting for release workflow run for {tag_name}")


def validate_latest_manifest(manifest_text: str, version: str, tag_name: str) -> dict[str, Any]:
    try:
        manifest = json.loads(manifest_text)
    except json.JSONDecodeError as error:
        raise ReleaseError("latest.json is not valid JSON") from error

    if manifest.get("version") != version:
        raise ReleaseError(
            f"latest.json version mismatch. Expected {version}, found {manifest.get('version')}",
        )

    platforms = manifest.get("platforms")
    if not isinstance(platforms, dict) or not platforms:
        raise ReleaseError("latest.json does not contain any platforms")

    expected_segment = f"/releases/download/{tag_name}/"
    for platform_name, platform_info in platforms.items():
        if not isinstance(platform_info, dict):
            raise ReleaseError(f"latest.json platform entry is invalid: {platform_name}")
        url = platform_info.get("url")
        if not isinstance(url, str) or expected_segment not in url:
            raise ReleaseError(
                f"latest.json platform URL is not pinned to {tag_name}: {platform_name}",
            )

    return manifest


def wait_for_latest_manifest(
    root: Path,
    owner: str,
    repo: str,
    version: str,
    tag_name: str,
    *,
    timeout_seconds: int,
    poll_interval_seconds: int,
) -> dict[str, Any]:
    url = f"https://github.com/{owner}/{repo}/releases/latest/download/latest.json"
    deadline = time.time() + timeout_seconds
    last_error: str | None = None

    while time.time() < deadline:
        try:
            manifest_text = run_command(
                ["curl", "-L", "--fail", "--silent", url],
                cwd=root,
                capture_output=True,
            )
            return validate_latest_manifest(manifest_text, version, tag_name)
        except ReleaseError as error:
            last_error = str(error)
            print(f"Waiting for latest.json to become ready: {last_error}", flush=True)
            time.sleep(poll_interval_seconds)

    raise ReleaseError(last_error or "Timed out waiting for latest.json")


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Publish MCP Manager end-to-end with automatic patch bump by default.",
    )
    parser.add_argument("--version", help="Explicit version to release. Defaults to the next patch version.")
    parser.add_argument("--branch", default="main", help="Branch to push before tagging. Default: main")
    parser.add_argument("--remote", default="origin", help="Git remote to push. Default: origin")
    parser.add_argument(
        "--workflow-timeout-seconds",
        type=int,
        default=5400,
        help="How long to wait for the release workflow. Default: 5400",
    )
    parser.add_argument(
        "--manifest-timeout-seconds",
        type=int,
        default=300,
        help="How long to wait for latest.json after publishing the release. Default: 300",
    )
    parser.add_argument(
        "--poll-interval-seconds",
        type=int,
        default=15,
        help="Polling interval for workflow and manifest checks. Default: 15",
    )
    parser.add_argument("--dry-run", action="store_true", help="Resolve version and repository context without changes.")
    return parser.parse_args(argv)


def main(argv: Sequence[str]) -> int:
    args = parse_args(argv)
    root = repo_root()

    ensure_clean_worktree(root)
    ensure_branch(root, args.branch)

    current_version = load_project_version(root)
    version = args.version or next_patch_version(current_version)
    if not SEMVER_PATTERN.fullmatch(version):
        raise ReleaseError(f"Invalid release version: {version}")

    tag_name = f"v{version}"
    remote_url = run_command(["git", "remote", "get-url", args.remote], cwd=root, capture_output=True).strip()
    owner, repo = parse_github_repository(remote_url)

    print(f"Current version: {current_version}", flush=True)
    print(f"Release version: {version}", flush=True)
    print(f"Repository: {owner}/{repo}", flush=True)
    print(f"Branch: {args.branch}", flush=True)

    if args.dry_run:
        print("Dry run complete. No changes made.", flush=True)
        return 0

    run_command(["make", "release-prepare", f"VERSION={version}"], cwd=root)
    run_command(["git", "add", *VERSION_FILES], cwd=root)
    run_command(["git", "commit", "-m", f"chore: release {tag_name}"], cwd=root)
    run_command(["git", "push", args.remote, args.branch], cwd=root)
    run_command(["make", "release-publish", f"VERSION={version}"], cwd=root)

    workflow_run = wait_for_release_run(
        root,
        owner,
        repo,
        tag_name,
        timeout_seconds=args.workflow_timeout_seconds,
        poll_interval_seconds=args.poll_interval_seconds,
    )
    release = gh_api_json(root, f"repos/{owner}/{repo}/releases/tags/{tag_name}")
    if release.get("draft"):
        release = gh_api_json(
            root,
            f"repos/{owner}/{repo}/releases/{release['id']}",
            method="PATCH",
            fields=["draft=false"],
        )

    validate_latest_manifest(
        json.dumps(
            wait_for_latest_manifest(
                root,
                owner,
                repo,
                version,
                tag_name,
                timeout_seconds=args.manifest_timeout_seconds,
                poll_interval_seconds=args.poll_interval_seconds,
            ),
        ),
        version,
        tag_name,
    )

    print(f"Release workflow: {workflow_run.get('html_url')}", flush=True)
    print(f"Published release: {release.get('html_url')}", flush=True)
    print(
        f"Updater manifest: https://github.com/{owner}/{repo}/releases/latest/download/latest.json",
        flush=True,
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ReleaseError as error:
        print(str(error), file=sys.stderr, flush=True)
        raise SystemExit(1)
