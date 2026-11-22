# Charter: secretos-config-tui

## Objetivo
Avanzar RFC-0020: archivo cifrado age tras SecretSource, keygen/recipients/set/unset, migracion de std::env::var y form TUI via RFC-0018, sin crate nuevo

## KPI verificable
0 valores de secreto impresos; round-trip y wrong-key en verde; fmt/clippy/test/audit/deny verdes; sin crate nuevo

## Restricciones
- Decisiones con fuentes verificadas: .kaizen/team/EVIDENCIA.md
- Vetos: security, architecture, qa, compliance, scope, resource
- Proactividad con frenos: .kaizen/team/FLUJOS.md
