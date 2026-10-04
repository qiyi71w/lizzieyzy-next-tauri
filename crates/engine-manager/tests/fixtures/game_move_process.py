"""Adversarial real-pipe fixture for the public manager move operation."""
import json
import os
from pathlib import Path
import sys
import time

mode = Path('mode').read_text()
Path('pid').write_text(str(os.getpid()))
active = {}
first = True
rejected = False
move_count = 0
for raw in sys.stdin:
    with open('trace', 'a') as trace:
        trace.write(raw)
    if mode == 'readiness_fail':
        raise SystemExit(3)
    if mode == 'readiness_hold':
        time.sleep(60)
    if raw.startswith('{'):
        request = json.loads(raw)
        ident = request['id']
        if request.get('action') == 'terminate':
            target = request['terminateId']
            time.sleep(0.08)
            if mode != 'unclean' and target in active:
                print(json.dumps({'id': target, 'turnNumber': active.pop(target),
                                  'isDuringSearch': False, 'noResults': True}), flush=True)
            continue
        turn = len(request.get('moves', []))
        active[ident] = turn
        if first:
            first = False
            Path('run-' + ident.removeprefix('lifecycle-readiness-')).write_text(str(os.getpid()))
            if mode == 'second_readiness_fail':
                remaining = int(Path('remaining-ready').read_text())
                if remaining == 0:
                    raise SystemExit(3)
                Path('remaining-ready').write_text(str(remaining - 1))
            print(json.dumps({'id': ident, 'turnNumber': 0}), flush=True)
            continue
        if mode in ('error', 'errors') and not rejected:
            rejected = True
            active.pop(ident)
            print(json.dumps({'id': ident, mode: 'query rejected'}), flush=True)
            continue
        move_count += 1
        if mode == 'pause_once' and move_count == 1:
            continue
        if mode == 'exit':
            raise SystemExit(3)
        if mode in ('hold', 'unclean'):
            continue
        result = {'id': ident, 'turnNumber': turn, 'isDuringSearch': False,
                  'moveInfos': [{'move': 'A1', 'order': 1, 'visits': 1000},
                                {'move': 'D4', 'order': 0, 'visits': 1}]}
        if 'reportDuringSearchEvery' in request:
            result['rootInfo'] = {'visits': request.get('maxVisits', 32), 'winrate': 0.6,
                                  'scoreLead': 1.5}
            for candidate in result['moveInfos']:
                candidate.update(winrate=0.6, scoreMean=1.5, pv=[candidate['move']])
            if request.get('includeOwnership'):
                result['ownership'] = [0.25] * (request['boardXSize'] * request['boardYSize'])
            for visits in range(1, 21 if mode == 'analysis_flood' else 2):
                intermediate = dict(result, isDuringSearch=True,
                                    rootInfo=dict(result['rootInfo'], visits=visits))
                print(json.dumps(intermediate), flush=True)
                time.sleep(0.005)
            if mode == 'analysis_hold':
                continue
            if mode == 'analysis_invalid_accounting':
                result['rootInfo']['visits'] = 0
        if mode == 'warning':
            print(json.dumps({'id': ident, 'warning': 'rules changed'}), flush=True)
        if mode == 'duplicate':
            result['moveInfos'][0]['order'] = 0
        if mode == 'missing':
            result['moveInfos'][1]['order'] = 1
        if mode == 'incomplete':
            result.pop('isDuringSearch')
        if mode == 'wrongturn':
            result['turnNumber'] += 1
        if mode == 'wrongid':
            result['id'] = 'retired-query'
        if mode == 'occupied':
            result['moveInfos'][1]['move'] = 'A5'
        if mode in ('pass', 'resign'):
            result['moveInfos'][1]['move'] = mode
        print(json.dumps(result), flush=True)
        continue
    ident, command = raw.strip().split(' ', 1)
    commands = ['boardsize', 'clear_board', 'komi', 'play', 'genmove', 'quit', 'time_settings', 'time_left']
    if mode == 'no_time_left':
        commands.remove('time_left')
    if mode == 'no_time_settings':
        commands.remove('time_settings')
    body = {'protocol_version': '2', 'name': 'GNU Go' if mode != 'identity' else 'Unqualified',
            'version': '3.8', 'list_commands': '\n'.join(commands)}.get(command, '')
    if mode == 'slow_sync' and command.startswith(('boardsize ', 'clear_board', 'komi ', 'play ')):
        time.sleep(0.35)
    if mode == 'shrinking' and command.startswith('time_settings '):
        time.sleep(0.65)
    if command.startswith('genmove '):
        if mode == 'hold':
            time.sleep(60)
        if mode == 'exit':
            raise SystemExit(3)
        body = {'pass': 'pass', 'resign': 'resign', 'malformed': 'I5',
                'occupied': 'A5'}.get(mode, 'D4')
    if command == 'quit':
        raise SystemExit(0)
    print('=' + ident + ' ' + body + '\n', flush=True)
