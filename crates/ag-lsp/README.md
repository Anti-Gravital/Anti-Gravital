# ag-lsp

Servidor LSP (Language Server Protocol) del Anti-DSL.

Proporciona diagnostics en tiempo real, autocompletado e informacion
hover para archivos `.ag` en cualquier editor compatible con LSP.

## Uso

El servidor se comunica por stdio. Los editores lo lanzan automaticamente:

    ag-lsp

## Instalacion

    cargo install ag-lsp

## Capacidades (alpha v0.1)

- Diagnostics en tiempo real al abrir o editar un archivo `.ag`
- Completion: keywords, tipos primitivos, anotaciones DSL, nombres de modelos
- Hover: descripcion de tipos (`UUID`, `String`...) y anotaciones (`@primary`...)

## Plugin VS Code

El plugin `tools/vscode-anti-gravital/` detecta `ag-lsp` en PATH
automaticamente. Si no esta instalado, ofrece instalarlo via `cargo install ag-lsp`.

Ver `docs/dsl/lsp-roadmap.md` para capacidades futuras.

## Estado

Alpha — Fase 3. Ver `docs/modules/ag-lsp/README.md` para detalles de implementacion.
