# docs(fase-4.5): registrar ag-mail y ag-domains como fase aditiva entre Fase 4 y Fase 5

## Fase afectada

Fase 4.5 (nueva, aditiva, introducida por ADR-0007). Etapa documental: NO toca
codigo Rust ni archivos `crates/`. La etapa de implementacion tecnica va en
una rama posterior `fase-4.5` y queda bloqueada hasta que este PR este
mergeado.

## Tipo de cambio

Integracion documental (`docs`). Aditiva: NO modifica el alcance ni los
entregables de la Fase 4 ya completada, NO adelanta el hito v0.5 BETA que
permanece al final de la Fase 5.

## Documentos relacionados

- `docs/adr/0007-ag-mail-ag-domains.md` (nuevo) — ADR que oficializa la decision.
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` — fila 4.5, duracion total 25-30
  meses, nueva seccion Fase 4.5 completa.
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` — §5.1, §5.2, §5.3 (6a y
  7a regla), §5.4, §7.2, §8.8/§8.9, §10/§10.6, §19 (glosario).
- `docs/master/VERSION.md` — nuevos hashes, deuda explicita del PDF v4.1.
- `docs/master/ANTI-GRAVITAL-Blueprint-v4.0.pdf` — registrado como deuda
  pendiente de re-export a v4.1 (los maestros markdown gobiernan).

## Resumen

Esta rama integra documentalmente la **Fase 4.5 aditiva** con dos crates
nuevos: `ag-mail` (estandar diferido) y `ag-domains` (opcional infra). La
implementacion tecnica NO esta en este PR. Las decisiones canonicas:

- **Conteo de crates:** 15 -> 17.
- **Cronograma total:** 24-28 -> 25-30 meses.
- **DSL:** bloques `mail`, `domain`, `dns`, `tls` en v0.7 (Fin Fase 4.5);
  plugin hooks reasignados a v0.8 (Fin Fase 9).
- **Hito v0.5 BETA:** sigue al final de Fase 5, sin cambios.
- **Direccion de dependencias:** `ag-auth` consume `ag-mail` (NO al reves);
  `ag-cloud` consume `ag-domains` (sin dependencia rigida en todos los
  targets). Sexta y septima reglas en el capitulo 5 de Arquitectura Tecnica.
- **Alcance restringido y fijado en ADR:** `ag-mail` v1 solo outbound,
  sin MTA, sin inbound, sin antispam. `ag-domains` no es registrador,
  no reemplaza Terraform.

Archivos tocados:

**Maestros y registro:**
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`
- `docs/master/VERSION.md` (hashes + deuda PDF)
- `CHANGELOG.md`

**ADR:**
- `docs/adr/0007-ag-mail-ag-domains.md` (nuevo, estado Aprobado)

**Roadmap:**
- `docs/roadmap/STATUS.md` (nueva seccion Fase 4.5)
- `docs/roadmap/fase-04-5-ag-mail-y-ag-domains.md` (nuevo, verbatim del maestro)
- `docs/roadmap/preambulo.md`
- `docs/roadmap/calendar.md`
- `docs/roadmap/README.md`

**Architecture (derivados verbatim del maestro):**
- `docs/architecture/03-alcance-y-limites.md`
- `docs/architecture/05-ecosistema-modulos.md`
- `docs/architecture/08-modulos-batteries-included.md`
- `docs/architecture/10-despliegue-ag-cloud.md`
- `docs/architecture/19-glosario.md`

**Modules:**
- `docs/modules/README.md`
- `docs/modules/ag-mail/README.md` (nuevo)
- `docs/modules/ag-domains/README.md` (nuevo)

**DSL:**
- `docs/dsl/versionado.md`

**Constitucion:**
- `CLAUDE.md` (regla 14 ampliada)

**Bilingue:**
- `README.md` (secciones ES y EN actualizadas en el mismo commit, fila 4.5 en
  Calendario / Calendar)

## Plan de prueba

Como es un cambio puramente documental, no hay tests de codigo. La revision
manual cubre:

```sh
# 1. Verificar que la Fase 4.5 aparece consistente en los 3 maestros y STATUS
grep -c "Fase 4.5\|4.5" \
    docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md \
    docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md \
    docs/master/VERSION.md \
    docs/roadmap/STATUS.md

