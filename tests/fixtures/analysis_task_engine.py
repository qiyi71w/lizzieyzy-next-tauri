"""Controllable JSONL engine for task lifecycle tests; not KataGo acceptance."""
import json
import os
from pathlib import Path
import sys
import threading
import time

root = Path(os.environ["TASK_ENGINE_DIR"])
lock = threading.Lock()
active = set()

def emit(value):
    with lock:
        print(json.dumps(value), flush=True)

def frame(query, during=False):
    visits = query.get("maxVisits", 8)
    return dict(id=query["id"], turnNumber=0, isDuringSearch=during,
                rootInfo=dict(visits=visits, winrate=0.6, scoreLead=2.5),
                moveInfos=[dict(move="D4", visits=visits, winrate=0.6,
                                scoreMean=2.5, pv=["D4"])])

def finish_cancel(query):
    target = query["terminateId"]
    (root / "cancel-seen").touch()
    while not (root / "cancel-final").exists():
        time.sleep(0.01)
    active.discard(target)
    # A budget-sized final after the Pause fence still must not count.
    emit(frame(dict(id=target, maxVisits=999)))

def search(query):
    identity = query["id"]
    # One-thread fixtures keep the finite task in the engine queue.
    while (root / "one-thread").exists() and any(x != identity for x in active):
        if identity not in active:
            return
        time.sleep(0.01)
    if identity not in active:
        return
    emit(frame(query, True))
    if "overrideSettings" in query:
        while identity in active:
            emit(frame(query, True))
            time.sleep(0.05)
        return
    while not (root / "finish").exists():
        if identity not in active:
            return
        time.sleep(0.01)
    if identity in active:
        active.discard(identity)
        emit(frame(query))

first = True
finite_count = 0
for line in sys.stdin:
    query = json.loads(line)
    if query.get("action") == "terminate":
        emit(query)  # Control ACK is not target-final cleanup.
        threading.Thread(target=finish_cancel, args=(query,), daemon=True).start()
    elif first:
        first = False
        emit(frame(query))
    else:
        with (root / "queries.jsonl").open("a") as log:
            log.write(json.dumps(query) + "\n")
        if "overrideSettings" not in query:
            finite_count += 1
        if finite_count == 1 and "overrideSettings" not in query and not (root / "hold-first").exists():
            emit(frame(query))
        else:
            active.add(query["id"])
            threading.Thread(target=search, args=(query,), daemon=True).start()
            if "overrideSettings" not in query and (root / "close-input").exists():
                os.close(0)
                (root / "input-closed").touch()
                while True:
                    time.sleep(0.05)
