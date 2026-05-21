# Spec: Fase 3 — Pendientes (cargo-fuzz, ag-lsp, Plugin VS Code)

**Fecha:** 2026-05-21
**Fase:** 3 — Anti-DSL alpha
**Estado:** Aprobado
**Autor:** Angel Nereira

---

## Contexto

DSL v0.1–v0.4 esta completo con 95.26% de cobertura de tests. Quedan tres
entregables de Fase 3 sin implementar:

1. Fuzzing del parser con `cargo-fuzz`
2. Servidor LSP `ag-lsp` con autocompletado y diagnostics
3. Plugin VS Code con syntax highlighting, diagnostics en tiempo real y
   deteccion dinamica del binario LSP

---

## Orden de implementacion

```
cargo-fuzz  →  ag-lsp (incluye refactor lint en ag-dsl)  →  Plugin VS Code
```

El orden minimiza retrabajos: fuzz valida lo existente, ag-lsp expone
capacidades, el plugin las consume con interfaz estable.

---

## 1. cargo-fuzz

### Ubicacion

```
fuzz/
|-- Cargo.toml
|-- fuzz_targets/
|   |-- fuzz_lexer.rs
|   |-- fuzz_parser.rs
|   `-- fuzz_compile.rs
```

Convencion estandar de `cargo-fuzz`. El directorio `fuzz/` vive en la raiz
del workspace.

### Targets

| Target | Entrada | Invariante |
|--------|---------|------------|
| `fuzz_lexer` | bytes arbitrarios | lexer termina sin panic |
| `fuzz_parser` | bytes arbitrarios | parser termina sin panic o retorna `Err` controlado |
| `fuzz_compile` | bytes arbitrarios | `ag_dsl::compile()` termina sin panic |

Los tres targets deben manejar cualquier entrada sin panics. Errores de
compilacion son resultado valido; panics son fallos.

### CI smoke test

Job `fuzz-smoke` en `.github/workflows/quality.yml`:

```yaml
- run: cargo fuzz run fuzz_lexer   -- -max_total_time=60
- run: cargo fuzz run fuzz_parser  -- -max_total_time=60
- run: cargo fuzz run fuzz_compile -- -max_total_time=60
```

- Corre en cada PR y push a `fase-3` y `main`
- Falla si hay crash (exit code != 0)
- Genera artefacto de reproduccion en caso de fallo
- Solo disponible en Linux (cargo-fuzz requiere nightly + libFuzzer)

### Gate manual de 24h

Documentado en `docs/fuzz/README.md`:

```bash
cargo fuzz run fuzz_lexer   -- -max_total_time=86400
cargo fuzz run fuzz_parser  -- -max_total_time=86400
cargo fuzz run fuzz_compile -- -max_total_time=86400
```

El mantenedor ejecuta las tres corridas antes de cerrar Fase 3. Los
resultados se registran en `docs/fuzz/results/` con fecha, hardware y
commit.

---

## 2. ag-lsp

### Crate

`crates/ag-lsp/` con entrada `[[bin]] name = "ag-lsp"`.

**Dependencias principales:**
- `tower-lsp` — servidor LSP async maduro, compatible con el stack Tokio
  existente
- `lsp-types` — tipos estandar del protocolo LSP
- `ag-dsl` (workspace dep) — reutiliza compilador y lint existentes

### Refactor previo: `ag_dsl::lint()`

Antes de implementar ag-lsp, la logica de `ag schema lint` se extrae del
CLI a una funcion publica en `ag-dsl`:

```rust
// crates/ag-dsl/src/lib.rs
pub fn lint(source: &str) -> Vec<LintWarning>
```

`ag-cli` pasa a ser wrapper:

```rust
// crates/ag-cli/src/commands/schema_lint.rs
let warnings = ag_dsl::lint(&source);
```

Esto elimina duplicacion y permite que ag-lsp llame lint directamente.

### Capacidades del alpha

| Metodo LSP | Comportamiento |
|------------|----------------|
| `textDocument/didOpen` | indexa el archivo `.ag`, compila y publica diagnostics |
| `textDocument/didChange` | recompila en memoria, actualiza diagnostics |
| `textDocument/publishDiagnostics` | errores del compilador (Error) + warnings de lint (Warning) |
| `textDocument/completion` | keywords (`model`, `endpoint`, `field`), anotaciones (`@primary`, `@unique`, `@auto`, `@min`, `@max`, `@email`, `@regex`, `@length`, `@relation`, `@references`), nombres de modelos definidos en el schema |
| `textDocument/hover` | muestra tipo y descripcion del campo bajo el cursor |

### Niveles de diagnostics

```
ag_dsl::compile(source)  →  Vec<Diagnostic>  →  lsp_types::Diagnostic (severity: Error)
ag_dsl::lint(source)     →  Vec<LintWarning> →  lsp_types::Diagnostic (severity: Warning)
```

Ambos se publican juntos en cada `didOpen` / `didChange`.

### Arquitectura interna

```
stdin/stdout (LSP transport — tower-lsp)
    |
