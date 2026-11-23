# Verificacion de issue #215 — Scheduled security audit failures

Fecha: 2026-11-23 23:33 UTC
Rama: `community29/i215` (worktree aislado)
Issue: https://github.com/Anti-Gravital/Anti-Gravital/issues/215
Workflow: `.github/workflows/security-audit.yml` (RFC-0022 seccion 4.2)

## Dictamen

CHANGES_REQUESTED / needs-human.

El fallo reportado NO es transitorio de red ni de cache de advisories. Es un
error determinista de invocacion CLI: `cargo deny check` no acepta el flag
`--all-features`. Las dos comprobaciones de fondo (advisories, licencias,
fuentes, bans y cargo-audit) pasan con las invocaciones correctas.

- `cargo deny check --all-features` -> exit 2 (REPRODUCIDO, identico al issue).
- `cargo deny --offline check` -> exit 0: `advisories ok, bans ok, licenses ok, sources ok`.
- `cargo audit --no-fetch` -> exit 0: `warning: 4 allowed warnings found`.

La correccion definitiva exige editar `.github/workflows/security-audit.yml`,
que es ruta protegida (L-4). Este agente NO la ejecuta: se escala a humano.

## Causa raiz

`.github/workflows/security-audit.yml:44` invoca:

```
cargo deny check --all-features > "$RUNNER_TEMP/ag-deny.out" 2>&1 || deny_status=$?
```

`cargo-deny 0.20.2` (instalado con `cargo install cargo-deny --locked` en el
propio workflow) no define `--all-features` en el subcomando `check`. Al no
existir ese flag, el parser de clap termina con exit 2 antes de leer
`deny.toml` o `Cargo.lock`. Por tanto `deny_status=2` en CADA ejecucion
semanal, sin importar el estado real de las dependencias.

`tools/security-report.sh:67` ademas rotula el bloque con el texto
`cargo deny check --all-features`, que es lo que aparece en el cuerpo del
issue #215. El rotulo es correcto: refleja literalmente el comando roto.

`deny.toml:5` ya fija `[graph] all-features = false`. Si se quiere auditar el
grafo con todas las features, el mecanismo soportado es esa clave de
configuracion, no un flag de `cargo deny check`.

## Evidencia (salida real, sin secretos ni datos personales)

Versiones:

```
$ cargo --version
cargo 1.99.1 (5f94df478 2026-08-27)
$ cargo audit --version
cargo-audit-audit 0.22.2
$ cargo deny --version
cargo-deny 0.20.2
```

Reproduccion del fallo (idempotente, sin red):

```
$ cd <worktree> && cargo deny check --all-features
error: unexpected argument '--all-features' found

  tip: to pass '--all-features' as a value, use '-- --all-features'

Usage: cargo-deny check [OPTIONS] [WHICH]...

For more information, try '--help'.
EXIT_CODE=2
```

Opciones reales de `cargo deny check` (no figura `--all-features`):

```
$ cargo deny check --help
Usage: cargo-deny check [OPTIONS] [WHICH]...
Arguments:
  [WHICH]...  [possible values: advisories, bans, licenses, sources, all]
Options:
  -g, --graph <GRAPH>          --hide-inclusion-graph
      --audit-compatible-output  -s, --show-stats
  -W, --warn <WARN>            -A, --allow <ALLOW>
  -D, --deny <DENY>            --feature-depth <FEATURE_DEPTH>
  -h, --help
```

(Opcion global util para ejecuciones sin red: `cargo deny --offline`.)

Invocacion correcta, todos los cheques en verde:

```
$ cargo deny --offline check
... avisos warning[duplicate] por versiones multiples de crates windows-targets ...
warning[advisory-not-detected]: advisory was not encountered  (RUSTSEC-2023-0071)
warning[advisory-not-detected]: advisory was not encountered  (RUSTSEC-2024-0436)
warning[advisory-not-detected]: advisory was not encountered  (RUSTSEC-2026-0542)
warning[advisory-not-detected]: advisory was not encountered  (RUSTSEC-2026-0543)
advisories ok, bans ok, licenses ok, sources ok
EXIT_CODE=0
```

Los `warning[duplicate]` y `warning[advisory-not-detected]` estan configurados
como `warn` en `deny.toml` (`[bans] multiple-versions = "warn"`, ignores
documentados). No son fallos y no explican el exit 2 reportado.

cargo-audit en verde (todas las advisories detectadas estan en la lista
`ignore` documentada de `.cargo/audit.toml`):

```
$ cargo audit --no-fetch
      Loaded 1563 security advisories (from /home/angel-nereira/.cargo/advisory-db)
    Scanning Cargo.lock for vulnerabilities (787 crate dependencies)
... RUSTSEC-2026-0440, RUSTSEC-2026-0543, RUSTSEC-2026-0542, RUSTSEC-2024-0436 ...
warning: 4 allowed warnings found
EXIT_CODE=0
```

Cuerpo actual del issue #215 (GitHub, leido via `gh issue view 215`):

```
Last run: 2026-11-23 18:06:51 UTC
Commit: 7856af8f28031cb5055dad94b1307b1f6ba5cfa5
### cargo deny check --all-features (exit 2)
error: unexpected argument '--all-features' found
```

Estado: OPEN, label `security`, autor `app/github-actions`, sin comentarios.

## Impacto

- Falso positivo recurrente: el unico canal de la auditoria semanal senala un
  fallo de seguridad que no existe, con ruido cada lunes.
- El issue nunca se cerrara solo: `tools/security-report.sh` solo cierra cuando
  `AUDIT_STATUS=0` y `DENY_STATUS=0`, y `deny_status` queda fijo en 2.
- Riesgo de fatiga de alertas: un advisory real podria mezclarse con este ruido.

## Condicion exacta de cierre (para el coordinador)

El issue #215 se cierra automaticamente cuando `tools/security-report.sh` ve
ambos exit en 0 en una ejecucion del workflow `security-audit`. Se requiere,
por un humano con permiso sobre `.github/`:

1. `.github/workflows/security-audit.yml:44`: sustituir
   `cargo deny check --all-features` por `cargo deny check`.
2. Opcional, solo si se desea cubrir todas las features:
   `deny.toml:5` -> `all-features = true` (cargo-deny no acepta el flag CLI).
3. `tools/security-report.sh:67`: actualizar el rotulo a `cargo deny check`
   para que el issue refleje el comando real.
4. Reimpactar el workflow (`workflow_dispatch`) y confirmar
   `cargo audit exit: 0` / `cargo deny exit: 0`; el script comenta y cierra
   el issue con `reason completed`.

Esta verificacion NO cierra el issue. No se edita `.github/` (L-4).

## Propuesta de comentario para el issue #215 (coordinador)

> Reproducible: `cargo deny check --all-features` exits 2 (unsupported flag).
> Correct `cargo deny check` + `cargo audit` pass. Fix needs
> .github/workflows/security-audit.yml (L-4 protected): escalated needs-human.
> Close after workflow fix + green weekly run.
