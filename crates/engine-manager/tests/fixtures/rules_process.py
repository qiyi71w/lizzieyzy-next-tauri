import os
import pathlib
import sys
import time

mode = pathlib.Path("mode").read_text()
commands = "protocol_version name version list_commands boardsize clear_board komi play genmove quit loadsgf showboard printsgf kata-get-rules kata-set-rules get_komi".split()
text = ""
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