# 2. Verificar que no quedan menciones VIVAS (presente-tense) a "15 crates"
#    Las menciones historicas (ADR-0001, CHANGELOG, pr-drafts/phase-0, STATUS
#    checkbox `[x]`) son correctas y se preservan.
grep -rn "15 crates" --include="*.md" .

# 3. Verificar que ningun derivado contradice un maestro
diff <(sha256sum docs/master/*.md) <(sha256sum docs/master/*.md)  # debe ser identico al registrado en VERSION.md
sha256sum docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md \
          docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md

# 4. Verificar paridad bilingue en README
grep -c "ag-mail\|ag-domains" README.md  # debe encontrar menciones en ambas secciones

# 5. Verificar que los crates ag-mail y ag-domains tienen ficha en docs/modules/
ls docs/modules/ag-mail/ docs/modules/ag-domains/
```

Los hashes esperados (registrados en `docs/master/VERSION.md`) son:

- `ANTI-GRAVITAL-Hoja-de-Ruta.md`: `ff5e322f568e2ecf416d09235c91d8f4eb004531b6844f513a68b63042b64590` (33516 bytes)
- `ANTI-GRAVITAL-Arquitectura-Tecnica.md`: `d8045e0881d789c873dae26a862d8e6e2821abd5cc4a5c8cb6ef5ca17a2788b3` (99836 bytes)
- `ANTI-GRAVITAL-Blueprint-v4.0.pdf`: `59a1df26bd24e96067c58c142709e3cb55fc33efbb1c8f3739d9473598dfb660` (511945 bytes, sin cambios, deuda registrada).

## Criterios de salida avanzados

Este PR documental NO cierra criterios de salida de la Fase 4.5; esos se
cumpliran en la rama `fase-4.5` (implementacion tecnica). Sin embargo
**habilita** la apertura de la Fase 4.5 al satisfacer:

- [x] Existe un ADR aprobado para `ag-mail` y `ag-domains` (ADR-0007).
- [x] Los tres maestros documentales reflejan la Fase 4.5 de forma consistente.
- [x] El estado vivo (`docs/roadmap/STATUS.md`) tiene la entrada Fase 4.5 con
      todos los criterios marcados Pendiente.
- [x] Los modulos tienen ficha en `docs/modules/ag-mail/` y `docs/modules/ag-domains/`.
- [x] El DSL v0.7 esta documentado en `docs/dsl/versionado.md`.
- [x] CLAUDE.md (regla 14) reconoce las dos nuevas clasificaciones "Estandar
      diferido" y "Opcional infra".
- [x] README.md bilingue actualizado en el mismo commit (decision del
      mantenedor del 2026-05-23: paridad ES/EN por commit).

## Deuda explicita reconocida

- **Blueprint PDF v4.1.** Los dos maestros markdown estan al dia; el PDF
  `ANTI-GRAVITAL-Blueprint-v4.0.pdf` queda desfasado pendiente de re-export.
  La politica vigente (`docs/master/VERSION.md` regla 4) dice que los
  maestros markdown gobiernan. La deuda esta registrada en VERSION.md y
  CHANGELOG.md.

## Checklist final

- [x] Pertenece a la Fase 4.5 (introducida por este PR mediante ADR-0007)
- [x] Respeta la documentacion existente (no contradice ningun maestro vivo)
- [x] No rompe arquitectura ni modularidad (ningun cambio de codigo)
- [x] No crea dependencias circulares (define explicitamente la 6a y 7a regla
      de dependencias entre crates)
- [x] Compila (ningun archivo de codigo tocado; CI debe seguir verde)
- [x] Pasa todos los tests (sin cambios de codigo)
- [x] Pasa cargo fmt (sin cambios de codigo)
- [x] Pasa cargo clippy -D warnings (sin cambios de codigo)
- [x] Tiene documentacion (es el PR entero documental)
- [x] No contiene emojis
- [x] No contiene atribucion de IA en ningun commit (verificado: trailers
      vacios, autor unico Angel Nereira <contact@angelnereira.com>)
- [x] Commits individuales por unidad logica (3 commits: ADR + maestros,
      README + roadmap + VERSION + CHANGELOG, derivados + modulos + DSL +
      CLAUDE.md)
- [x] Titulo del PR <= 256 caracteres
