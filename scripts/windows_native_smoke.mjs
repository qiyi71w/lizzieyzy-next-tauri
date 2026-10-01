import assert from 'node:assert/strict';
import { spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, openSync, closeSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'node:net';
import { setTimeout as delay } from 'node:timers/promises';

const args = process.argv.slice(2);
if (args.includes('--help')) {
  console.log('Usage: npm run desktop:smoke:windows -- --candidate <candidate.json> --run-directory <fresh-directory>\nWindows or WSL; Node 22+. Uses prepared candidate helpers, an isolated Win32 desktop and real WebView2/native Open/Save. No global input or clipboard.');
  process.exit(0);
}
const options = {};
for (let i = 0; i < args.length; i += 2) {
  assert(['--candidate', '--run-directory'].includes(args[i]) && args[i + 1], 'Expected --candidate and --run-directory');
  options[args[i].slice(2)] = args[i + 1];
}
assert(options.candidate && options['run-directory'], 'Use --help for required arguments');
if (process.platform !== 'win32') {
  const windowsPath = (path) => {
    const result = spawnSync('wslpath', ['-w', resolve(path)], { encoding: 'utf8' });
    assert.equal(result.status, 0, result.stderr || 'Windows smoke requires Windows or WSL');
    return result.stdout.trim();
  };
  const result = spawnSync('node.exe', [windowsPath(fileURLToPath(import.meta.url)), '--candidate', windowsPath(options.candidate), '--run-directory', windowsPath(options['run-directory'])], { stdio: 'inherit', timeout: 360000 });
  if (result.error) console.error(result.error.message);
  process.exit(result.status ?? 1);
}
assert(typeof WebSocket === 'function', 'Node 22+ is required');
const candidateFile = resolve(options.candidate);
const candidate = JSON.parse(readFileSync(candidateFile, 'utf8'));
assert.equal(candidate.project, 'lizzieyzy-next-tauri');
assert.equal(candidate.status, 'BUILT');
const tools = join(dirname(candidateFile), '.acceptance-tools');
for (const name of ['windows-desktop-session.ps1', 'windows-native-action.ps1', 'windows-desktop-isolation.ps1']) assert(existsSync(join(tools, name)), `Re-run prepare-windows-candidate: missing ${name}`);
assert(candidate.application_id?.startsWith('org.lizzieyzy.next.acceptance.'), 'Expected an isolated acceptance application ID');
const profileOwner = join(tools, 'native-smoke-profile.json');
if (existsSync(profileOwner)) {
  assert.equal(JSON.parse(readFileSync(profileOwner, 'utf8')).application_id, candidate.application_id, 'Smoke profile ownership changed');
} else {
  assert(!existsSync(join(process.env.APPDATA, candidate.application_id)), 'Candidate has pre-existing app data; prepare a fresh dedicated smoke candidate. Existing recovery state will not be discarded.');
  writeFileSync(profileOwner, JSON.stringify({ application_id: candidate.application_id }) + '\n', { flag: 'wx' });
}
const runDirectory = resolve(options['run-directory']);
assert(!existsSync(runDirectory), 'Run directory must be fresh');
mkdirSync(dirname(runDirectory), { recursive: true });
const runFile = join(runDirectory, 'run.json');
const report = { status: 'BLOCKED', candidate: candidate.candidate, candidate_file: candidateFile, automation_script: fileURLToPath(import.meta.url), cases: [], human_actions: [], not_covered: ['physical DPI/monitor changes', 'OS drag/drop and clipboard permissions', 'GPU, audio, engine lifecycle, release packaging'] };
const readRun = () => JSON.parse(readFileSync(runFile, 'utf8'));
const psArgs = (script, extra) => ['-NoLogo', '-NoProfile', '-NonInteractive', '-File', join(tools, script), ...extra];
function ps(script, extra) {
  const result = spawnSync('pwsh.exe', psArgs(script, extra), { encoding: 'utf8', timeout: 35000 });
  assert.equal(result.status, 0, result.error?.message || result.stderr || result.stdout);
  return result.stdout;
}
async function until(label, predicate, milliseconds = 15000) {
  const deadline = Date.now() + milliseconds;
  while (Date.now() < deadline) {
    const value = await predicate();
    if (value) return value;
    await delay(150);
  }
  throw new Error(`Timed out: ${label}`);
}
const port = await new Promise((resolvePort, reject) => {
  const server = createServer();
  server.on('error', reject);
  server.listen(0, '127.0.0.1', () => { const address = server.address(); server.close(() => resolvePort(address.port)); });
});
let socket;
let nextId = 0;
const pending = new Map();
function cdp(method, params = {}) {
  const id = ++nextId;
  return new Promise((resolveCall, reject) => {
    const timer = setTimeout(() => { pending.delete(id); reject(new Error(`CDP timed out: ${method}`)); }, 15000);
    pending.set(id, { resolve: resolveCall, reject, timer });
    socket.send(JSON.stringify({ id, method, params }));
  });
}
async function evaluate(expression) {
  const result = await cdp('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
  assert(!result.exceptionDetails, JSON.stringify(result.exceptionDetails));
  return result.result.value;
}
async function click(label) {
  await until(`actionable control: ${label}`, () => evaluate(`(() => { const e = [...document.querySelectorAll('button')].find(e => e.getAttribute('aria-label') === ${JSON.stringify(label)} || e.title === ${JSON.stringify(label)} || e.textContent.trim() === ${JSON.stringify(label)}); if (!e || e.disabled || !e.getClientRects().length) return false; e.click(); return true; })()`));
}
const serialize = () => evaluate("window.__TAURI_INTERNALS__.invoke('serialize_current_game')");
async function nativeDialog(name, kind, path, cancel = false) {
  const evidence = join(runDirectory, `${name}.json`);
  ps('windows-native-action.ps1', ['-RunFile', runFile, '-Action', 'FileDialog', '-DialogKind', kind, '-EvidencePath', evidence, '-TimeoutSeconds', '25', ...(cancel ? ['-Cancel'] : ['-FilePath', path])]);
  assert.equal(JSON.parse(readFileSync(evidence, 'utf8')).status, 'ACTION_SENT');
}
async function screenshot(name) {
  const shot = await cdp('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false });
  writeFileSync(join(runDirectory, `${name}.png`), Buffer.from(shot.data, 'base64'), { flag: 'wx' });
}
const logFile = `${runDirectory}.supervisor.log`;
const log = openSync(logFile, 'wx');
const supervisor = spawn('pwsh.exe', psArgs('windows-desktop-session.ps1', ['-Action', 'Run', '-CandidateFile', candidateFile, '-RunDirectory', runDirectory, '-CdpPort', String(port), '-DesktopMode', 'Isolated']), { stdio: ['ignore', log, log], windowsHide: true });
closeSync(log);
let supervisorError;
supervisor.on('error', error => { supervisorError = error; });
try {
  const run = await until('isolated application readiness', () => {
    if (supervisorError) throw supervisorError;
    if (supervisor.exitCode !== null) throw new Error(`Supervisor exited: ${supervisor.exitCode}; ${logFile}`);
    if (!existsSync(runFile)) return false;
    const current = readRun();
    if (current.status === 'BLOCKED') throw new Error(current.error);
    return current.status === 'READY' && current;
  }, 60000);
  assert.equal(run.desktop_mode, 'isolated');
  assert.match(run.desktop_name, /^weiqi-[a-f0-9]{32}$/);
  report.run_identity = run;
  const status = JSON.parse(ps('windows-desktop-session.ps1', ['-Action', 'Status', '-RunDirectory', runDirectory]));
  assert.equal(status.status, 'READY');
  assert.equal(status.pid, run.process.pid);
  const targets = await until('WebView2 debugging target', async () => {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/json/list`, { signal: AbortSignal.timeout(2000) });
      const pages = (await response.json()).filter(target => target.type === 'page' && target.webSocketDebuggerUrl);
      return pages.length ? pages : false;
    } catch { return false; }
  });
  assert.equal(targets.length, 1, 'Expected exactly one WebView2 page for the candidate');
  socket = new WebSocket(targets[0].webSocketDebuggerUrl);
  socket.addEventListener('message', event => {
    const message = JSON.parse(event.data);
    const call = pending.get(message.id);
    if (!call) return;
    clearTimeout(call.timer); pending.delete(message.id);
    if (message.error) call.reject(new Error(JSON.stringify(message.error))); else call.resolve(message.result);
  });
  await new Promise((resolveOpen, reject) => {
    const timer = setTimeout(() => reject(new Error('WebView2 connection timed out')), 15000);
    socket.addEventListener('open', () => { clearTimeout(timer); resolveOpen(); }, { once: true });
    socket.addEventListener('error', () => { clearTimeout(timer); reject(new Error('WebView2 connection failed')); }, { once: true });
  });
  await until('native board rendering', () => evaluate("Boolean(window.__TAURI_INTERNALS__ && document.querySelector('canvas[aria-label=\"棋盘\"]')?.width > 0 && document.querySelector('button[aria-label=\"打开\"]'))"));
  const startup = await until('ready document or owned smoke recovery', () => evaluate(`document.querySelector('[role="dialog"][aria-label="恢复当前棋谱"]') ? 'recovery' : (document.querySelector('button[aria-label="打开"]')?.disabled === false ? 'ready' : false)`));
  if (startup === 'recovery') {
    await click('恢复');
    await until('owned smoke recovery completed', () => evaluate(`!document.querySelector('[role="dialog"][aria-label="恢复当前棋谱"]')`));
    report.restored_owned_smoke_session = true;
  }
  report.cases.push({ name: 'native-startup-and-identity', status: 'PASS' });
  const fixture = join(runDirectory, '矩形棋盘 smoke.sgf');
  writeFileSync(fixture, '(;FF[4]GM[1]CA[UTF-8]SZ[5:4]KM[0.5]PB[Smoke Black]PW[Smoke White];B[aa];W[bb];B[cc];W[dd];B[])\n', { flag: 'wx' });
  await click('打开');
  await nativeDialog('open-fixture', 'Open', fixture);
  await until('imported fixture', () => evaluate("document.querySelector('.doc-name')?.textContent.includes('矩形棋盘 smoke.sgf')"));
  const projection = await evaluate("window.__TAURI_INTERNALS__.invoke('project_current_game_mainline')");
  assert.equal(projection.summary.board_width, 5); assert.equal(projection.summary.board_height, 4); assert.equal(projection.summary.move_count, 5);
  if (await evaluate("document.querySelector('input[aria-label=\"跳转手数\"]')?.value !== '0'")) await click('首手');
  await until('first move navigation', () => evaluate("document.querySelector('input[aria-label=\"跳转手数\"]')?.value === '0'"));
  await click('末手');
  await until('last move navigation', () => evaluate("document.querySelector('input[aria-label=\"跳转手数\"]')?.value === '5'"));
  const expected = await serialize();
  assert(expected.includes('B[]'), 'Pass move was lost');
  await screenshot('opened-last-move');
  report.cases.push({ name: 'native-open-and-navigation', status: 'PASS', board: [5, 4], moves: 5 });
  const initial = await serialize();
  await click('打开');
  await nativeDialog('open-cancel', 'Open', null, true);
  assert.equal(await serialize(), initial, 'Cancelling Open changed the document');
  report.cases.push({ name: 'native-open-cancel', status: 'PASS' });
  const saved = join(runDirectory, '另存 smoke.sgf');
  await click('文件'); await click('另存为(S)');
  await nativeDialog('save-as', 'SaveAs', saved);
  await until('saved document identity', () => evaluate("document.querySelector('.doc-name')?.textContent.trim() === '另存 smoke.sgf'"));
  assert.equal(readFileSync(saved, 'utf8').trim(), expected.trim(), 'Native Save As changed SGF content');
  const replacement = join(runDirectory, 'empty-before-reopen.sgf');
  writeFileSync(replacement, '(;FF[4]GM[1]SZ[9])\n', { flag: 'wx' });
  await click('打开'); await nativeDialog('replace-before-reopen', 'Open', replacement);
  await until('replacement document', () => evaluate("document.querySelector('.doc-name')?.textContent.trim() === 'empty-before-reopen.sgf'"));
  const empty = await evaluate("window.__TAURI_INTERNALS__.invoke('project_current_game_mainline')");
  assert.equal(empty.summary.board_width, 9); assert.equal(empty.summary.move_count, 0);
  await click('打开'); await nativeDialog('reopen-saved', 'Open', saved);
  await until('reopened document identity', () => evaluate("document.querySelector('.doc-name')?.textContent.trim() === '另存 smoke.sgf'"));
  assert.equal(await serialize(), expected, 'Reopening saved SGF changed the document');
  await screenshot('reopened-saved');
  report.cases.push({ name: 'native-save-as-and-reopen', status: 'PASS' });
  socket.close(); socket = null;
  ps('windows-desktop-session.ps1', ['-Action', 'Stop', '-RunDirectory', runDirectory]);
  await until('clean supervisor exit', () => supervisor.exitCode !== null);
  assert.equal(supervisor.exitCode, 0);
  assert.equal(readRun().status, 'EXITED');
  report.cases.push({ name: 'clean-exit', status: 'PASS' });
  report.status = 'PASS';
} catch (error) {
  if (error.code === 'ERR_ASSERTION' && report.run_identity) report.status = 'FAIL';
  report.error = String(error.stack || error);
  if (socket?.readyState === WebSocket.OPEN) {
    try { await screenshot('failure'); }
    catch (screenshotError) { report.screenshot_error = String(screenshotError); }
  }
  process.exitCode = 1;
} finally {
  if (socket) socket.close();
  for (const call of pending.values()) clearTimeout(call.timer);
  if (existsSync(runFile)) {
    const run = readRun();
    if (run.process && run.status !== 'EXITED') {
      try { ps('windows-desktop-session.ps1', ['-Action', 'Stop', '-RunDirectory', runDirectory]); }
      catch {
        try { ps('windows-desktop-session.ps1', ['-Action', 'Stop', '-RunDirectory', runDirectory, '-Force']); }
        catch (error) { report.cleanup_error = String(error); process.exitCode = 1; }
      }
    }
  }
  if (supervisor.exitCode === null && !supervisorError) {
    try { await until('supervisor cleanup', () => supervisor.exitCode !== null, 10000); }
    catch (error) { report.cleanup_error = String(error); supervisor.kill(); process.exitCode = 1; }
  }
  const reportPath = existsSync(runDirectory) ? join(runDirectory, 'smoke.json') : `${runDirectory}.smoke.json`;
  writeFileSync(reportPath, JSON.stringify(report, null, 2) + '\n', { flag: 'wx' });
  console.log(`${report.status} Windows isolated native smoke\nEvidence: ${reportPath}`);
  if (report.error) console.error(report.error);
}
