"""Real-pipe adversarial GTP process for lifecycle regression tests (no subprocesses)."""
import os
import pathlib
import sys
import time

mode, trace = sys.argv[1:]
pathlib.Path(trace + '.pid').write_text(str(os.getpid()))
commands = 'boardsize\nclear_board\nkomi\nplay\ngenmove\nquit\ntime_settings\ntime_left'
for line in sys.stdin:
    with open(trace, 'a') as log:
        log.write(line)
    if line.startswith('{'):
        raise SystemExit(91)
    ident, command = line.strip().split(' ', 1)
    if command == 'protocol_version':
        if mode == 'exit':
            raise SystemExit(7)
        if mode in ('hang', 'slow'):
            time.sleep(0.8 if mode == 'slow' else 60)
        if mode == 'dribble':
            sys.stdout.write('=' + ident + ' 2\n')
            sys.stdout.flush()
            while True:
                sys.stdout.write('x')
                sys.stdout.flush()
                time.sleep(0.01)
        if mode == 'flood':
            sys.stdout.write('=' + ident + ' 2\n' + ('x' * 100 + '\n') * 10000)
            sys.stdout.flush()
            time.sleep(60)
        if mode == 'oversize':
            sys.stdout.write('=' + ident + ' ' + 'x' * 100000)
            sys.stdout.flush()
            time.sleep(60)
        if mode == 'stderr':
            sys.stderr.write('?1 fake stderr response\n\n' + 'x' * 200000)
            sys.stderr.flush()
        if mode == 'badid':
            ident = '99'
        if mode == 'badframe':
            sys.stdout.write('banner\n\n')
            sys.stdout.flush()
            time.sleep(60)
        if mode == 'error':
            sys.stdout.write('?' + ident + ' refused\n\n')
            sys.stdout.flush()
            time.sleep(60)
    body = {'protocol_version': '1' if mode == 'v1' else '2', 'name': 'Fixture GTP',
            'version': '0.1', 'list_commands': commands.replace('genmove\n', '') if mode == 'missing' else commands}[command]
    response = '=' + ident + ' ' + body + '\n\n'
    if mode == 'fragment':
        for byte in response.replace('\n', '\r\n').encode():
            os.write(1, bytes([byte]))
            time.sleep(0.0002)
    else:
        sys.stdout.write(response)
        sys.stdout.flush()
    if command == 'list_commands' and mode in ('eof', 'unsolicited'):
        while not pathlib.Path(trace + '.idle-release').exists():
            time.sleep(0.005)
        if mode == 'eof':
            os.close(1)
        else:
            sys.stdout.write('=99 stale\n\n')
            sys.stdout.flush()
        time.sleep(60)
