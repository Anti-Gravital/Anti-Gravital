# Documentos maestros — Registro de integridad

Este archivo registra la versión, fecha de instalación y hash SHA-256 de
cada documento maestro presente en este directorio. Los archivos
contenidos en `docs/master/` son la fuente de verdad del proyecto y no
deben modificarse fuera de un procedimiento RFC explícito.

Si cualquiera de los hashes registrados aquí no coincide con el archivo
real en disco, asuma que el documento maestro fue alterado y restaure
desde el origen autorizado antes de continuar.

## Versión vigente

- Versión documental: 4.1 (markdown) — Blueprint PDF pendiente de re-export
- Fecha de la versión documental: 2026-05-26 (cierre documental Fase 4.0-4.5)
- Fecha de instalación en el repositorio: 2026-05-19 (v4.0 inicial)
- Origen: aporte directo de Gravital Labs (Nereira Technology and
  Business Solutions), República de Panamá.
- Licencia de la documentación: Apache 2.0, igual que el código.
- Idioma: maestros markdown bilingües (inglés canónico primero, español
  después) según `ADR-0008`. El Blueprint v4.1.md es la fuente markdown.

## Maestros instalados

| Archivo | Tamano (bytes) | SHA-256 |
| --- | --- | --- |
| `ANTI-GRAVITAL-Blueprint-v4.0.pdf` | 511945 | `59a1df26bd24e96067c58c142709e3cb55fc33efbb1c8f3739d9473598dfb660` |
| `ANTI-GRAVITAL-Blueprint-v4.1.md` | 10693 | `be7c175f133580a007864d5740ebebe8da762468c5b564650a6f0bd33f355cbb` |
| `ANTI-GRAVITAL-Arquitectura-Tecnica.md` | 206984 | `e055e7f4ea9f1fbbf0cf6bd05208afcefb31c7f2c7f849012d89da67885726f5` |
| `ANTI-GRAVITAL-Hoja-de-Ruta.md` | 70021 | `32b36442ec47f304a1bdf43ef1fee2b02df6886307d93a101d20d37f96157ba0` |

### Deuda explícita — Blueprint PDF v4.1

El PDF `ANTI-GRAVITAL-Blueprint-v4.0.pdf` está **desfasado** respecto a los
dos maestros markdown, que ya incorporan los cambios de la Fase 4.5 según
`ADR-0007` (Hoja de Ruta: fila 4.5, duración total 25–30 meses, nueva
sección 4.5 completa; Arquitectura Técnica: ag-mail/ag-domains en §5.1/§5.2,
sextas y séptimas reglas de dependencia en §5.3, tabla DSL realineada en
§7.2, subsecciones 8.8/8.9, integración §10.6 con `ag-domains`). El
re-export a `ANTI-GRAVITAL-Blueprint-v4.1.pdf` queda como **tarea pendiente
de tooling de exportación** y se ejecuta fuera del scope de esta rama
documental. La política #4 de este mismo archivo gobierna la discrepancia:
**los maestros markdown gobiernan**.

Cuando se ejecute la re-export, la entrada del PDF se reemplaza por
`ANTI-GRAVITAL-Blueprint-v4.1.pdf` con tamaño y hash actualizados, y el
PDF antiguo se mueve a un archivo de historial documentado en una entrada
nueva del historial de revisiones.

## Historial de revisiones de maestros

