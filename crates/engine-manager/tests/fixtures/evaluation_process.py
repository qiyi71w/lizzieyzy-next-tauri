#!/usr/bin/python3
"""Controlled child for the production saved-profile benchmark slot."""
import json
import os
import pathlib
import sys
import time

pathlib.Path("spawned.json").write_text(json.dumps({"argv": sys.argv[1:], "pid": os.getpid()}))

if sys.argv[1] == "version":
    print("KataGo v1.18.2")
    print("Using Eigen(CPU) backend")
    sys.exit(0)
if sys.argv[1] == "analysis":
    pathlib.Path("launch.json").write_text(json.dumps({"argv": sys.argv[1:], "cwd": os.getcwd(), "pid": os.getpid()}))
    first = True
    for line in sys.stdin:
        query = json.loads(line)
        if query.get("action") == "terminate":
            print(json.dumps(query), flush=True)
            print(json.dumps({"id": query["terminateId"], "turnNumber": 0, "isDuringSearch": False, "noResults": True}), flush=True)
        elif first:
            print(json.dumps({"id": query["id"], "turnNumber": 0}), flush=True)
            first = False
        else:
            print(json.dumps({"id": query["id"], "turnNumber": 0, "isDuringSearch": True,
                "rootInfo": {"visits": 1, "winrate": 0.5, "scoreMean": 0},
                "moveInfos": [{"move": "D4", "visits": 1, "winrate": 0.5, "scoreMean": 0}]}), flush=True)
    sys.exit(0)
pathlib.Path("observed.json").write_text(json.dumps({"argv": sys.argv[1:], "cwd": os.getcwd(), "pid": os.getpid()}))
print("benchmark started", flush=True)
mode = next((entry.split("=", 1)[1] for arg in sys.argv for entry in arg.split(",") if entry.startswith("testMode=")), "success")
if mode == "hold":
    while not pathlib.Path("release").exists():
        time.sleep(0.01)
if mode == "failure":
    print("failed secret=do-not-show /private/user/path", flush=True)
    sys.exit(7)
if mode == "help":
    print("benchmark help, not a measurement", flush=True)
    sys.exit(0)
if mode == "privacy":
    print('token="private-first-line', flush=True)
    print('private-second-line" /home/private-owner/model.bin \x1b[31m', flush=True)
if mode in ("raw", "sweep", "variations"):
    filename = {"raw": "stdout.txt", "sweep": "sweep-stdout.txt", "variations": "variations.txt"}[mode]
    sys.stdout.buffer.write((pathlib.Path(__file__).parent / "katago-1.18.2-benchmark" / filename).read_bytes())
    sys.stdout.flush()
    # Rows must survive diagnostic-tail eviction without being paired across rows.
    for i in range(150):
        print("trailing diagnostic", flush=True)
    sys.exit(0)
if mode in ("truncated", "overlong", "incomplete"):
    row = "numSearchThreads = 1: 1 / 1 positions, visits/s = 123 nnEvals/s = 87 (0.3 secs)"
    if mode == "overlong":
        row += " " * 2048 + "\n"
    elif mode == "incomplete":
        row = "numSearchThreads = 1: 0 / 1 positions, visits/s = 123 nnEvals/s = 87 (0.3 secs)\n"
    sys.stdout.write(row)
    sys.exit(0)
if mode == "burst":
    for i in range(300):
        print("x" * 4096, flush=True)
print("numSearchThreads = 1: 1 / 1 positions, visits/s = " + ("unavailable" if mode == "missing" else "123.5") + " nnEvals/s = 87.25 (0.3 secs)", flush=True)
