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
queries = {}

def emit(value):
    with lock:
        print(json.dumps(value), flush=True)

def configured_visits(query):
    value = query.get("overrideSettings", {}).get("maxVisits", query.get("maxVisits", 8))
    return min(value, 999)

def frame(query, during=False, visits=None, candidates=None):
    visits = configured_visits(query) if visits is None else visits
    if candidates is None:
        candidates = [dict(move="D4", order=0, visits=visits, winrate=0.6,
                           scoreMean=2.5, pv=["D4"])]
    return dict(id=query["id"], turnNumber=0, isDuringSearch=during,
                rootInfo=dict(visits=visits, winrate=0.6, scoreLead=2.5),
                moveInfos=candidates)

def invalid_final(query, kind):
    result = frame(query, visits=8)
    if kind == "unmarked":
        result.pop("isDuringSearch")
    elif kind == "empty":
        result["moveInfos"] = []
    elif kind == "semantic":
        result["moveInfos"][0].update(move="Z99", pv=["Z99"])
    elif kind == "no_results":
        result["noResults"] = True
    else:
        result["rootInfo"]["visits"] = -1
    return result

def finish_cancel(control):
    target = control["terminateId"]
    (root / "cancel-seen").touch()
    while not (root / "cancel-final").exists():
        time.sleep(0.01)
    query = queries[target]
    active.discard(target)
    if (root / "malformed-final").exists():
        emit(invalid_final(query, (root / "malformed-final").read_text().strip()))
        return
    mode = (root / "budget-smoke").read_text().strip() if (root / "budget-smoke").exists() else ""
    if mode == "natural_race":
        if query.get("fixtureIndex", 0) % 2 == 0:
            (root / "stale-terminate").touch()
        emit(frame(query, visits=4))
        return
    if mode == "total_visits":
        emit(frame(query, visits=8))
    elif mode == "leading_candidate_visits":
        emit(frame(query, visits=12, candidates=[
            dict(move="D4", order=0, visits=5, winrate=0.6, scoreMean=2.5, pv=["D4"])
        ]))
    else:
        # A budget-sized final after a strong Pause/Cancel fence still must not count.
        emit(frame(query, visits=999))

def search(query):
    identity = query["id"]
    # One-thread fixtures keep the finite task in the engine queue.
    while (root / "one-thread").exists() and any(x != identity for x in active):
        if identity not in active:
            return
        time.sleep(0.01)
    if identity not in active:
        return
    mode = (root / "budget-smoke").read_text().strip() if (root / "budget-smoke").exists() else ""
    if mode in ("unmarked", "empty"):
        active.discard(identity)
        emit(invalid_final(query, mode))
        return
    if mode == "natural_race":
        emit(frame(query, True, visits=4))
        time.sleep(0.03)
        if identity in active:
            active.discard(identity)
            emit(frame(query, visits=8))
        return
    if mode == "time_seconds" and ":" in identity:
        started = time.monotonic()
        seconds = query["overrideSettings"]["maxTime"]
        while identity in active and time.monotonic() - started < seconds:
            emit(frame(query, True, visits=4))
            time.sleep(query.get("reportDuringSearchEvery", 0.1))
        if identity in active:
            active.discard(identity)
            emit(frame(query, visits=4))
        return
    if ":" not in identity and "overrideSettings" in query:
        while identity in active:
            emit(frame(query, True))
            time.sleep(query.get("reportDuringSearchEvery", 0.1))
        return
    if mode == "invalid_report":
        emit(frame(query, True, visits=8, candidates=[
            dict(move="Z99", order=0, visits=8, winrate=0.6, scoreMean=2.5, pv=["Z99"])
        ]))
        (root / "invalid-sent").touch()
        while not (root / "finish").exists() and identity in active:
            time.sleep(0.01)
        if identity in active:
            active.discard(identity)
            emit(frame(query, visits=8))
        return
    if mode == "total_visits":
        emit(frame(query, True, visits=4))
        emit(frame(query, True, visits=8))
        return
    if mode == "leading_candidate_visits":
        emit(frame(query, True, visits=8, candidates=[
            dict(move="D4", order=0, visits=4, winrate=0.6, scoreMean=2.5, pv=["D4"])
        ]))
        emit(frame(query, True, visits=25, candidates=[
            dict(move="D4", order=1, visits=20, winrate=0.6, scoreMean=2.5, pv=["D4"]),
            dict(move="E5", order=0, visits=5, winrate=0.6, scoreMean=2.5, pv=["E5"])
        ]))
        return
    if (root / "hold-deep").exists() and configured_visits(query) >= 500:
        while (root / "hold-deep").exists() and identity in active:
            emit(frame(query, True, visits=max(1, configured_visits(query) // 2)))
            time.sleep(query.get("reportDuringSearchEvery", 0.1))
        if identity not in active:
            return
        active.discard(identity)
        emit(frame(query))
        return
    while not (root / "finish").exists():
        if identity not in active:
            return
        emit(frame(query, True, visits=max(1, configured_visits(query) // 2)))
        time.sleep(query.get("reportDuringSearchEvery", 0.1))
    if identity in active:
        active.discard(identity)
        emit(frame(query))

first = True
task_count = 0
last_task_query = None
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
        queries[query["id"]] = query
        is_task = ":" in query["id"]
        mode = (root / "budget-smoke").read_text().strip() if (root / "budget-smoke").exists() else ""
        if is_task:
            task_count += 1
            query["fixtureIndex"] = task_count
        if is_task and mode == "natural_race" and task_count % 2 == 1:
            emit(frame(query, True, visits=8))
            emit(frame(query, visits=8))
            continue
        if is_task and mode == "ignored_setting":
            emit(dict(id=query["id"], warning="setting ignored", field="overrideSettings.maxTime"))
            active.add(query["id"])
            threading.Thread(target=search, args=(query,), daemon=True).start()
            continue
        if is_task and mode == "duplicate_late":
            if last_task_query is not None:
                emit(frame(last_task_query, visits=999))
            emit(frame(query, visits=8))
            emit(frame(query, visits=999))
            last_task_query = query
        elif is_task and task_count == 1 and not (root / "hold-first").exists() and not mode:
            emit(frame(query))
        else:
            active.add(query["id"])
            threading.Thread(target=search, args=(query,), daemon=True).start()
            if is_task and (root / "close-input").exists():
                os.close(0)
                (root / "input-closed").touch()
                while True:
                    time.sleep(0.05)
