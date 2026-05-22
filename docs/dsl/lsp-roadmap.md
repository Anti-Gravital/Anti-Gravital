# Hoja de Ruta del Servidor LSP Anti-Gravital

## Capacidades del alpha (v0.1 — Fase 3)

- diagnostics Error y Warning en tiempo real (textDocument/didOpen + didChange)
- completion: keywords, tipos primitivos, anotaciones, nombres de modelos
- hover: descripcion de tipos primitivos y anotaciones

## Capacidades futuras

- textDocument/definition — ir a la definicion del modelo referenciado
- textDocument/references — todas las referencias a un modelo
- textDocument/rename — renombrar modelo y todos sus usos
- textDocument/codeAction — sugerencias de correccion automatica
- textDocument/formatting — formateo del archivo .ag
- Soporte multi-archivo — index de todos los .ag del proyecto
- Bundled binary — el plugin VS Code incluye ag-lsp precompilado por plataforma
