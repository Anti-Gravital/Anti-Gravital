# Resumen

Preparar el alpha `0.1.0-alpha.1`: version del workspace, workflow de release para cuatro targets con checksums y prerelease DRAFT, RFC-0032 y seccion de CHANGELOG.

## Fase afectada

Infraestructura de release del hito alpha (pre-0.5). No modifica ninguna fase del roadmap ni cierra puertas de salida; GOVERNANCE.md reserva la liberacion de versiones hasta 0.5 al BDFL.

## Tipo de cambio

- [x] RFC aceptada (RFC-0032, dispensa BDFL registrada)
- [x] Versionado y manifiestos Cargo (Cargo.toml, Cargo.lock)
- [x] CI: nuevo workflow `.github/workflows/release.yml`
- [x] Documentacion: indice de RFC, CHANGELOG, descriptor
- [ ] Codigo de crates, DSL o API publica (no aplica)
- [ ] Publicacion en crates.io o release (no aplica; `publish = false`, DRAFT)

## Documentos relacionados

- `docs/rfc/RFC-0032-alpha-release-process.md` (aceptada)
- `docs/rfc/RFC-0030-signed-release-manifest-and-trust-anchor.md` (borrador; no cierra #141)
- `RFC-0022` (toda invocacion Cargo en CI usa `--locked`)
- `GOVERNANCE.md` (autoridad de release del BDFL hasta 0.5)
- `CHANGELOG.md` (seccion `[0.1.0-alpha.1] - 2026-11-22`)

## Plan de prueba

Comandos ejecutados en el worktree con el directorio de destino compartido:

```
CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 cargo check --workspace
CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 cargo check --locked --workspace
python3 -c "import yaml,sys; [yaml.safe_load(open(f)) for f in sys.argv[1:]]" .github/workflows/release.yml
grep -qiE "^[[:space:]>*-]*(closes|fixes|resolves)[[:space:]:]+(#[0-9]+|none|ninguno)\b" docs/pr-drafts/community5-release.md
grep -RIPn "[\x{1F300}-\x{1FAFF}]|[\x{2600}-\x{27BF}]|[\x{1F000}-\x{1F2FF}]" --include='*.md' --include='*.yml' --include='*.toml' --include='*.sh' --exclude-dir=.git .
git diff --check
```

El workflow de GitHub no se ejecuta en este cambio: no se han hecho push, tags ni release. La verificacion local cubre manifiestos, lockfile, sintaxis YAML, politicas de descriptor y contenido prohibido; el primer run real se revisara en el DRAFT.

## Criterios de salida que avanza

- Hito alpha: plataforma ejecutable, instalable y probable con una version identificable y artifacts por target.
- Trazabilidad de release: RFC aceptada que fija esquema de version, matriz de build, manifiesto, limite de publicacion y rollback.
- Gobernanza: decision de release registrada con dispensa explicita, no implicita.
- Interfaz entre roles: assets `ag-<target>` / `ag-lsp-<target>` y
  `checksums.txt` con nombres de archivo desnudos, segun lo que consume el
  canal alpha de los instaladores (community5/installer).

## Cierre de issues

Closes: none
Refs #141 (sigue abierto y bloqueado: no hay clave de firma ni ancla de confianza; RFC-0032 no lo cierra)

## Checklist

- [x] Pertenece a la fase correcta (infraestructura de release alpha)
- [x] Respeta la documentacion y no anade features al producto
- [x] Compila (`cargo check --workspace`) y el lockfile esta sincronizado (`--locked`)
- [x] Tiene documentacion (RFC, indice, CHANGELOG) y verificacion registrada
- [x] Sin evidencia de herramientas IA, emojis, push, tag ni publicacion
