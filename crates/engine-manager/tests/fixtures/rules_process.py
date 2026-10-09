import os
import pathlib
import sys
import time

mode = pathlib.Path("mode").read_text()
commands = "protocol_version name version list_commands boardsize clear_board komi play genmove quit loadsgf showboard printsgf kata-get-rules kata-set-rules get_komi".split()
if mode.startswith("analysis"):
    commands += ["kata-analyze", "stop"]
commands += ["kata-set-param", "kata-get-param"]
threads = 1
pathlib.Path("pid").write_text(str(os.getpid()))
stream_count = 0
def analysis_frame(winrate):
    return f"info move D4 visits 32 winrate {winrate} order 0 pv D4 E5 rootInfo visits 32 winrate {winrate}"
text = ""
pair_first = None
for raw in sys.stdin:
    with open("trace", "a") as trace:
        trace.write(raw)
    number, command = raw.strip().split(" ", 1)
    name = command.split()[0]
    body = ""
    marker = "="
    if name == "protocol_version": body = "2"
    elif name == "name": body = "KataGo"
    elif name == "version": body = "1.18.2"
    elif name == "list_commands": body = "\n".join(commands)
    elif name == "quit": break
    elif name == "kata-set-param":
        value = int(command.split()[2])
        if value < 1 or value > 4096: marker, body = "?", "invalid threads"
        else: threads = value
    elif name == "kata-get-param":
        parameter = command.split()[1]
        if parameter in ("playoutDoublingAdvantage", "analysisWideRootNoise"):
            behavior = pathlib.Path("pair-mode").read_text() if pathlib.Path("pair-mode").exists() else "forward"
            if parameter == "playoutDoublingAdvantage":
                pair_first = number
                continue
            if behavior == "closed_stdout":
                pathlib.Path("pair-held").write_text(number)
                while not pathlib.Path("pair-release").exists(): time.sleep(0.005)
                os.close(1)
                while True: time.sleep(1)
            pda, wrn = "-0.5", "0.04"
            if behavior == "nan": pda = "NaN"
            if behavior == "infinity": wrn = "inf"
            if behavior == "pda_domain": pda = "3.1"
            if behavior == "wrn_domain": wrn = "-0.01"
            if behavior == "extra_value": wrn = "0.04 0.05"
            if behavior == "wrong_order":
                print(f"={pair_first} {pda}\n={number} {wrn}\n", flush=True)
                continue
            if behavior == "stream_only":
                print("info move D4 visits 1 winrate 0.5", flush=True)
                continue
            replies = [(pair_first, pda, "?" if behavior == "error_first" else "="),
                       (number, wrn, "?" if behavior == "error_second" else "=")]
            if behavior in ("hold", "timeout"):
                print(f"={pair_first} 1.5\n", flush=True)
                pathlib.Path("pair-held").write_text(number)
                while not pathlib.Path("pair-release").exists(): time.sleep(0.005)
                replies = [(number, "0.25", "=")]
            if behavior == "missing": replies = replies[:1]
            if behavior == "reverse": replies.reverse()
            for response_id, value, reply_marker in replies:
                if behavior == "old_ids": response_id = str(int(response_id) - 2)
                print(f"{reply_marker}{response_id} {value}\n", flush=True)
            continue
        behavior = pathlib.Path("thread-mode").read_text() if pathlib.Path("thread-mode").exists() else "valid"
        if behavior in ("hold", "timeout"):
            pathlib.Path("thread-held").write_text(number)
            while not pathlib.Path("thread-release").exists(): time.sleep(0.005)
        if behavior == "negative": marker, body = "?", "read failed"
        elif behavior == "mismatch": body = "4096"
        elif behavior == "invalid": body = "NaN"
        else: body = str(threads)
    elif name == "kata-analyze":
        stream_count += 1
        print(f"={number}", flush=True)
        if mode == "analysis_invalid":
            print("info move D4 visits 2 winrate NaN", flush=True)
        else:
            print(analysis_frame(0.6 + stream_count / 100), flush=True)
        if mode == "analysis_closed_stdout":
            pathlib.Path("held").write_text(number)
            while not pathlib.Path("release").exists(): time.sleep(0.005)
            os.close(1)
            while True: time.sleep(1)
        continue
    elif name == "stop":
        print(analysis_frame(0.01), flush=True)
        print("", flush=True)
        if mode == "analysis_hold_stop":
            print(f"={int(number)-20}\n", flush=True)
            pathlib.Path("held").write_text(number)
            while not pathlib.Path("release").exists(): time.sleep(0.005)
        if mode == "analysis_negative_stop": marker, body = "?", "cannot stop"
    elif name == "loadsgf":
        if mode == "negative": marker, body = "?", "rejected exact position"
        elif mode in ("hold", "stream_only"):
            pathlib.Path("held").write_text(number)
            if mode == "stream_only":
                print("info move D4 visits 10 winrate 0.5 pv D4", flush=True)
            while not pathlib.Path("release").exists(): time.sleep(0.005)
        elif mode == "exit": sys.exit(3)
        text = pathlib.Path(command.split()[1]).read_text()
        if mode == "old_ack": number = str(int(number) - 1)
    elif name == "kata-get-rules":
        body = '{"friendlyPassOk":true,"hasButton":false,"ko":"SIMPLE","scoring":"AREA","suicide":false,"tax":"NONE","whiteHandicapBonus":"N"}'
        if mode == "wrong_rules": body = body.replace("SIMPLE", "POSITIONAL")
    elif name == "get_komi": body = "7.5"
    elif name == "showboard":
        body = "MoveNum: 0\n   A B C D E F G H J\n" + "\n".join(f" {row} . . . . . . . . ." for row in range(9, 0, -1)) + "\nNext player: Black"
    elif name == "printsgf": body = text
    if mode == "stream" and name not in ("protocol_version", "name", "version", "list_commands"):
        print("info move D4 visits 10 winrate 0.5 pv D4", flush=True)
    print(f"{marker}{number} {body}\n", flush=True)
    if mode == "send_failure" and name == "list_commands":
        os.close(0)
        pathlib.Path("held").write_text("stdin closed")
        while True: time.sleep(1)
