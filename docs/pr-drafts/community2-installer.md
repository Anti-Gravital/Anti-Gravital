# Design proposal: signed release manifest and trust anchor (RFC-0030)

## Resumen

Add RFC-0030 design for an offline release key and signed checksum manifest; update security docs and changelog. Issue #141 stays open.

## Fase afectada

Release infrastructure and security governance, pre-1.0. No phase state changes and no master document is touched. Enables the open custody item of RFC-0021 stage C without starting that stage.

## Tipo de cambio

- [x] RFC nuevo en estado borrador (reglas 22 y 28)
- [x] Indice de RFC (`docs/rfc/README.md`)
- [x] Hoja de ruta del ancla en `SECURITY.md` y `docs/security/INSTALLATION_INTEGRITY.md`
- [x] Entrada en CHANGELOG con `Refs #141`
- [ ] Codigo, dependencias, CI o clave (ninguno; diseno unicamente)
- [ ] Cambio de estado de fase o de documentos maestros (ninguno)

## Documentos relacionados

- RFC-0030 `docs/rfc/RFC-0030-signed-release-manifest-and-trust-anchor.md`
- Issue #141 y el comentario de revision de infraestructura
- `docs/audits/infrastructure-review-112-149-141.md`, seccion 4
- `ADR-0009` reglas 2, 4 y 5; `ADR-0008`; `ADR-0011`; `CLAUDE.md` reglas 5, 22, 26, 28
- RFC-0021 secciones 4.1 y 9 (custodia de la clave de release aun abierta)
- `docs/rfc/README.md`; `SECURITY.md`; `docs/security/INSTALLATION_INTEGRITY.md`

## Cambios concretos

- RFC-0030: generacion y custodia offline de clave Ed25519, publicacion de la
  clave publica en dos canales fuera del repositorio, formato del manifiesto y
  la firma, orden de verificacion, rotacion y revocacion, arranque offline,
  paridad Windows/macOS, comportamiento ante fallos y condicion exacta de cierre.
- El RFC declara explicitamente que no existe clave, artefacto firmado ni
  workflow, y que no cierra #141.
- Indice de RFC y secciones de hoja de ruta en los dos documentos de seguridad.
- CHANGELOG: entrada bajo `[Unreleased]` con `Refs #141`.

## Plan de prueba

```
git diff --check
git status --short
grep -n '0030' docs/rfc/README.md
grep -n 'RFC-0030' SECURITY.md docs/security/INSTALLATION_INTEGRITY.md CHANGELOG.md
grep -RInE '[\x{1F300}-\x{1FAFF}]|[\x{2600}-\x{27BF}]' docs/rfc/RFC-0030-signed-release-manifest-and-trust-anchor.md
grep -nE '^(Closes|Refs|Fixes|Resolves) ' docs/pr-drafts/community2-installer.md
bash tools/check-installers.sh
```

No hay codigo: no aplican cargo fmt/clippy/test/audit/deny. `bash tools/check-installers.sh` solo confirma que el interface de instalador existente no se rompio. La verificacion completa del diseno queda descrita en RFC-0030 seccion 11 y no se ejecuta en este PR.

## Criterios de salida que avanza

- Issue #141: avanza con un diseno auditable y verificable; permanece abierta
  porque no hay clave, publicacion externa ni evidencia de extremo a extremo.
- RFC-0021 etapa C: queda registrada la propuesta de custodia que la RFC dejaba
  como pregunta abierta; la etapa C no se inicia.
- Ninguna casilla de `docs/roadmap/STATUS.md` cambia de estado.

## Cierre de issues

Refs #141
Closes: none

## Checklist

- [x] Diseno dentro del alcance: sin codigo, sin clave, sin servicio obligatorio
- [x] Sin cambios en `docs/master/` ni en el estado de fases
- [x] RFC con secciones obligatorias, estado `borrador` y fecha de decision pendiente
- [x] Documentacion de seguridad actualizada en la misma rama, sin afirmar capacidades inexistentes
- [x] Sin emojis, sin evidencia de herramientas IA, mensajes y titulo <=256 caracteres
