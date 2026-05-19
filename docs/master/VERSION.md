# Documentos maestros — Registro de integridad

Este archivo registra la versión, fecha de instalación y hash SHA-256 de
cada documento maestro presente en este directorio. Los archivos
contenidos en `docs/master/` son la fuente de verdad del proyecto y no
deben modificarse fuera de un procedimiento RFC explícito.

Si cualquiera de los hashes registrados aquí no coincide con el archivo
real en disco, asuma que el documento maestro fue alterado y restaure
desde el origen autorizado antes de continuar.

## Versión vigente

- Versión documental: 4.0
- Fecha de la versión documental: Mayo 2026
- Fecha de instalación en el repositorio: 2026-05-19
- Origen: aporte directo de Gravital Labs (Nereira Technology and
  Business Solutions), República de Panamá.
- Licencia de la documentación: Apache 2.0, igual que el código.

## Maestros instalados

| Archivo | Tamano (bytes) | SHA-256 |
| --- | --- | --- |
| `ANTI-GRAVITAL-Blueprint-v4.0.pdf` | 511945 | `59a1df26bd24e96067c58c142709e3cb55fc33efbb1c8f3739d9473598dfb660` |
| `ANTI-GRAVITAL-Arquitectura-Tecnica.md` | 88066 | `c6f16879cd8c4d7772dbe2911043e31cce9196a31f75a235fdd8bcf72afd67ae` |
| `ANTI-GRAVITAL-Hoja-de-Ruta.md` | 28433 | `534f15ac103cdd45fdca7e91f1c5bd0e3a81527d72c59b0a25eb0425a02a80d4` |

## Historial de revisiones de maestros

| Fecha | Cambio | Origen |
| --- | --- | --- |
| 2026-05-19 | Instalacion inicial de los tres maestros v4.0. | Aporte directo de Gravital Labs. |
| 2026-05-19 | Reemplazo de placeholders de email: `security@gravital.io` y `hello@antigravital.dev` por `anti@gravitalcloud.com` (raiz) con `angelnereira@gravitalcloud.com` como respaldo de seguridad. Registrado en `docs/adr/0005-contact-identities.md`. | Decision del BDFL inicial. |

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
