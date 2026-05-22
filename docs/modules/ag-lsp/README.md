# ag-lsp

> Capitulo de arquitectura: `docs/architecture/07-anti-dsl.md`.
> Hoja de ruta LSP: `docs/dsl/lsp-roadmap.md`.
> Criticidad: Nucleo.
> Fase de implementacion: Fase 3.

## Dominio

Servidor LSP (Language Server Protocol) del Anti-DSL. Proporciona diagnostics
en tiempo real, autocompletado e informacion hover para archivos `.ag` en
cualquier editor compatible con LSP.

## Dependencias internas permitidas

Depende unicamente de `ag-dsl` (para `lint()` y `compile()`). No depende
de ningun otro crate Anti-Gravital.

## Reglas aplicables

Vease las reglas 14 y 15 de `CLAUDE.md`.

## Estado

Fase 3 completado. Binario `ag-lsp` funcional. Smoke test del protocolo
LSP verificado: responde `initialize` con `serverInfo.name = "ag-lsp"`.

## Capacidades implementadas (alpha v0.1)

- `textDocument/initialize` — capabilities: completion, hover, sync full.
- `textDocument/didOpen` + `didChange` — diagnostics en tiempo real
  via `ag_dsl::lint()`. Errores y warnings con rango de texto correcto.
- `textDocument/completion` — keywords DSL, tipos primitivos, anotaciones,
  y nombres de modelos del schema activo.
- `textDocument/hover` — descripcion markdown de tipos (`UUID`, `String`,
  `Int`...) y anotaciones (`@primary`, `@references`...).

## Uso

El servidor se comunica por stdio con el cliente LSP (VS Code, Neovim, etc.):

    ag-lsp

El plugin VS Code en `tools/vscode-anti-gravital/` detecta automaticamente
el binario en PATH y lo lanza. Ver `docs/dsl/lsp-roadmap.md` para
capacidades futuras.

## Tests

10 tests unitarios en `crates/ag-lsp/src/backend.rs`:
- `byte_to_position_*` — conversion de offsets a posiciones LSP
- `word_at_position_*` — extraccion de la palabra bajo el cursor
- `hover_content_*` — contenido hover por tipo y anotacion
- `static_items_*` — lista de completion items
- `ag_diag_*_maps_to_lsp_*` — mapeo de severidad ag-dsl a LSP
