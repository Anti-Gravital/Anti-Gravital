## Resumen
Revalidar sin cambios de codigo los issues #180 y #179 y registrar notas fechadas con la condicion exacta de cierre.

## Componente afectado
Auditorias de dependencias (`docs/audits`), ag-dsl y ag-domains solo como documento. Proyecto de equipo `atender-issues`.

## Tipo de cambio
- [ ] bug
- [ ] enhancement
- [x] tech-debt
- [x] documentation
- [ ] refactor

## Documentos relacionados
- `docs/audits/chumsky-migration-spike.md` (seccion 10 anadida: revalidacion 2026-11-23 de #180)
- `docs/audits/instant-acme-08-compile-spike.md` (seccion anadida: revalidacion 2026-11-23 de #179)
- `docs/rfc/RFC-0027-chumsky-parser-upgrade.md` y `docs/adr/0021-chumsky-upgrade-deferral.md` (NO modificados; rigen #180)
- `docs/ag-domains/reference/instant-acme-0.8-migration.md` y `docs/audits/acme-staging-handoff.md` (NO modificados; rigen #179)
- `docs/kaizen/journal/2026-11-23-i180-revalidation-1918.md`

## Plan de prueba
- `cargo audit --file Cargo.lock` -> exit 0; 1563 advisories; 4 allowed unmaintained (astral-tokio-tar, chrono, humantime, paste); sin entrada para chumsky ni instant-acme.
- `https://crates.io/api/v1/crates/chumsky` -> max_stable_version 0.14.2, updated_at 2026-11-17T16:52:46Z; sin release nuevo.
- `https://crates.io/api/v1/crates/instant-acme` -> max_stable_version 0.8.5, updated_at 2026-02-24T09:47:58Z; sin release nuevo.
- `gh api repos/Anti-Gravital/Anti-Gravital/issues/{180,179,87}` -> los tres OPEN; #87 bloqueado sin evidencia de staging.
- `git diff 72b97ad..HEAD -- crates/ag-dsl/src/parser.rs crates/ag-domains/src/acme/renewal.rs Cargo.toml crates/ag-domains/Cargo.toml` -> sin cambios; solo 7 lineas no relacionadas en Cargo.lock.
- `git diff --check`, `bash scripts/ag-check`, `bash scripts/kaizen check` y escaneos de contenido (resultado real en el diario).

## Impacto en calidad (ratchet)
- Cobertura: igual (sin cambios de codigo)
- Advertencias: igual (sin cambios de codigo ni supresiones S-10)
- Deuda (TODO/FIXME): igual

## Criterios de salida que avanza
- Confirmar con metadatos en vivo que no hay release nuevo de chumsky ni de instant-acme (hecho).
- Confirmar que #87 sigue sin evidencia de staging y que el pin de produccion no cambio (hecho).
- Restablecer por escrito la condicion exacta de cierre de #180 y #179 (secciones anadidas).
- Mantener ambos issues ABIERTOS; usar `Refs #180, #179`, nunca `Closes #180` ni `Closes #179`.

## Aprendizaje (opcional)
`scripts/ag-check` y `scripts/kaizen check` exigen una palabra clave de cierre en el descriptor. Para no fingir un cierre se anade `Closes: none` junto a `Refs #180, #179`: ninguno de los dos issues se cierra en este cambio.

## Cierre de issues
Closes: none
Refs #180, #179

## Checklist
- [x] Compila (sin cambios de codigo)
- [x] Pasa tests (verificacion de solo lectura; sin cambios de codigo)
- [x] `scripts/kaizen check` en verde (resultado real en el diario)
- [x] Documentacion actualizada
- [x] Linea base no empeora
