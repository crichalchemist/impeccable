import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('..', import.meta.url));

describe('skill reference authoring contracts', () => {
  it('keeps direction contracts in development-only surface briefs', () => {
    const newWork = readFileSync(join(ROOT, 'skill/reference/new-work.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const recordDecision = newWork.match(/## 5\. Record the decision\n([\s\S]*?)\n## 6\./)?.[1] ?? '';

    assert.match(recordDecision, /development-only contract/);
    assert.match(recordDecision, /under `## Direction contract` in the relevant surface brief/);
    assert.match(recordDecision, /read the brief once more/i);
    assert.match(recordDecision, /all six contract blocks and the seed key/);

    for (const block of ['THESIS', 'OWN-WORLD', 'STORY', 'FIRST VIEWPORT', 'FORM', 'FINISH']) {
      assert.match(recordDecision, new RegExp(`${block}:`));
    }

    for (const browserArtifact of [
      /HTML or framework comments/,
      /hidden DOM/,
      /<template>/,
      /`data-\*` attributes/,
      /serialized props or state/,
      /React Server Component payloads/,
      /client bundles/,
      /metadata or JSON-LD/,
      /accessibility-only text/,
    ]) {
      assert.match(recordDecision, browserArtifact);
    }

    assert.match(recordDecision, /Never copy the direction contract into implementation source or any browser-delivered artifact/);
    assert.doesNotMatch(newWork, /contract in the artifact's opening comment/);
    assert.doesNotMatch(newWork, /survives the production build/);
    assert.doesNotMatch(newWork, /grep the built output/);
    assert.doesNotMatch(newWork, /emitted markup/);
    assert.doesNotMatch(newWork, /first child of the document's body/);
  });

  it('keeps reduced-motion guidance on the animation build path', () => {
    const animate = readFileSync(join(ROOT, 'skill/reference/animate.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const accessibility = animate.match(/## Accessibility and control\n([\s\S]*?)\n## Verify/)?.[1] ?? '';
    const verify = animate.match(/## Verify\n([\s\S]*?)(?:\n## |$)/)?.[1] ?? '';

    assert.match(accessibility, /prefers-reduced-motion/);
    assert.match(accessibility, /intentional alternative/);
    assert.match(accessibility, /not disabling all motion/);
    assert.match(verify, /reduced[- ]motion/i);
  });

  it('uses an exact content fingerprint before inheriting a critique snapshot', () => {
    const critique = readFileSync(join(ROOT, 'skill/reference/critique.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const polish = readFileSync(join(ROOT, 'skill/reference/polish.md'), 'utf-8').replace(/\r\n?/g, '\n');

    assert.match(critique, /records an exact content fingerprint/);
    assert.match(polish, /compares the file's exact current content fingerprint/);
    assert.match(polish, /Unchanged staged, unstaged, or untracked content remains current/);
    assert.match(polish, /any byte change, deletion, or replacement with a non-file closes the backlog/);
    assert.match(polish, /latest "<resolved target>" --json/);
    assert.match(polish, /exact `snapshot_file` identity/);
    assert.match(polish, /close "<resolved target>" "<snapshot_file returned by latest>"/);
    assert.match(polish, /if a newer critique landed meanwhile, its backlog stays live/);
    assert.doesNotMatch(polish, /git status|git log/);
  });

  it('keeps touch-gesture verification in the adapt, audit, and harden references', () => {
    const adapt = readFileSync(join(ROOT, 'skill/reference/adapt.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const audit = readFileSync(join(ROOT, 'skill/reference/audit.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const harden = readFileSync(join(ROOT, 'skill/reference/harden.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const verifyAdaptations = adapt.match(/## Verify Adaptations\n([\s\S]*?)\n## /)?.[1] ?? '';
    const responsive = audit.match(/### 4\. Responsive Design\n([\s\S]*?)\n### 5\./)?.[1] ?? '';
    const edgeCases = harden.match(/### Edge Cases & Boundary Conditions\n([\s\S]*?)\n### /)?.[1] ?? '';
    const verifyHardening = harden.match(/## Verify Hardening\n([\s\S]*?)(?:\n## |$)/)?.[1] ?? '';

    assert.match(verifyAdaptations, /\*\*Primary gesture\*\*/);
    assert.match(verifyAdaptations, /produced the evidence/);
    assert.match(verifyAdaptations, /verify layout, never a gesture/);
    assert.match(verifyAdaptations, /reported gap, not a blocker/);
    assert.match(verifyAdaptations, /\*\*Scroll across it\*\*[\s\S]*without activating it/);
    assert.match(responsive, /\*\*Broken touch interaction\*\*/);
    assert.match(responsive, /what stayed untested/);
    assert.match(responsive, /Exercise the gesture when a browser tool can synthesize touch/);
    assert.match(edgeCases, /\*\*Interrupted gestures\*\*[\s\S]*works without a reload/);
    assert.match(edgeCases, /clear the dragging state and release capture/);
    assert.match(verifyHardening, /\*\*Interrupted gestures\*\*/);
  });

  it('gives terminal projects a reference shaped like the mobile ones', () => {
    const terminal = readFileSync(join(ROOT, 'skill/reference/terminal.md'), 'utf-8').replace(/\r\n?/g, '\n');
    for (const heading of [
      '## The terminal slop test',
      '## Layout & structure',
      '## Keyboard & input',
      '## Typography',
      '## Color',
      '## Components & controls',
      '## Motion',
      '## Verifying the build',
    ]) {
      assert.ok(terminal.includes(`\n${heading}\n`), `missing ${heading}`);
    }
    const bullets = terminal.split('\n').filter((l) => l.startsWith('- **'));
    assert.ok(bullets.length >= 25, `expected at least 25 bullets, found ${bullets.length}`);
    for (const b of bullets) assert.match(b, /<!-- rule:terminal-[a-z0-9-]+ -->$/, `bullet lacks a rule marker: ${b.slice(0, 60)}`);
    assert.match(terminal, /NO_COLOR/);
    assert.match(terminal, /tmux capture-pane/);
    assert.doesNotMatch(terminal, /—/, 'em dash');
  });

  it('keeps the audit report skeleton identical across the web, native, and terminal variants', () => {
    // The verdict heading names what each variant judges (implementation
    // integrity on the web, platform conformance elsewhere); every other
    // heading must match exactly.
    const skeleton = (name) => {
      const text = readFileSync(join(ROOT, `skill/reference/${name}`), 'utf-8').replace(/\r\n?/g, '\n');
      return text
        .split('\n')
        .filter((l) => /^#{2,3} /.test(l) && !/^### \d\./.test(l))
        .map((l) => (/^### .* Verdict$/.test(l) ? '### <verdict>' : l));
    };
    const web = skeleton('audit.md');
    assert.deepEqual(skeleton('audit.native.md'), web);
    assert.deepEqual(skeleton('audit.terminal.md'), web);
    const terminal = readFileSync(join(ROOT, 'skill/reference/audit.terminal.md'), 'utf-8');
    assert.equal((terminal.match(/^### \d\. /gm) || []).length, 5, 'five scored dimensions');
    assert.match(terminal, /\*\*\?\?\/20\*\*/);
    assert.match(terminal, /NO_COLOR/);
    const adapt = readFileSync(join(ROOT, 'skill/reference/adapt.terminal.md'), 'utf-8');
    assert.match(adapt, /under 60 columns/);
    assert.match(adapt, /120 columns and above/);
    assert.match(adapt, /16 colors/);
    assert.match(adapt, /truecolor/);
  });

  it('classifies every terminal width from narrow to wide with no gap', () => {
    // The engine's narrow cutoff (NARROW_COLUMNS = 60 in crates/terminal)
    // and the adapt table must agree, and no width between 40 and 120
    // columns may fall outside a class.
    const adapt = readFileSync(join(ROOT, 'skill/reference/adapt.terminal.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const terminal = readFileSync(join(ROOT, 'skill/reference/terminal.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const rows = adapt.split('\n').filter((l) => /^\| (Narrow|Compact|Standard|Wide) \|/.test(l));
    assert.deepEqual(rows.map((l) => l.split('|')[2].trim()), [
      'under 60 columns',
      '60 to 79 columns',
      '80 to 119 columns',
      '120 columns and above',
    ]);
    assert.match(adapt, /At 120 columns and above, text blocks stay near 80 to 100 cells/);
    assert.doesNotMatch(adapt, /Above 120 columns/);
    assert.doesNotMatch(adapt, /all three classes/);
    // The matrix probes 40 columns: every capture command passes 40x24.
    assert.match(terminal, /80x24, 120x40, and 40 columns;/);
    assert.doesNotMatch(terminal, /about 30 columns/);
  });

  it('routes terminal projects to the terminal references and keeps detect available to them', () => {
    const skill = readFileSync(join(ROOT, 'skill/SKILL.src.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const routing = readFileSync(join(ROOT, 'skill/reference/routing.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const init = readFileSync(join(ROOT, 'skill/reference/init.md'), 'utf-8').replace(/\r\n?/g, '\n');
    const audit = readFileSync(join(ROOT, 'skill/reference/audit.md'), 'utf-8');
    const adapt = readFileSync(join(ROOT, 'skill/reference/adapt.md'), 'utf-8');

    const auditRow = skill.split('\n').find((l) => l.startsWith('| `audit [target]`')) ?? '';
    const adaptRow = skill.split('\n').find((l) => l.startsWith('| `adapt [target]`')) ?? '';
    assert.match(auditRow, /terminal: \[reference\/audit\.terminal\.md\]\(reference\/audit\.terminal\.md\)/);
    assert.match(adaptRow, /terminal: \[reference\/adapt\.terminal\.md\]\(reference\/adapt\.terminal\.md\)/);
    assert.match(skill, /platform variant when the Commands table lists one/);
    assert.match(skill, /`ios`, `android`, `adaptive`, or `terminal`/);
    assert.match(skill, /Terminal surfaces usually resolve to Operate or Read/);
    assert.match(skill.split('\n')[2], /terminal UIs \(TUIs\)/);

    assert.match(routing, /`live` and `generate` are web-only/);
    assert.match(routing, /`terminal` projects keep `impeccable detect`/);
    assert.match(init, /`web`, `ios`, `android`, `adaptive`, or `terminal`/);
    assert.match(init, /\[terminal\.md\]\(terminal\.md\)/);
    assert.match(audit, /Terminal projects \(`terminal`\) route to \[audit\.terminal\.md\]/);
    assert.match(adapt, /Terminal projects \(`terminal`\) route to \[adapt\.terminal\.md\]/);

    const generate = readFileSync(join(ROOT, 'skill/reference/generate.md'), 'utf-8');
    assert.match(generate, /on `ios` \/ `android` \/ `adaptive` \/ `terminal` projects, decline this command/);

    // live.md carries the same guard as generate.md, near the top so it is
    // read before the setup steps.
    const liveHead = readFileSync(join(ROOT, 'skill/reference/live.md'), 'utf-8').replace(/\r\n?/g, '\n').split('\n').slice(0, 5).join('\n');
    assert.match(liveHead, /\*\*Web only\.\*\* Live mode's browser overlay has no native or terminal equivalent/);
    assert.match(liveHead, /on `ios` \/ `android` \/ `adaptive` \/ `terminal` projects, decline this command/);
  });

  it('gives terminal projects a branch wherever the skill splits web from native', () => {
    // Each file that names a web case and a native case must also name the
    // terminal case, or a terminal project falls through to the wrong one.
    const read = (p) => readFileSync(join(ROOT, p), 'utf-8').replace(/\r\n?/g, '\n');
    const branches = {
      'skill/SKILL.src.md': [
        /the shipped device classes on a native platform; the tmux capture matrix on a terminal\)/,
        /platform guidance for a native or terminal project when applicable/,
      ],
      'skill/reference/polish.md': [
        /on `terminal`, the tmux capture matrix from \[terminal\.md\]\(terminal\.md\)'s Verifying the build section/,
        /on terminal, the 80x24, 120x40, and 40x24 captures, a live resize/,
        /on terminal, a first frame within 100 ms, a terminal left as found after quit and after `Ctrl-C`/,
        /never add another detector pass beyond the one terminal run above/,
      ],
      'skill/reference/layout.md': [/^- \*\*Terminal:\*\* follow the Layout & structure section of \[terminal\.md\]\(terminal\.md\)/m],
      'skill/reference/typeset.md': [/^- \*\*Terminal:\*\* follow the Typography section of \[terminal\.md\]\(terminal\.md\)/m],
      'skill/reference/doctor.md': [/native build files or terminal dependencies/, /\[android\.md\]\(android\.md\), or \[terminal\.md\]\(terminal\.md\)/],
      'skill/reference/init.md': [
        /skip native, terminal, or non-runnable projects/,
        /the platform reference above is the one thing/,
        /the platform is not `terminal` \(a terminal surface always builds code-led/,
      ],
    };
    for (const [file, patterns] of Object.entries(branches)) {
      const text = read(file);
      for (const pattern of patterns) assert.match(text, pattern, `${file} has no terminal branch matching ${pattern}`);
    }
    assert.doesNotMatch(read('skill/reference/init.md'), /the native reference above/);
    assert.doesNotMatch(read('skill/SKILL.src.md'), /native-platform guidance/);
  });
});
