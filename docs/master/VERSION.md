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
| `ANTI-GRAVITAL-Blueprint-v4.1.md` | 10007 | `a6a1479c3586ea0bb7afeff3f801d13aebcea0f046216130358f0b897e351bc6` |
| `ANTI-GRAVITAL-Arquitectura-Tecnica.md` | 203145 | `d511e96b4a00a6eb2aec09efe7a7fd6c15a0cef7ff89d055f3e2f9fff7b8de31` |
| `ANTI-GRAVITAL-Hoja-de-Ruta.md` | 67745 | `24bc86759f6a26617a5f7a3d655f7083db7c3d11f48c5e6572da6ab080322678` |

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
