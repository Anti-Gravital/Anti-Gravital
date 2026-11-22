#!/usr/bin/env python3
"""Drive the interactive `ag` interface in a real pseudo-terminal.

Standard library only. The harness spawns the binary in a pty (or with piped
stdin), injects a scripted sequence of keystrokes, records the raw output with
timestamps, and reconstructs the visible screen from the ANSI stream. Nothing is
generated or guessed: every transcript entry is captured from a live process.

Usage:
    python3 tools/tui-pty-harness.py --binary /path/to/ag --out /path/to/out
"""

from __future__ import annotations

import argparse
import errno
import fcntl
import json
import os
import pty
import re
import select
import struct
import subprocess
import termios
import threading
import time
from datetime import datetime, timezone
from pathlib import Path


ESC = "\x1b"
COLOR_SGR = re.compile(r"\x1b\[(?:38|48);[0-9;]*m")
ALT_ENTER = "\x1b[?1049h"
ALT_LEAVE = "\x1b[?1049l"
CURSOR_SHOW = "\x1b[?25h"
CURSOR_HIDE = "\x1b[?25l"


class VTerm:
    """Minimal ANSI interpreter: enough to rebuild what a user saw on screen."""

    def __init__(self, rows: int, cols: int) -> None:
        self.rows = rows
        self.cols = cols
        self.buf = [[" "] * cols for _ in range(rows)]
        self.r = 0
        self.c = 0

    def reset(self) -> None:
        """A new screen buffer (alternate screen entered or left)."""
        self.buf = [[" "] * self.cols for _ in range(self.rows)]
        self.r = 0
        self.c = 0

    def _put(self, ch: str) -> None:
        if ch == "\n":
            self.r = min(self.r + 1, self.rows - 1)
        elif ch == "\r":
            self.c = 0
        elif ch == "\b":
            self.c = max(0, self.c - 1)
        else:
            if self.c >= self.cols:
                self.c = 0
                self.r = min(self.r + 1, self.rows - 1)
            self.buf[self.r][self.c] = ch
            self.c += 1

    def _erase_display(self, mode: int) -> None:
        if mode == 2:
            self.buf = [[" "] * self.cols for _ in range(self.rows)]
        elif mode == 0:
            for c in range(self.c, self.cols):
                self.buf[self.r][c] = " "
            for r in range(self.r + 1, self.rows):
                self.buf[r] = [" "] * self.cols
        elif mode == 1:
            for r in range(0, self.r):
                self.buf[r] = [" "] * self.cols
            for c in range(0, self.c + 1):
                self.buf[self.r][c] = " "

    def _erase_line(self, mode: int) -> None:
        if mode == 2:
            self.buf[self.r] = [" "] * self.cols
        elif mode == 0:
            for c in range(self.c, self.cols):
                self.buf[self.r][c] = " "
        elif mode == 1:
            for c in range(0, self.c + 1):
                self.buf[self.r][c] = " "

    def _csi(self, params: str, final: str) -> None:
        nums = []
        for part in params.split(";"):
            if part.isdigit():
                nums.append(int(part))
            elif part.startswith("?"):
                nums.append(int(part[1:]) if part[1:].isdigit() else 0)
            else:
                nums.append(0)
        first = nums[0] if nums else 0

        if final in ("H", "f"):
            row = nums[0] if len(nums) > 0 and nums[0] else 1
            col = nums[1] if len(nums) > 1 and nums[1] else 1
            self.r = max(0, min(self.rows - 1, row - 1))
            self.c = max(0, min(self.cols - 1, col - 1))
        elif final == "A":
            self.r = max(0, self.r - (first or 1))
        elif final == "B":
            self.r = min(self.rows - 1, self.r + (first or 1))
        elif final == "C":
            self.c = min(self.cols - 1, self.c + (first or 1))
        elif final == "D":
            self.c = max(0, self.c - (first or 1))
        elif final == "J":
            self._erase_display(first)
        elif final == "K":
            self._erase_line(first)
        # SGR, cursor visibility and mode switches are intentionally ignored:
        # they affect colour and cursor, not the glyphs on screen.

    def feed(self, text: str) -> None:
        i = 0
        n = len(text)
        while i < n:
            ch = text[i]
            if ch == ESC:
                if i + 1 >= n:
                    break
                nxt = text[i + 1]
                if nxt == "[":
                    j = i + 2
                    while j < n and not (0x40 <= ord(text[j]) <= 0x7E):
                        j += 1
                    if j >= n:
                        break
                    self._csi(text[i + 2 : j], text[j])
                    i = j + 1
                elif nxt in "()":
                    i += 3
                elif nxt == "]":
                    j = i + 2
                    while j < n and text[j] not in "\x07\x1b":
                        j += 1
                    i = j + 1 if j < n else n
                else:
                    i += 2
            else:
                self._put(ch)
                i += 1

    def screen(self) -> str:
        return "\n".join("".join(row).rstrip() for row in self.buf)


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="milliseconds")


