# Resumen

Add measured chumsky 0.9.3 to 0.14.2 migration-cost evidence; keep the deferral.

- Fase afectada: ninguna fase nueva; evidencia aditiva sobre la decision de la Fase 3 ya tomada en RFC-0027 y ADR-0021 (ola 12).
- Tipo de cambio: [x] documentacion y gobernanza  [ ] codigo de crate  [ ] DSL  [ ] API publica  [ ] CI  [ ] dependencias.
- Documentos relacionados: RFC-0027 (seccion 11, aditiva), ADR-0021, RFC-0003,
  `docs/audits/chumsky-migration-spike.md`, `docs/audits/low-level-review-178-180.md`,
  `docs/INDEX.md`, `CHANGELOG.md`, CLAUDE.md reglas 17, 19, 22, 28 y 36.

## Cambios

- `docs/audits/chumsky-migration-spike.md`: evidencia nueva. Un spike desechable
  fuera del repositorio porto una unidad pequena y coherente del parser
  (`config_parser` + `config_field_parser`, el plumbing de entrada de
  `parse_tokens`, la conversion de errores y una sonda de `map_with_span`) de
  chumsky 0.9.3 a 0.14.2. Incluye la tabla de diferencias de API verificadas
  por compilacion, dos errores de compilacion reproducidos y corregidos, la
  diferencia observable de diagnostico con entrada malformada y 10 trials
  intercalados de microbenchmark con los metadatos completos de la regla 17.
- `docs/rfc/RFC-0027-chumsky-parser-upgrade.md`: seccion 11 aditiva. No modifica
  la decision de la seccion 9 ni ADR-0021; el diferimiento sigue vigente.
- `docs/INDEX.md` y `CHANGELOG.md`: navegacion y registro del cambio.

No se tocaron `Cargo.toml`, `Cargo.lock`, `crates/ag-dsl/src/parser.rs`,
`fuzz/`, `.github/` ni ningun archivo de produccion. El codigo del spike no se
commiteo al repositorio.

## Plan de prueba

- `git diff --check`
- `git diff --name-only`: solo archivos de documentacion (auditoria, RFC-0027,
  indices, CHANGELOG y este descriptor); ningun archivo de produccion.
- `git diff -- Cargo.toml Cargo.lock crates/ag-dsl fuzz .github`: sin salida.
- Escaneos de contenido prohibido: sin emojis, sin evidencia de herramientas IA
  ni atribucion, sin URLs de sesiones de chat, sin nombres de marca prohibidos
  por ADR-0011 (las tablas y rutas incluidas no contienen ninguno).
- Evidencia del spike (fuera del repo): `cargo build --release` en ambos crates
  con `CARGO_INCREMENTAL=0` (ambos exitosos), ejecucion de ambos binarios con
  aserciones de entrada valida superadas y 10 trials intercalados de 5000
  iteraciones medidas cada uno. Resultados en
  `/tmp/opencode/spike-chumsky/bench-results.txt` y `diagnostics-diff.txt`.

## Criterios de salida que avanza

- Convierte la estimacion de coste de RFC-0027 en evidencia medida sin alterar
  el contrato arquitectonico: ADR-0021 se mantiene, 0.9.3 sigue fijado.
- Documenta tres condiciones nuevas para una RFC sucesora (renderizado de
  tokens esperados, golden tests de diagnosticos como prerrequisito y la
  ausencia de evidencia de rendimiento concluyente).
- No cierra ninguna puerta: la fila de fuzz 24h sigue pendiente (#156) y la
  issue #180 permanece abierta.

## Cierre de issues

Refs #180
Closes: none

## Checklist

- [x] Pertenece a la fase correcta (evidencia aditiva, ola 12, sin cambio de fase).
- [x] Respeta la documentacion y no anade features no contempladas.
- [x] No rompe arquitectura ni anade dependencias circulares.
- [x] No anade complejidad innecesaria: el spike vive fuera del repositorio.
- [x] Compila / no aplica (sin codigo de produccion; los dos crates desechables compilaron).
- [x] Pasa `git diff --check` y los escaneos de contenido prohibido.
- [x] Tiene documentacion (auditoria, nota de RFC, indice y changelog).
- [x] No inventa resultados: la medicion se informa con su ruido y sus limitaciones.
- [x] No cierra la issue ni afirma que paso una puerta no verificada.
