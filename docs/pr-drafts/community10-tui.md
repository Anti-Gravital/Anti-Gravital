## Resumen

Real-terminal PTY audit of the interactive ag interface: harness, raw transcripts and evidence (wave 10)

## Fase afectada

Quality assurance of the existing RFC-0018 interactive interface. No roadmap phase advanced and no code behaviour changed.

## Tipo de cambio

- [ ] Feature
- [ ] Fix
- [ ] Refactor
- [x] Documentation and audit evidence
- [ ] Breaking change
- [ ] CI / tooling

## Documentos relacionados

- RFC-0018 (interactive interface principles and stages B+C)
- `docs/audits/tui-user-journey.md` (captured screens and key sequences)
- `docs/audits/tui-transcripts/` (17 raw JSON transcripts plus summary)
- `tools/tui-pty-harness.py` (standard-library PTY harness)
- Issue #200 (truncated dns help text, corroborated, not closed)

## Plan de prueba

```sh
cd worktrees10/tui
python3 -m py_compile tools/tui-pty-harness.py
python3 tools/tui-pty-harness.py \
  --binary <path-to-ag> --out /tmp/opencode/w10-tui-repro
git diff --check
```

The harness ran 17 scenarios in a real PTY: every process exited without a timeout. Verify the captured screens and exit codes in `docs/audits/tui-transcripts/summary.json`.

## Criterios de salida que avanza

This change adds audit evidence only. It does not mark any `docs/roadmap/STATUS.md` gate as passed.

## Cierre de issues

Refs #200
Closes: none

## Checklist

- [x] Real commands executed and output captured, not simulated
- [x] Affected documentation updated in the same change
- [x] No fabricated verification or closed issues
- [x] Commit message and title within 256 characters, no emojis, no AI-tool evidence
- [x] No push, no merge, no shared code edits