Backend (impl LanguageServer)
    |-- on_open/on_change:
    |       ag_dsl::compile() + ag_dsl::lint()
    |       → PublishDiagnosticsParams
    |
    |-- completion:
    |       keyword list estatico + modelos del AST en memoria
    |
    `-- hover:
            busca field bajo cursor en AST en memoria
            → tipo + anotaciones como MarkupContent
```

### Fuera del alcance del alpha

- `goto definition`
- `rename`
- `code actions` / `quick fix`
- Soporte multi-archivo (el LSP procesa un archivo a la vez)
- Formateo automatico

Documentados como mejoras futuras en `docs/dsl/lsp-roadmap.md`.

---

## 3. Plugin VS Code

### Ubicacion

```
tools/vscode-anti-gravital/
|-- package.json
|-- tsconfig.json
|-- .vscodeignore
|-- src/
|   `-- extension.ts
|-- syntaxes/
|   `-- anti-gravital.tmLanguage.json
|-- language-configuration.json
`-- README.md
```

### Dependencias

- `vscode` (engine)
- `vscode-languageclient` — cliente LSP estandar

### Flujo de arranque

```
extension activada (archivo .ag abierto)
    |
which ag-lsp  (o where ag-lsp en Windows)
    |
encontrado ──yes──> lanza LanguageClient apuntando al binario
    |
   no
    |
notificacion VS Code:
"ag-lsp not found. Install it with cargo to enable IntelliSense."
[Install via cargo]  [Dismiss]
    |
[Install via cargo]
    → abre terminal integrada
    → ejecuta: cargo install ag-lsp
    → al terminar, reintenta arranque automaticamente
```

### Capacidades activadas

| Capacidad | Fuente |
|-----------|--------|
| Syntax highlighting `.ag` | tmLanguage grammar |
| Diagnostics Error en tiempo real | ag-lsp via LSP |
| Diagnostics Warning (lint) en tiempo real | ag-lsp via LSP |
| Autocompletado keywords y anotaciones | ag-lsp via LSP |
| Autocompletado nombres de modelos | ag-lsp via LSP |
| Hover con tipo y anotaciones | ag-lsp via LSP |

### tmLanguage — scopes cubiertos

- Keywords: `model`, `endpoint`, `field`, `request`, `response`, `error`
- Metodos HTTP: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`
- Anotaciones: `@primary`, `@unique`, `@auto`, `@min`, `@max`, `@email`,
  `@regex`, `@length`, `@relation`, `@references`, `@on_delete`,
  `@on_update`, `@default`, `@auto_update`
- Tipos primitivos: `String`, `Int`, `Float`, `Bool`, `DateTime`, `UUID`
- Strings, numeros, comentarios `//`

### Empaquetado

```bash
cd tools/vscode-anti-gravital
npm install
vsce package   # genera anti-gravital-X.Y.Z.vsix
```

El `.vsix` se instala localmente con:

```bash
code --install-extension anti-gravital-X.Y.Z.vsix
```

La publicacion al marketplace queda documentada como paso manual cuando
el repositorio sea publico y el proyecto tenga cuenta de publisher en
VS Code Marketplace.

### Fuera del alcance del alpha

- Bundled binaries por plataforma (Linux/Mac/Windows precompilados)
- Snippets
- Icono de archivo custom
- Tema de color
- Soporte multi-raiz (multi-root workspace)

---

## Checklist de cierre

Antes de marcar los tres items como completados:

- [ ] `cargo fuzz run fuzz_lexer/parser/compile -- -max_total_time=60` pasan en CI
- [ ] `ag-lsp` compila, pasa `cargo clippy`, pasa tests
- [ ] `ag_dsl::lint()` es funcion publica con tests propios
- [ ] `ag schema lint` CLI sigue funcionando (ahora como wrapper)
- [ ] Plugin VS Code compila con `tsc --noEmit`
- [ ] Plugin detecta `ag-lsp` en PATH y lanza el LanguageClient
- [ ] Plugin muestra notificacion cuando `ag-lsp` no esta en PATH
- [ ] `.vsix` se genera sin errores con `vsce package`
- [ ] `docs/fuzz/README.md` documenta corridas manuales de 24h
- [ ] `docs/dsl/lsp-roadmap.md` documenta capacidades futuras del LSP
- [ ] README actualizado con estado de Fase 3

---

## Criterios de salida de Fase 3 afectados

De `ANTI-GRAVITAL-Hoja-de-Ruta.md` seccion 3.2:

- [ ] Servidor LSP basico con autocompletado y diagnostics — cubierto por ag-lsp
- [ ] Plugin VS Code publicado en marketplace — codigo listo, publicacion manual pendiente
- [ ] Fuzzing del parser con cargo-fuzz: 24 horas sin crashes — harness listo, gate manual pendiente
