# Journal 2026-11-23 03:57Z — roadmap-avance (A5, dep-guard)

## Tarea
Automatizar la guarda de direccion de dependencias de los 19 crates (task A5 del tablero `roadmap-avance`), que hasta ahora solo se verificaba a mano (`docs/audits/pre-fase5-architecture.md:48-62`). Sin push, PR ni merge; sin tocar `.github/`.

## Decision
Implementar un script de bash opt-in en lugar de un test Rust: la comprobacion es sobre texto de manifiestos, y el script corre sin toolchain, sin red, sin secretos y sin anadir dependencias ni crates. La politica se codifica como roster + aislamiento de `ag-core` + lista blanca de 16 pares + direcciones prohibidas + arista obligatoria `ag-edge -> ag-domains` + deteccion de ciclos + parseo falla cerrado. Fuentes: `CLAUDE.md` reglas 14/15, Arquitectura Tecnica 5.3, ADR-0007/0012, RFC-0011/0012, `docs/graph/crates.md` y `docs/audits/oportunidades-arquitectura.md` A5. No se cambio ninguna regla de dependencias; por eso no hace falta RFC (reglas 22/28).

## Evidencia (comandos y resultados reales)
- `bash -n scripts/check-dependency-direction.sh`: sin salida, RC=0.
- `scripts/check-dependency-direction.sh` en el workspace real: 4 lineas PASS, `dependency-direction: PASS (19 crates, 17 edges, 1 self dev-dependency ignored)`, RC=0.
- Copia scratch con `mktemp -d`, invocada desde `/tmp`: S0 limpio PASS; S1 `ag-mail -> ag-auth` RC=1 con direccion prohibida, arista no listada y ciclo `ag-auth -> ag-mail -> ag-auth`; S2 `ag-cache -> ag-ghost` RC=1 por crate desconocido; S3 `ag-domains -> ag-edge` RC=1 con ciclo; S4 declaracion `ag-*` no parseable RC=1 (falla cerrado); S5 sin `ag-edge -> ag-domains` RC=1 por arista obligatoria ausente; `--help` RC=0; `--bogus` RC=2.
- `git diff --check`: sin salida, RC=0.
- `bash scripts/ag-check`: `Summary: PASS=8 FAIL=0 SKIP=0`, RC=0.
- Escaneo de contenido en los seis archivos tocados: cero coincidencias de atribucion IA, marcas retiradas, asignaciones de secretos, emojis y prefijos de rama prohibidos. Sin hallazgos S-11.

## Resultado
Entregable: `scripts/check-dependency-direction.sh`, `docs/architecture/dependency-direction-guard.md`, punteros en las dos secciones del README, entrada en CHANGELOG y descriptores `docs/kaizen/pr-drafts/community18-dep-guard.md` + espejo en `docs/pr-drafts/`. Salida exacta de todas las verificaciones en el descriptor. Sin push ni PR.

## Aprendizaje
Una lista blanca que iguala las aristas presentes genera la friccion deseada: anadir una direccion nueva exige citar el documento que la autoriza en el mismo cambio. Conviene distinguir las declaraciones de arista (17) de los pares unicos (16): `ag-auth -> ag-mail` aparece en dependencias normales y de desarrollo, y la autodependencia de test de `ag-mail` no es un ciclo de crates.
