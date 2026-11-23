## Resumen
Automate the crate dependency-direction guard (CLAUDE.md rule 15) as an opt-in, offline local check with no new dependency.

## Componente afectado
Tooling y documentacion de arquitectura; workspace de 19 crates (sin cambios de produccion).

## Tipo de cambio
- [x] enhancement
- [x] documentation

## Documentos relacionados
- `CLAUDE.md` reglas 14 y 15
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 5.3
- `docs/adr/0007-ag-mail-ag-domains.md`, `docs/adr/0012-ag-domains-control-plane.md`
- `docs/rfc/RFC-0011-ag-domains-control-plane.md`, `docs/rfc/RFC-0012-ag-workers-execution-model.md`
- `docs/audits/oportunidades-arquitectura.md` A5 (propuesta de origen)
- `docs/architecture/dependency-direction-guard.md` (nuevo)

## Diseno y justificacion
Se elige un script de bash y no un test de Rust porque la comprobacion es sobre texto de manifiestos: asi permite ejecutarse sin toolchain, sin red y sin anadir dependencias ni crates. Un test Rust exigiria compilar el workspace y tocar manifiestos de prueba para reutilizar `toml`. El script es determinista, falla cerrado y nombra el par de crates y la linea infractora. Es opt-in: no toca `.github/` ni `scripts/ag-check`.

El guarda afirma: (1) roster de 19 crates; (2) `ag-core` sin aristas `ag-*` salientes; (3) lista blanca de 16 pares derivada de documentos canonicos; (4) direcciones prohibidas por nombre (sexta regla, regla 2, RFC-0012, ADR-0012); (5) arista obligatoria `ag-edge -> ag-domains`; (6) grafo aciclico; (7) parseo falla cerrado. La autodependencia de test de `ag-mail` se excluye y se contabiliza por separado.

## Plan de prueba
Comandos ejecutados en la rama `community18/dep-guard`, worktree `worktrees18/dep-guard`:

- `bash -n scripts/check-dependency-direction.sh`
- `scripts/check-dependency-direction.sh`
- Copia scratch con 19 manifiestos y cinco mutaciones: `ag-mail -> ag-auth`, `ag-cache -> ag-ghost`, `ag-domains -> ag-edge`, declaracion `ag-*` no parseable en `ag-core`, y eliminacion de `ag-edge -> ag-domains`. Ademas `--help` y argumento desconocido.
- `git diff --check`
- `bash scripts/ag-check`
- Escaneo de contenido: `grep -rniE` de atribucion de herramientas IA, marcas retiradas, secretos y emojis sobre los archivos tocados.

## Evidencia (salida exacta)

### `bash -n` y guarda en el workspace real
Sin salida, `RC=0` para `bash -n`. El guarda devolvio:

```
PASS roster: 19 crates match CLAUDE.md rule 14
PASS directions: 17 internal edges reviewed, 1 self dev-dependency ignored
PASS required-edges: ag-edge -> ag-domains control plane present
PASS acyclic: no cycle among the 19 crates
dependency-direction: PASS (19 crates, 17 edges, 1 self dev-dependency ignored)
```

`RC=0`. Las 17 declaraciones incluyen `ag-auth -> ag-mail` en dependencias normales y de desarrollo; los 16 pares unicos estan en la lista blanca.

### Escenarios de fallo en copia scratch
Directorio temporal creado con `mktemp -d /tmp/opencode/dep-guard-scratch.XXXXXX`; el guarda se invoco desde `/tmp` para verificar que resuelve su propia raiz.

S0 (copia limpia): misma salida que el workspace real, `RC=0`.

S1, anadir `ag-auth` a `[dependencies]` de `ag-mail`:

```
PASS roster: 19 crates match CLAUDE.md rule 14
FAIL forbidden direction: ag-mail -> ag-auth at crates/ag-mail/Cargo.toml:58 [dependencies] (source: CLAUDE.md:348 y Arq-Tecnica 5.3 regla 6)
FAIL unlisted edge: ag-mail -> ag-auth at crates/ag-mail/Cargo.toml:58 [dependencies] (update the allow-list with an RFC/ADR citation; no silent new direction)
PASS required-edges: ag-edge -> ag-domains control plane present
FAIL cycle: ag-auth -> ag-mail -> ag-auth (CLAUDE.md:383, no circular dependencies)
dependency-direction: FAIL (3 violation(s))
```

`RC=1`. Ademas de la direccion prohibida, detecta el ciclo que la nueva arista crea con `ag-auth -> ag-mail`.