def decode(raw: bytes) -> str:
    return raw.decode("utf-8", errors="replace")


def raw_repr(raw: bytes, cap: int = 12000) -> str:
    text = repr(raw)
    if len(text) > cap:
        return text[:cap] + f"... [truncated {len(text) - cap} chars]"
    return text


class Capture:
    """Collects timestamped output chunks from a child process."""

    def __init__(self, fd: int) -> None:
        self.fd = fd
        self.chunks: list[tuple[float, bytes]] = []
        self.condition = threading.Condition()
        self.thread = threading.Thread(target=self._read, daemon=True)

    def start(self) -> None:
        self.thread.start()

    def _read(self) -> None:
        while True:
            try:
                readable, _, _ = select.select([self.fd], [], [], 0.2)
                if not readable:
                    continue
                data = os.read(self.fd, 65536)
            except OSError as exc:
                if exc.errno in (errno.EIO, errno.EBADF):
                    break
                raise
            if not data:
                break
            with self.condition:
                self.chunks.append((time.monotonic(), data))
                self.condition.notify_all()

    def mark(self) -> int:
        with self.condition:
            return len(self.chunks)

    def since(self, mark: int) -> bytes:
        with self.condition:
            return b"".join(chunk for _, chunk in self.chunks[mark:])

    def all_bytes(self) -> bytes:
        with self.condition:
            return b"".join(chunk for _, chunk in self.chunks)

    def finish(self) -> None:
        self.thread.join(timeout=5)


def make_env(overrides: dict[str, str], cwd: Path) -> dict[str, str]:
    env = os.environ.copy()
    for key in ("CI", "AG_NO_TUI", "NO_COLOR", "TERM", "COLUMNS", "LINES"):
        env.pop(key, None)
    env["TERM"] = "xterm-256color"
    env["HOME"] = str(cwd)
    env.update(overrides)
    return env


def step_record(label: str, sent: bytes | None, delay: float, raw: bytes,
                vterm: VTerm) -> dict:
    text = decode(raw)
    # Ratatui draws differentially, so the screen must accumulate across steps.
    # Entering/leaving the alternate screen swaps the buffer.
    if ALT_ENTER in text or ALT_LEAVE in text:
        vterm.reset()
    vterm.feed(text)
    return {
        "label": label,
        "sent": raw_repr(sent) if sent is not None else None,
        "delay_after_s": delay,
        "raw_len": len(raw),
        "raw_repr": raw_repr(raw),
        "screen": vterm.screen(),
        "color_sgr_count": len(COLOR_SGR.findall(text)),
        "alt_screen_enter": text.count(ALT_ENTER),
        "alt_screen_leave": text.count(ALT_LEAVE),
        "cursor_show": text.count(CURSOR_SHOW),
        "cursor_hide": text.count(CURSOR_HIDE),
    }


def snapshot_files(cwd: Path) -> dict[str, str]:
    result: dict[str, str] = {}
    for path in sorted(cwd.iterdir()):
        if path.is_file() and path.stat().st_size < 4096:
            try:
                result[path.name] = path.read_text(errors="replace")
            except OSError:
                result[path.name] = "<unreadable>"
        elif path.is_dir():
            result[path.name + "/"] = ", ".join(sorted(p.name for p in path.iterdir()))
    return result


