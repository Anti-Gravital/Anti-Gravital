# ag-auth: autenticacion completa — JWT Ed25519, API keys BLAKE3, AuthConfig

## Fase afectada

Fase 4 — Modulos Estandar

## Tipo de cambio

Nuevo crate funcional (`feat`)

## Documentos relacionados

- `docs/superpowers/specs/2026-05-22-fase4-modulos-estandar-design.md` seccion ag-auth
- `docs/superpowers/plans/2026-05-22-fase4-ag-auth.md`
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` seccion 8.1

## Resumen

Implementa `ag-auth` con los componentes criticos de la iteracion inicial:

- `AuthConfig::from_env()` — lectura de claves JWT y configuracion WebAuthn desde variables de entorno
- `JwtSigner` — firma y verificacion de JWT con algoritmo EdDSA (Ed25519) via `jsonwebtoken`
- `api_keys` — generacion con 32 bytes de entropia OS, hash BLAKE3, verificacion en tiempo constante
- `AgAuth` — fachada publica que agrupa los componentes anteriores
- Tres migraciones SQL para `ag_sessions`, `ag_api_keys` y `ag_webauthn_credentials`
- TECH-DEBT explicito para WebAuthn, OAuth2 y SessionStore (segunda iteracion)

## Plan de prueba

- `cargo test -p ag-auth` — 11 tests unitarios, 2 doc-tests, todos pasan
- `cargo clippy -p ag-auth -- -D warnings` — cero advertencias
- `cargo fmt --all` — sin cambios pendientes
- `cargo build --workspace` — sin errores

## Criterios de salida avanzados

- ag-auth compila y pasa tests en rama `fase-4`
- JWT Ed25519 sign/verify funcional con claves generadas en runtime
- API keys con hash BLAKE3 y comparacion en tiempo constante
- Migraciones SQL para persistencia futura bajo feature `persistent`
- TECH-DEBT documentado segun regla 29 de CLAUDE.md

## Checklist final

- [x] Pertenece a Fase 4 segun Hoja de Ruta
- [x] Respeta documentacion (spec y plan)
- [x] No rompe arquitectura ni modularidad
- [x] No anade complejidad innecesaria
- [x] No crea dependencias circulares
- [x] Compila sin errores
- [x] Pasa todos los tests (13 en total)
- [x] Pasa cargo fmt
- [x] Pasa cargo clippy -D warnings
- [x] Tiene documentacion (doc comments + TECH-DEBT)
- [x] No contiene emojis
- [x] No contiene atribucion de IA
- [x] Commits individuales por componente logico