| Fecha | Cambio | Origen |
| --- | --- | --- |
| 2026-05-19 | Instalacion inicial de los tres maestros v4.0. | Aporte directo de Gravital Labs. |
| 2026-05-19 | Reemplazo de placeholders de email: `security@gravital.io` y `hello@antigravital.dev` por `anti@gravitalcloud.com` (raiz) con `angelnereira@gravitalcloud.com` como respaldo de seguridad. Registrado en `docs/adr/0005-contact-identities.md`. | Decision del BDFL inicial. |
| 2026-05-23 | Suplemento Fase 4.5: Hoja-de-Ruta y Arquitectura-Técnica actualizados con `ag-mail` (estándar diferido) y `ag-domains` (opcional infra), 15→17 crates, 24–28→25–30 meses, tabla DSL realineada, dirección de dependencia `ag-auth → ag-mail` documentada. Registrado en `docs/adr/0007-ag-mail-ag-domains.md`. Blueprint PDF queda como deuda explícita pendiente de re-export. | Decisión del BDFL vía ADR-0007. |
| 2026-05-25 | Cierre documental Fase 4.0-4.5: (1) maestros markdown convertidos a bilingüe (inglés canónico primero, español después) según `ADR-0008`; (2) fidelidad a implementación real — fases 1-4.5 marcadas técnicamente completas en la Hoja-de-Ruta, y en la Arquitectura se corrigió WebAuthn (ciborium/p256/ed25519 vía ADR-0006), templates de correo (StringTemplate), y caché L2 (RFC-0005, no implementado); (3) nuevo `ANTI-GRAVITAL-Blueprint-v4.1.md` como fuente markdown versionable del Blueprint. Blueprint PDF sigue como deuda pendiente de re-export. | Decisión del BDFL vía ADR-0008. |
| 2026-05-26 | Hoja-de-Ruta: checkboxes de la Fase 4.5 (4.5.1, 4.5.2, 4.5.3) marcados `[x]` con notas de implementación real; cabecera de estado bilingüe añadida. Hash actualizado a `24bc86759f6a26617a5f7a3d655f7083db7c3d11f48c5e6572da6ab080322678` (67745 bytes). | Rama `docs-cierre-fase-4.5`, cierre de fase. |
| 2026-05-26 | Hoja-de-Ruta: correcciones correctivas pre-Fase-5 (deudas tecnicas DEBT-001–011 actualizadas, estado real de fases). Hash actualizado a `ce1218a572e09bfebf997752273e009a1963f57a3add0b3b71b09c25fc9b5a1d` (67968 bytes). | Rama `corrective-before-fase-5`. |
| 2026-06-03 | Pivot `ag-mail` a MTA outbound nativo (`ADR-0010` / `RFC-0009`): nota de actualizacion de alcance en Arquitectura §8.8 (EN+ES) y nota futura Fase 4.6 en Hoja-de-Ruta §4.5.5 (EN+ES). No declara el MTA implementado. Arquitectura → `b3752b155dc9269238a13d81e8fae7f342c321c0e13a3826e144fe1e7f2a0ad6` (204649 bytes); Hoja-de-Ruta → `cf7bc8ae7f0914374c08cc978637762941f4ad5af112242f6e37d475491b2d36` (69443 bytes). | Decision del BDFL vía ADR-0010. |
| 2026-06-03 | Implementacion Fase 4.6-A (motor MTA nativo opt-in, feature `mta`): Hoja-de-Ruta §4.5.5 (EN+ES) actualizada para reflejar el nucleo 4.6-A implementado (resolucion MX, ESMTP+STARTTLS, DKIM Ed25519, clasificacion de bounces). Hoja-de-Ruta → `07d0ad40ac7f19a169e9507ba7a581f770b72e38115c526563614bca2c434b4b` (69789 bytes). | Implementacion RFC-0009 Fase 4.6-A. |
| 2026-06-04 | Politica de marcas comerciales (`ADR-0011`/`RFC-0010`): retirados los adaptadores con nombre de marca de `ag-mail`; saneo de las menciones de marca en ambos maestros (relay SMTP nativo / MTA nativo). Arquitectura → `34026dbe0ae6ecb8f6280e07d4cf5e5da33c126e352973f54df44df463090af6` (204762 bytes); Hoja-de-Ruta → `02c5b8a2529cf24ae308e45f9d236f2534b53a73da46d1a255adcd133b5dc424` (69719 bytes). | Decision del BDFL vía ADR-0011. |
| 2026-06-10 | Roster de crates sincronizado con la realidad (`ADR-0012`/`ADR-0013`): anadidos `ag-workers` (estandar diferido), `ag-edge` (opcional infra) y `ag-lsp` (nucleo) en §5.1/§5.2 (EN+ES) de Arquitectura, en la seccion 4 del Blueprint (EN+ES) y en la nota de conteo de la Hoja-de-Ruta (EN+ES); conteo 17→20. Blueprint-v4.1 → `be7c175f133580a007864d5740ebebe8da762468c5b564650a6f0bd33f355cbb` (10693 bytes); Arquitectura → `e055e7f4ea9f1fbbf0cf6bd05208afcefb31c7f2c7f849012d89da67885726f5` (206984 bytes); Hoja-de-Ruta → `32b36442ec47f304a1bdf43ef1fee2b02df6886307d93a101d20d37f96157ba0` (70021 bytes). | Sincronizacion de decisiones ya aprobadas (ADR-0012/0013). |

## Verificación local

Desde la raíz del repositorio:

```sh
sha256sum docs/master/ANTI-GRAVITAL-Blueprint-v4.0.pdf \
          docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md \
          docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
```

Los valores devueltos deben coincidir letra por letra con los listados
en la tabla anterior.

## Política de modificación

1. Los archivos en `docs/master/` no se editan. Una nueva versión de
   un maestro implica un commit dedicado que reemplaza el archivo
   completo y actualiza la tabla de hashes en este documento.
2. Toda modificación requiere una RFC aprobada bajo `docs/rfc/`.
3. Los archivos derivados bajo `docs/architecture/`, `docs/roadmap/`,
   `docs/modules/`, `docs/dsl/`, `docs/security/`, `docs/governance/` y
   `docs/benchmarks/` se regeneran a partir de los maestros y nunca al
   revés. Si un derivado contradice un maestro, gana el maestro.
4. El Blueprint PDF es la versión unificada de presentación. Cuando
   exista discrepancia entre el PDF y los maestros markdown, los
   maestros markdown gobiernan, porque son el formato auditable.
