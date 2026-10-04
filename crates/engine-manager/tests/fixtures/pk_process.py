"""Controlled dual-process KataGo/GTP protocol boundary for PK owner regression tests.

Each real PID records its argv and requests. KataGo per-job reply files release results
without blocking stdin, so terminate acknowledgements and late replies can be ordered.
A GNU Go argv selects the GTP dialect: each genmove publishes `genmove-<pid>` with the
commands synchronized since `boardsize`, then waits for `reply-gtp-<pid>`.
"""
import fcntl
import json
import os
from pathlib import Path
import select
import sys
import time

pid = os.getpid()
with open("pids", "a+") as ledger:
    fcntl.flock(ledger, fcntl.LOCK_EX)
    ledger.seek(0)
    ordinal = len(ledger.readlines()) + 1
    ledger.write(f"{pid}\n")
    ledger.flush()
    fcntl.flock(ledger, fcntl.LOCK_UN)
Path(f"argv-{pid}").write_text(json.dumps(sys.argv[1:]))


def startup_behavior():
    startup = Path(f"startup-{ordinal}")
    behavior = startup.read_text() if startup.exists() else "ready"
    if behavior == "fail":
        raise SystemExit(3)
    if behavior == "hold":
        while not Path(f"release-startup-{ordinal}").exists():
            time.sleep(0.005)
    return behavior


def gtp():
    commands = "boardsize\nclear_board\nkomi\nplay\ngenmove\nquit\ntime_settings\ntime_left"
    first = True
    synced = []
    buffer = b""
    while True:
        if Path(f"exit-{pid}").exists():
            raise SystemExit(7)
        readable, _, _ = select.select([sys.stdin], [], [], 0.005)
        if readable:
            chunk = os.read(sys.stdin.fileno(), 65536)
            if not chunk:
                return
            buffer += chunk
        while b"\n" in buffer:
            raw, buffer = buffer.split(b"\n", 1)
            line = raw.decode().strip()
            if not line:
                continue
            with open(f"trace-{pid}", "a") as trace:
                trace.write(line + "\n")
            ident, command = line.split(" ", 1)
            if first:
                first = False
                startup_behavior()
            body = {"protocol_version": "2", "name": "GNU Go", "version": "3.8",
                    "list_commands": commands}.get(command, "")
            if command.startswith("boardsize "):
                synced = [command]
            elif command.startswith(("clear_board", "komi ", "play ", "time_settings ", "time_left ")):
                synced.append(command)
            elif command.startswith("genmove "):
                pending = Path(f"genmove-{pid}")
                pending.write_text(json.dumps({"pid": pid, "command": command, "synced": synced}))
                reply = Path(f"reply-gtp-{pid}")
                while not reply.exists() or not reply.read_text():
                    if Path(f"exit-{pid}").exists():
                        raise SystemExit(7)
                    time.sleep(0.005)
                action = reply.read_text()
                reply.unlink()
                pending.unlink()
                if action == "eof":
                    raise SystemExit(5)
                if action == "protocol":
                    sys.stdout.write(f"?{ident} controlled protocol failure\n\n")
                    sys.stdout.flush()
                    continue
                body = action
            sys.stdout.write(f"={ident} {body}\n\n")
            sys.stdout.flush()
            if command == "quit":
                raise SystemExit(0)


if "--mode" in sys.argv and "gtp" in sys.argv:
    gtp()
    raise SystemExit(0)

first = True
pending = {}
buffer = b""

def emit(value):
    print(json.dumps(value), flush=True)

while True:
    if Path(f"exit-{pid}").exists():
        raise SystemExit(7)
    readable, _, _ = select.select([sys.stdin], [], [], 0.005)
    if readable:
        chunk = os.read(sys.stdin.fileno(), 65536)
        if not chunk:
            break
        buffer += chunk
    while b"\n" in buffer:
        raw, buffer = buffer.split(b"\n", 1)
        request = json.loads(raw)
        with open(f"trace-{pid}", "a") as trace:
            trace.write(json.dumps(request) + "\n")
        ident = request["id"]
        if request.get("action") == "terminate":
            target = request["terminateId"]
            Path(f"cancel-{target}").write_text(str(pid))
            if not Path("hold-cancel").exists():
                emit({"id": target, "turnNumber": len(pending.get(target, {}).get("moves", [])),
                      "isDuringSearch": False, "noResults": True})
            continue
        if first:
            first = False
            startup = Path(f"startup-{ordinal}")
            behavior = startup.read_text() if startup.exists() else "ready"
            if behavior == "fail":
                raise SystemExit(3)
            if behavior == "hold":
                while not Path(f"release-startup-{ordinal}").exists():
                    time.sleep(0.005)
            emit({"id": ident, "turnNumber": 0})
            if behavior == "exit":
                raise SystemExit(4)
            continue
        pending[ident] = request
        Path(f"pending-{ident}").write_text(json.dumps({"pid": pid, "request": request}))
    for ident, request in list(pending.items()):
        reply = Path(f"reply-{ident}")
        if not reply.exists():
            continue
        action = reply.read_text()
        if not action:
            continue
        reply.unlink()
        if action == "eof":
            raise SystemExit(5)
        if action == "protocol":
            emit({"id": ident, "error": "controlled protocol failure"})
        else:
            emit({"id": ident, "turnNumber": len(request.get("moves", [])),
                  "isDuringSearch": False, "moveInfos": [{"move": action, "order": 0, "visits": 8}]})
        del pending[ident]