def run_pty(scenario: dict, binary: str, base_dir: Path) -> dict:
    rows = scenario["rows"]
    cols = scenario["cols"]
    cwd = base_dir / scenario["id"]
    cwd.mkdir(parents=True, exist_ok=True)
    for name, content in scenario.get("setup_files", {}).items():
        (cwd / name).write_text(content)
    env = make_env(scenario.get("env", {}), cwd)
    argv = [binary] + scenario["argv"]
    started = now_iso()

    pid, master = pty.fork()
    if pid == 0:
        try:
            os.chdir(cwd)
            fcntl.ioctl(0, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
            for key, value in env.items():
                os.environ[key] = value
            os.execve(binary, argv, env)
        except BaseException:
            os._exit(127)

    capture = Capture(master)
    steps = []
    vterm = VTerm(rows, cols)
    marker = 0
    capture.start()
    # Wait for the first frame instead of assuming a fixed startup time.
    waited = 0.0
    while not capture.since(marker) and waited < 1.5:
        time.sleep(0.05)
        waited += 0.05
    time.sleep(0.15)
    steps.append(step_record("launch", None, round(waited + 0.15, 2),
                             capture.since(marker), vterm))

    for action in scenario["actions"]:
        label = action["label"]
        payload = action.get("send", "").encode() if "send" in action else None
        delay = float(action.get("delay", 0.3))
        marker = capture.mark()
        if payload is not None and payload:
            os.write(master, payload)
        time.sleep(delay)
        steps.append(step_record(label, payload, delay, capture.since(marker), vterm))

    timed_out = False
    deadline = time.monotonic() + 2.0
    while deadline > time.monotonic():
        waited, status = os.waitpid(pid, os.WNOHANG)
        if waited:
            exit_code = os.waitstatus_to_exitcode(status)
            break
        time.sleep(0.05)
    else:
        timed_out = True
        os.kill(pid, 9)
        _, status = os.waitpid(pid, 0)
        exit_code = os.waitstatus_to_exitcode(status)

    time.sleep(0.1)
    capture.finish()
    os.close(master)
    return {
        "id": scenario["id"],
        "description": scenario["description"],
        "mode": "pty",
        "argv": argv,
        "cwd": str(cwd),
        "env": env,
        "size": {"rows": rows, "cols": cols},
        "started_at": started,
        "finished_at": now_iso(),
        "exit_code": exit_code,
        "timed_out": timed_out,
        "total_bytes": len(capture.all_bytes()),
        "actions": scenario["actions"],
        "steps": steps,
        "final_files": snapshot_files(cwd),
    }


def run_pipe(scenario: dict, binary: str, base_dir: Path, pty_stdout: bool) -> dict:
    cwd = base_dir / scenario["id"]
    cwd.mkdir(parents=True, exist_ok=True)
    env = make_env(scenario.get("env", {}), cwd)
    argv = [binary] + scenario["argv"]
    started = now_iso()
    master = None
    if pty_stdout:
        master, slave = pty.openpty()
        stdout_target = slave
    else:
        slave = None
        stdout_target = subprocess.PIPE
    try:
        proc = subprocess.Popen(
            argv,
            cwd=cwd,
            env=env,
            stdin=subprocess.DEVNULL,
            stdout=stdout_target,
            stderr=subprocess.STDOUT,
            start_new_session=pty_stdout,
        )
    finally:
        if slave is not None:
            os.close(slave)

    if pty_stdout:
        capture = Capture(master)
        capture.start()
        time.sleep(0.6)
        timed_out = False
        deadline = time.monotonic() + 2.0
        while deadline > time.monotonic():
            waited, status = os.waitpid(proc.pid, os.WNOHANG)
            if waited:
                exit_code = os.waitstatus_to_exitcode(status)
                break
            time.sleep(0.05)
        else:
            timed_out = True
            proc.kill()
            _, status = os.waitpid(proc.pid, 0)
            exit_code = os.waitstatus_to_exitcode(status)
        capture.finish()
        raw = capture.all_bytes()
        os.close(master)
    else:
        try:
            raw, _ = proc.communicate(timeout=2.0)
            exit_code = proc.returncode
            timed_out = False
        except subprocess.TimeoutExpired:
            proc.kill()
            raw, _ = proc.communicate()
            exit_code = proc.returncode
            timed_out = True

    steps = [step_record("launch", None, 0.6, raw, VTerm(24, 80))]
    return {
        "id": scenario["id"],
        "description": scenario["description"],
        "mode": "pipe-stdin-pty-out" if pty_stdout else "pipe",
        "argv": argv,
        "cwd": str(cwd),
        "env": env,
        "size": {"rows": 24, "cols": 80},
        "started_at": started,
        "finished_at": now_iso(),
        "exit_code": exit_code,
        "timed_out": timed_out,
        "total_bytes": len(raw),
        "actions": [],
        "steps": steps,
        "final_files": snapshot_files(cwd),
    }


SCENARIOS: list[dict] = [
    {
        "id": "home-nav-launch",
        "description": "Bare ag home screen outside a project; arrow navigation then Enter on the version entry.",
        "argv": [], "rows": 30, "cols": 100,
        "env": {},
        "actions": [
            {"label": "down-1", "send": "\x1b[B", "delay": 0.3},
            {"label": "down-2", "send": "\x1b[B", "delay": 0.3},
            {"label": "enter-launch", "send": "\r", "delay": 0.6},
        ],
    },
    {
        "id": "home-in-project",
        "description": "Bare ag home screen inside a project; context menu, quit with q.",
        "argv": [], "rows": 30, "cols": 100, "env": {},
        "setup_files": {
            "config.toml": 'bind = "0.0.0.0:8080"\n',
            "Cargo.toml": '[package]\nname = "demo"\nversion = "0.1.0"\n'
                          '\n[dependencies]\nag-core = "0.1"\n',
        },
        "actions": [{"label": "q-quit", "send": "q", "delay": 0.4}],
    },
    {
        "id": "home-quit",
        "description": "Bare ag home screen; quit with q and check terminal restoration.",
        "argv": [], "rows": 30, "cols": 100, "env": {},
        "actions": [{"label": "q-quit", "send": "q", "delay": 0.4}],
    },
    {
        "id": "new-wizard-full",
        "description": "ag new with no name or template; complete all three wizard steps and create a project.",
        "argv": ["new"], "rows": 30, "cols": 100, "env": {},
        "actions": [
            {"label": "type-name", "send": "demo-app", "delay": 0.4},
            {"label": "enter-template-step", "send": "\r", "delay": 0.4},
            {"label": "down-template", "send": "\x1b[B", "delay": 0.3},
            {"label": "enter-review", "send": "\r", "delay": 0.4},
            {"label": "enter-submit", "send": "\r", "delay": 0.8},
        ],
    },
    {
        "id": "new-wizard-cancel",
        "description": "ag new wizard; type a name, go to template step, abort with q. Nothing should be created.",
        "argv": ["new"], "rows": 30, "cols": 100, "env": {},
        "actions": [
            {"label": "type-name", "send": "cancelme", "delay": 0.3},
            {"label": "enter-template-step", "send": "\r", "delay": 0.3},
            {"label": "q-abort", "send": "q", "delay": 0.4},
        ],
    },
    {
        "id": "new-wizard-invalid-name",
        "description": "ag new wizard; invalid name with a space shows an inline error, then backspace to fix and Esc to cancel.",
        "argv": ["new"], "rows": 30, "cols": 100, "env": {},
        "actions": [
            {"label": "type-invalid-name", "send": "bad name", "delay": 0.3},
            {"label": "enter-with-error", "send": "\r", "delay": 0.3},
            {"label": "backspace-space", "send": "\x7f", "delay": 0.3},
            {"label": "finish-name", "send": "okname", "delay": 0.3},
            {"label": "esc-cancel", "send": "\x1b", "delay": 0.4},
        ],
    },
    {
        "id": "config-form-edit-submit",
        "description": "ag config form; edit bind, review and submit. config.toml should be written.",
        "argv": ["config"], "rows": 30, "cols": 100, "env": {},
        "actions": [
            {"label": "enter-edit", "send": "\r", "delay": 0.3},
            {"label": "clear-buffer", "send": "\x7f" * 30, "delay": 0.3},
            {"label": "type-bind", "send": "127.0.0.1:9000", "delay": 0.3},
            {"label": "commit-edit", "send": "\r", "delay": 0.3},
            {"label": "r-review", "send": "r", "delay": 0.3},
            {"label": "enter-submit", "send": "\r", "delay": 0.6},
        ],
    },
    {
        "id": "config-form-quit",
        "description": "ag config form; quit with q and confirm no file change.",
        "argv": ["config"], "rows": 30, "cols": 100, "env": {},
        "actions": [{"label": "q-quit", "send": "q", "delay": 0.4}],
    },
    {
        "id": "config-small-terminal",
        "description": "ag config in a 60x20 terminal: must fail fast with a size error, not hang.",
        "argv": ["config"], "rows": 20, "cols": 60, "env": {},
        "actions": [],
    },
    {
        "id": "new-small-terminal-lines",
        "description": "ag new in a 60x20 terminal: degrades to line prompts and completes with defaults.",
        "argv": ["new"], "rows": 20, "cols": 60, "env": {},
        "actions": [
            {"label": "name", "send": "lineproj\n", "delay": 0.3},
            {"label": "template-default", "send": "\n", "delay": 0.3},
            {"label": "confirm", "send": "y\n", "delay": 0.8},
        ],
    },
    {
        "id": "new-small-terminal-eof",
        "description": "ag new in a 60x20 terminal with EOF (Ctrl-D) at the first prompt: must cancel, not hang.",
        "argv": ["new"], "rows": 20, "cols": 60, "env": {},
        "actions": [{"label": "ctrl-d", "send": "\x04", "delay": 0.5}],
    },
    {
        "id": "home-no-color",
        "description": "Bare ag home with NO_COLOR=1: still interactive, but no colour SGR sequences.",
        "argv": [], "rows": 30, "cols": 100, "env": {"NO_COLOR": "1"},
        "actions": [{"label": "q-quit", "send": "q", "delay": 0.4}],
    },
    {
        "id": "home-term-dumb",
        "description": "Bare ag with TERM=dumb: non-interactive help, no alternate screen.",
        "argv": [], "rows": 30, "cols": 100, "env": {"TERM": "dumb"},
        "actions": [],
    },
    {
        "id": "new-noninteractive-flag",
        "description": "ag new --no-interactive in a full terminal: usage error, no prompt.",
        "argv": ["new", "--no-interactive"], "rows": 30, "cols": 100, "env": {},
        "actions": [],
    },
    {
        "id": "config-noninteractive-flag",
        "description": "ag config --no-interactive in a full terminal: usage error, no form.",
        "argv": ["config", "--no-interactive"], "rows": 30, "cols": 100, "env": {},
        "actions": [],
    },
    {
        "id": "new-stdin-pipe",
        "description": "ag new with piped (non-TTY) stdin: usage error, no hang, nothing created.",
        "argv": ["new"], "mode": "pipe", "env": {}, "actions": [],
    },
    {
        "id": "home-pipe-stdin-tty-out",
        "description": "Bare ag with piped stdin but TTY stdout: prints help, no TUI.",
        "argv": [], "mode": "pipe-stdin-pty-out", "env": {}, "actions": [],
    },
]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True, help="path to the ag binary")
    parser.add_argument("--out", required=True, help="output directory for transcripts")
    args = parser.parse_args()
    binary = str(Path(args.binary).resolve())
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    base = out / "work"
    base.mkdir(parents=True, exist_ok=True)

    summary = []
    for scenario in SCENARIOS:
        if scenario.get("mode") == "pipe":
            record = run_pipe(scenario, binary, base, pty_stdout=False)
        elif scenario.get("mode") == "pipe-stdin-pty-out":
            record = run_pipe(scenario, binary, base, pty_stdout=True)
        else:
            record = run_pty(scenario, binary, base)
        path = out / f"{scenario['id']}.json"
        path.write_text(json.dumps(record, indent=2, ensure_ascii=False) + "\n")
        summary.append({
            "id": scenario["id"],
            "exit_code": record["exit_code"],
            "timed_out": record["timed_out"],
            "total_bytes": record["total_bytes"],
            "transcript": str(path),
        })
        print(f"{scenario['id']:32} exit={record['exit_code']} "
              f"timeout={record['timed_out']} bytes={record['total_bytes']}")

    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")


if __name__ == "__main__":
    main()