S2, anadir `ag-ghost` a `ag-cache`:

```
PASS roster: 19 crates match CLAUDE.md rule 14
FAIL unknown crate: ag-cache -> ag-ghost at crates/ag-cache/Cargo.toml:27 [dependencies] (target outside the 19-crate roster)
PASS required-edges: ag-edge -> ag-domains control plane present
PASS acyclic: no cycle among the 19 crates
dependency-direction: FAIL (1 violation(s))
```

`RC=1`.

S3, anadir `ag-edge` a `ag-domains`:

```
PASS roster: 19 crates match CLAUDE.md rule 14
FAIL forbidden direction: ag-domains -> ag-edge at crates/ag-domains/Cargo.toml:47 [dependencies] (source: ADR-0012 y docs/audits/oportunidades-arquitectura.md:24)
FAIL unlisted edge: ag-domains -> ag-edge at crates/ag-domains/Cargo.toml:47 [dependencies] (update the allow-list with an RFC/ADR citation; no silent new direction)
PASS required-edges: ag-edge -> ag-domains control plane present
FAIL cycle: ag-domains -> ag-edge -> ag-domains (CLAUDE.md:383, no circular dependencies)
dependency-direction: FAIL (3 violation(s))
```

`RC=1`.

S4, declaracion `ag-weird.workspace = true` en `ag-core`:

```
FAIL manifest parse: crates/ag-core/Cargo.toml:35: unparsed ag-* declaration : policy needs a parser update (fail closed)
PASS roster: 19 crates match CLAUDE.md rule 14
PASS required-edges: ag-edge -> ag-domains control plane present
PASS acyclic: no cycle among the 19 crates
dependency-direction: FAIL (1 violation(s))
```

`RC=1`: una forma no reconocida no se cuenta como conforme.

S5, eliminar `ag-domains` de `ag-edge`:

```
PASS roster: 19 crates match CLAUDE.md rule 14
PASS directions: 16 internal edges reviewed, 1 self dev-dependency ignored
FAIL required edge missing: ag-edge -> ag-domains (source: ADR-0012 y RFC-0011 (plano de control))
PASS acyclic: no cycle among the 19 crates
dependency-direction: FAIL (1 violation(s))
```

`RC=1`.

S6, `--help`: tres lineas de uso, `RC=0`. S7, `--bogus`: `dependency-direction: ERROR unknown argument --bogus (use --help)`, `RC=2`.

### `git diff --check`
Sin salida, `RC=0`.

### `bash scripts/ag-check`
```
PASS: descriptors
PASS: fmt
PASS: clippy
PASS: test
PASS: audit
PASS: deny
PASS: masters
PASS: prohibited
Summary: PASS=8 FAIL=0 SKIP=0
ag-check: all selected checks passed.
```

`AG_CHECK_RC=0`. Ocho comprobaciones, cero fallos, cero omitidas.

### Escaneo de contenido
Sobre los seis archivos tocados (`scripts/check-dependency-direction.sh`, `docs/architecture/dependency-direction-guard.md`, ambos descriptores, `README.md` y `CHANGELOG.md`):

```
===AI ATTRIBUTION SCAN===
RC=1
===RETIRED BRAND SCAN===
RC=1
===SECRET ASSIGNMENT SCAN===
RC=1
===EMOJI SCAN===
RC=1
===PROHIBITED BRANCH PREFIX IN DRAFTS===
RC=1
```

`RC=1` significa cero coincidencias en cada patron (atribucion de herramientas IA, marcas retiradas segun ADR-0011, asignaciones de secretos, emojis y prefijos de rama prohibidos en descriptores). Ningun hallazgo que reportar por outbox S-11.

## Impacto en calidad (ratchet)
- Cobertura: igual (sin codigo Rust de produccion)
- Advertencias: igual
- Deuda (TODO/FIXME): igual

## Criterios de salida que avanza
- A5: guarda automatica de direccion de dependencias para los 19 crates, implementable now, sin RFC nuevo.
- KPI del tablero: cambio con prueba reproducible (guarda en verde + cinco fallos simulados), sin saltar Fase 5 ni relajar gates.

## Cierre de issues
Closes: none

## Checklist
- [x] No anade dependencias ni crates
- [x] No toca `.github/` ni rutas protegidas
- [x] Guarda en verde en el workspace real
- [x] Falla cerrado con evidencia simulada
- [x] `git diff --check` en verde
- [x] `bash scripts/ag-check` ejecutado: 8 PASS / 0 FAIL / 0 SKIP
- [x] Documentacion y CHANGELOG actualizados
