/**
 * The tmux engine against a scratch tmux server (spec section 5, "Oracle and
 * tests"). Starts `tests/fixtures/terminal-apps/nested-boxes.sh` in a detached
 * server on its own socket, points the engine at that server through an
 * IMPECCABLE_TMUX wrapper that adds `-L`, and checks the findings and the
 * restore. Skips without a binary or without tmux. The server is killed in
 * after(); the runner's process-group guard covers a SIGKILLed test process.
 */
import { after, before, describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { ENGINE_MISSING_MESSAGE, engineEnv, findEngineBinary } from './lib/engine-bin.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const BIN = findEngineBinary();
const TMUX_OK = spawnSync('tmux', ['-V'], { encoding: 'utf8' }).status === 0;
const SOCKET = `impeccable-test-${process.pid}`;
const APP = path.join(ROOT, 'tests', 'fixtures', 'terminal-apps', 'nested-boxes.sh');

const tmux = (...args) => execFileSync('tmux', ['-L', SOCKET, ...args], { encoding: 'utf8' }).trim();
let wrapper;

function detect(...args) {
  return spawnSync(BIN, ['detect', '--no-config', ...args], {
    encoding: 'utf8',
    env: engineEnv(BIN, { IMPECCABLE_TMUX: wrapper }),
    timeout: 30_000,
  });
}

describe('tmux engine against a scratch server', { skip: !BIN ? ENGINE_MISSING_MESSAGE : !TMUX_OK ? 'tmux is not installed' : false }, () => {
  before(() => {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'impeccable-tmux-'));
    wrapper = path.join(dir, 'tmux');
    fs.writeFileSync(wrapper, `#!/bin/sh\nexec tmux -L ${SOCKET} "$@"\n`, { mode: 0o755 });
    tmux('new-session', '-d', '-x', '80', '-y', '24', '-s', 'app', `sh ${APP}`);
    spawnSync('sleep', ['0.5']);
  });

  after(() => {
    try { tmux('kill-server'); } catch { /* already gone */ }
    if (wrapper) fs.rmSync(path.dirname(wrapper), { recursive: true, force: true });
  });

  it('captures the pane, walks the size matrix, and restores the window', () => {
    const r = detect('--json', '--tmux', 'app:0.0', '--tmux-sizes', '40x24');
    assert.equal(r.status, 0, r.stderr);
    const findings = JSON.parse(r.stdout);
    const ids = new Set(findings.map((f) => f.antipattern));
    for (const id of ['tui-rt-nested-borders', 'tui-rt-low-contrast', 'tui-rt-no-key-hints', 'tui-rt-spinner-never-rests']) {
      assert.ok(ids.has(id), `${id} missing from ${[...ids].join(', ')}`);
    }
    assert.ok(findings.some((f) => f.antipattern === 'tui-rt-collapse-narrow' && f.frame === '40x24'), 'the 40-column frame collapses');
    assert.ok(findings.every((f) => f.file === 'tmux:app:0.0' && f.advisory === true));
    assert.equal(tmux('display', '-p', '-t', 'app:0.0', '#{window_width}x#{window_height} #{window-size}'), '80x24 latest', 'window size and mode restored');
  });

  it('puts back a window-size the user had set before the scan', () => {
    tmux('set-option', '-w', '-t', 'app:0.0', 'window-size', 'largest');
    try {
      const r = detect('--json', '--tmux', 'app:0.0', '--tmux-sizes', '40x24');
      assert.equal(r.status, 0, r.stderr);
      assert.equal(tmux('display', '-p', '-t', 'app:0.0', '#{window_width}x#{window_height} #{window-size}'), '80x24 largest', 'size and the user value restored');
    } finally {
      tmux('set-option', '-w', '-t', 'app:0.0', '-u', 'window-size');
    }
  });

  it('restores the window and fails loudly when SIGINT lands during the size pass', async () => {
    const child = spawn(BIN, ['detect', '--no-config', '--tmux', 'app:0.0', '--tmux-sizes', '40x24', '--tmux-settle', '5000'], {
      env: engineEnv(BIN, { IMPECCABLE_TMUX: wrapper }),
    });
    let stderr = '';
    child.stderr.setEncoding('utf8').on('data', (d) => { stderr += d; });
    const exited = new Promise((resolve) => child.on('exit', (code, signal) => resolve({ code, signal })));
    // Frame 0, the recaptures (1000 ms), show-options, and the resize take
    // about 1.1 s, so 2.5 s lands inside the 5 s settle. Read the width
    // before signalling, so a failed assertion never leaves the child running.
    await new Promise((resolve) => setTimeout(resolve, 2500));
    const midSettleWidth = tmux('display', '-p', '-t', 'app:0.0', '#{window_width}');
    child.kill('SIGINT');
    const { code, signal } = await exited;
    assert.equal(midSettleWidth, '40', 'the scan was mid-settle at 40 columns when SIGINT arrived');
    assert.equal(signal, null, 'the engine caught SIGINT instead of dying');
    assert.equal(code, 1, stderr);
    assert.match(stderr, /^Error: interrupted; window restored$/m);
    assert.equal(tmux('display', '-p', '-t', 'app:0.0', '#{window_width}x#{window_height} #{window-size}'), '80x24 latest', 'restored after the interrupt');
  });

  it('reports an unknown pane as an operational failure and touches nothing', () => {
    const r = detect('--tmux', 'nope:0.0');
    assert.equal(r.status, 1);
    assert.match(r.stderr, /^Error: tmux list-panes: can.t find session: nope/);
    assert.equal(r.stdout, '');
  });
});
