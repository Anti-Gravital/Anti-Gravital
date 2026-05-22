# Anti-Gravital DSL — VS Code Extension

Soporte de lenguaje para archivos `.ag` del ecosistema Anti-Gravital.

## Capacidades

- Syntax highlighting para archivos `.ag`
- Diagnostics en tiempo real (errores y warnings del compilador DSL)
- Autocompletado de keywords, tipos primitivos y anotaciones
- Hover con descripcion de tipos y anotaciones

## Requisitos

El servidor LSP `ag-lsp` debe estar instalado:

    cargo install ag-lsp

Si no esta instalado, la extension muestra una notificacion con boton
para instalarlo automaticamente desde VS Code.

## Instalacion local

    cd tools/vscode-anti-gravital
    npm install
    npx vsce package
    code --install-extension anti-gravital-0.1.0.vsix

## Estado

Alpha — Fase 3 del proyecto Anti-Gravital.
Capacidades avanzadas en fases futuras: ver docs/dsl/lsp-roadmap.md
