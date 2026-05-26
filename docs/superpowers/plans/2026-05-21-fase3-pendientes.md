# Fase 3 Pendientes — cargo-fuzz + ag-lsp + Plugin VS Code

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Completar los tres entregables restantes de Fase 3: harness de fuzzing del compilador DSL, servidor LSP con diagnostics en tiempo real y autocompletado, y plugin VS Code con deteccion dinamica de ag-lsp.

**Architecture:** Tres fases secuenciales. Phase 1 (cargo-fuzz): valida el compilador existente con libfuzzer, requiere exponer `ag_dsl::lint()` como API publica. Phase 2 (ag-lsp): nuevo crate `crates/ag-lsp/` con tower-lsp que reutiliza ag-dsl directamente. Phase 3 (VS Code plugin): TypeScript en `tools/vscode-anti-gravital/` que detecta ag-lsp en PATH con fallback a `cargo install ag-lsp`.

**Tech Stack:** cargo-fuzz/libfuzzer/nightly Rust (Phase 1), tower-lsp 0.20/tokio (Phase 2), TypeScript 5/vscode-languageclient 8/vsce (Phase 3).

**Spec:** `docs/superpowers/specs/2026-05-21-fase3-pendientes-design.md`

---

## Mapa de archivos

### Crear
- `fuzz/Cargo.toml`
- `fuzz/fuzz_targets/fuzz_lexer.rs`
- `fuzz/fuzz_targets/fuzz_parser.rs`
- `fuzz/fuzz_targets/fuzz_compile.rs`
- `docs/fuzz/README.md`
- `crates/ag-lsp/Cargo.toml`
- `crates/ag-lsp/src/main.rs`
- `crates/ag-lsp/src/backend.rs`
- `docs/dsl/lsp-roadmap.md`
- `tools/vscode-anti-gravital/package.json`
- `tools/vscode-anti-gravital/tsconfig.json`
- `tools/vscode-anti-gravital/.vscodeignore`
- `tools/vscode-anti-gravital/language-configuration.json`
- `tools/vscode-anti-gravital/syntaxes/anti-gravital.tmLanguage.json`
- `tools/vscode-anti-gravital/src/extension.ts`
- `tools/vscode-anti-gravital/README.md`

### Modificar
- `crates/ag-dsl/src/lib.rs` — agregar `pub fn lint()`
- `crates/ag-cli/src/main.rs` — actualizar `cmd_schema_lint` para usar `ag_dsl::lint()`
- `Cargo.toml` (root) — agregar `crates/ag-lsp` a members y `tower-lsp` a workspace.dependencies
- `.github/workflows/quality.yml` — agregar job `fuzz-smoke`
- `docs/pr-drafts/fase-3.md` — actualizar descriptor para la PR final
- `README.md` — actualizar estado Fase 3

---

## PHASE 1: cargo-fuzz

---

### Task 1: Agregar `ag_dsl::lint()` como API publica

`lint()` ejecuta el pipeline completo (lex + parse + semantic) y retorna TODOS
los diagnosticos (errores y warnings) sin bloquear en errores. A diferencia de
`compile()`, no descarta warnings cuando no hay errores.

**Files:**
- Modify: `crates/ag-dsl/src/lib.rs`

- [ ] **Step 1: Escribir el test que falla**

Agregar al bloque `#[cfg(test)]` en `crates/ag-dsl/src/lib.rs`:

```rust
#[test]
fn lint_returns_warnings_for_model_without_primary() {
    // "model Tag { name String }" compila OK pero deberia tener warning de @primary
    let src = "model Tag { name String }";
    let diags = lint(src);
    // compile() retorna Ok (warnings no bloquean), lint() debe retornar al menos 1 warning
    assert!(
        !diags.is_empty(),
        "lint debe retornar warnings aunque compile() retorne Ok"
    );
    assert!(
        diags.iter().all(|d| !d.is_error()),
        "para este schema solo deben ser warnings, no errores"
    );
}

#[test]
fn lint_returns_errors_for_invalid_schema() {
    let src = "model Bad { id UUID @primary @auto @min(1) }";
    let diags = lint(src);
    assert!(diags.iter().any(|d| d.is_error()));
}

#[test]
fn lint_returns_empty_for_clean_schema() {
    let src = r#"
model User {
    id    UUID   @primary @auto
    email String @unique @email @max(255)
    name  String @min(2)
}
"#;
    let diags = lint(src);
    let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
    assert!(errors.is_empty(), "schema limpio no debe tener errores");
}
```

- [ ] **Step 2: Correr para verificar que falla**

```bash
cargo test -p ag-dsl lint 2>&1 | head -20
```

Resultado esperado: `error[E0425]: cannot find function lint in this scope`

- [ ] **Step 3: Implementar `lint()` en `crates/ag-dsl/src/lib.rs`**

Agregar inmediatamente despues de la funcion `compile()` (linea ~89):

```rust
/// Analiza el fuente DSL y retorna todos los diagnosticos (errores y warnings).
///
/// A diferencia de `compile()`, siempre retorna la lista completa: no descarta
/// warnings cuando no hay errores. Usar en el servidor LSP y en `ag schema lint`.
pub fn lint(source: &str) -> Vec<Diagnostic> {
    let (tokens, lex_spans) = lexer::tokenize(source);
    let mut all_diags: Vec<Diagnostic> =
        lex_spans.into_iter().map(Diagnostic::lex_error).collect();

    let (ast, parse_diags) = parser::parse_tokens(tokens, source.len());
    all_diags.extend(parse_diags);

    if let Some(schema) = ast {
        all_diags.extend(semantic::analyze(&schema));
    }

    all_diags
}
```

- [ ] **Step 4: Correr para verificar que pasa**

```bash
cargo test -p ag-dsl lint
```

Resultado esperado: `test tests::lint_returns_warnings_for_model_without_primary ... ok`

- [ ] **Step 5: Commit**

```bash
git add crates/ag-dsl/src/lib.rs
git commit -m "feat(dsl): expone lint() como API publica — retorna todos los diagnosticos"
```

---

### Task 2: Actualizar `cmd_schema_lint` en ag-cli para usar `ag_dsl::lint()`

**Files:**
- Modify: `crates/ag-cli/src/main.rs`

- [ ] **Step 1: Escribir test de regresion**

Agregar al bloque `#[cfg(test)]` de `crates/ag-cli/src/main.rs`:

```rust
#[test]
fn lint_fn_surfaces_warnings_for_model_without_primary() {
    let src = "model Tag { name String }";
    let diags = ag_dsl::lint(src);
    assert!(!diags.is_empty(), "debe haber warnings para modelo sin @primary");
}
```

- [ ] **Step 2: Correr para verificar que pasa**

```bash
cargo test -p ag-cli lint_fn_surfaces
```

- [ ] **Step 3: Actualizar `cmd_schema_lint`**

Localizar la funcion `cmd_schema_lint` en `crates/ag-cli/src/main.rs` (linea ~336) y reemplazarla:

```rust
fn cmd_schema_lint(schema_path: &Path) -> Result<(), String> {
    let source = read_schema(schema_path)?;
    let diags = ag_dsl::lint(&source);

    if diags.is_empty() {
        println!("'{}': sin problemas encontrados.", schema_path.display());
        return Ok(());
    }

    let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
    let warnings: Vec<_> = diags.iter().filter(|d| !d.is_error()).collect();

    for w in &warnings {
        println!("warning: {}", w.display(&source));
    }
    for e in &errors {
        eprintln!("error:   {}", e.display(&source));
    }

    if !errors.is_empty() {
        return Err(format!(
            "{} error(es) en '{}'",
            errors.len(),
            schema_path.display()
        ));
    }
    Ok(())
}
```

- [ ] **Step 4: Correr tests del CLI**

```bash
cargo test -p ag-cli && cargo clippy -p ag-cli -- -D warnings
```

Resultado esperado: todos los tests pasan, clippy sin warnings.

- [ ] **Step 5: Commit**

```bash
git add crates/ag-cli/src/main.rs
git commit -m "refactor(cli): schema lint usa ag_dsl::lint() directamente"
```

---

### Task 3: Crear el crate de fuzzing

**Files:**
- Create: `fuzz/Cargo.toml`
- Create: `fuzz/fuzz_targets/fuzz_lexer.rs`
- Create: `fuzz/fuzz_targets/fuzz_parser.rs`
- Create: `fuzz/fuzz_targets/fuzz_compile.rs`

- [ ] **Step 1: Crear `fuzz/Cargo.toml`**

```toml
[package]
name = "ag-dsl-fuzz"
version = "0.0.0"
edition = "2021"
publish = false

[[bin]]
name = "fuzz_lexer"
path = "fuzz_targets/fuzz_lexer.rs"
test = false
doc = false

[[bin]]
name = "fuzz_parser"
path = "fuzz_targets/fuzz_parser.rs"
test = false
doc = false

[[bin]]
name = "fuzz_compile"
path = "fuzz_targets/fuzz_compile.rs"
test = false
doc = false

[dependencies]
libfuzzer-sys = "0.4"
ag-dsl = { path = "../crates/ag-dsl" }

[workspace]
```

- [ ] **Step 2: Crear `fuzz/fuzz_targets/fuzz_lexer.rs`**

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // El lexer nunca debe entrar en panico con texto UTF-8 arbitrario
        let _ = ag_dsl::lint(s);
    }
});
```

- [ ] **Step 3: Crear `fuzz/fuzz_targets/fuzz_parser.rs`**

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // El parser nunca debe entrar en panico con entrada arbitraria
        let _ = ag_dsl::lint(s);
    }
});
```

- [ ] **Step 4: Crear `fuzz/fuzz_targets/fuzz_compile.rs`**

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // El pipeline completo (lex+parse+semantic+codegen) no debe entrar en panico
        let diags = ag_dsl::lint(s);
        if diags.iter().all(|d| !d.is_error()) {
            if let Ok(schema) = ag_dsl::compile(s) {
                let _ = ag_dsl::generate(&schema);
            }
        }
    }
});
```

- [ ] **Step 5: Verificar que el crate compila con nightly**

```bash
cd fuzz && cargo +nightly build 2>&1 | tail -5
```

Resultado esperado: `Compiling ag-dsl-fuzz v0.0.0` sin errores.

- [ ] **Step 6: Correr fuzz_compile por 10 segundos como smoke local**

```bash
cargo +nightly fuzz run fuzz_compile -- -max_total_time=10
```

Resultado esperado: termina sin `CRASH` ni `panic`.

- [ ] **Step 7: Commit**

```bash
cd ..
git add fuzz/
git commit -m "test(dsl): cargo-fuzz — 3 targets fuzz_lexer/parser/compile"
```

---

### Task 4: CI smoke test de fuzzing + documentacion

**Files:**
- Modify: `.github/workflows/quality.yml`
- Create: `docs/fuzz/README.md`

- [ ] **Step 1: Agregar job `fuzz-smoke` en `.github/workflows/quality.yml`**

Agregar al final del archivo, despues del job `deny`:

```yaml
  fuzz-smoke:
    name: cargo fuzz smoke (60s)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: nightly
      - name: install cargo-fuzz
        run: cargo install cargo-fuzz --locked
      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            fuzz/target
          key: ${{ runner.os }}-fuzz-${{ hashFiles('fuzz/Cargo.toml') }}
      - name: fuzz fuzz_lexer 60s
        working-directory: fuzz
        run: cargo fuzz run fuzz_lexer -- -max_total_time=60
      - name: fuzz fuzz_parser 60s
        working-directory: fuzz
        run: cargo fuzz run fuzz_parser -- -max_total_time=60
      - name: fuzz fuzz_compile 60s
        working-directory: fuzz
        run: cargo fuzz run fuzz_compile -- -max_total_time=60
      - name: upload crash artifacts
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: fuzz-crashes
          path: fuzz/artifacts/
```

- [ ] **Step 2: Crear `docs/fuzz/README.md`**

```markdown
# Fuzzing del compilador Anti-DSL

## Smoke test continuo (CI)

El job `fuzz-smoke` en `.github/workflows/quality.yml` ejecuta cada target
por 60 segundos en cada PR y push. Verifica que el harness no este roto.

## Gate manual de 24 horas (requerido antes de cerrar Fase 3)

Ejecutar en hardware Linux x86-64 antes de mergear la rama `fase-3`:

    cd fuzz

    cargo +nightly fuzz run fuzz_lexer   -- -max_total_time=86400
    cargo +nightly fuzz run fuzz_parser  -- -max_total_time=86400
    cargo +nightly fuzz run fuzz_compile -- -max_total_time=86400

Registrar el resultado en `docs/fuzz/results/YYYY-MM-DD.md` con:

- Fecha y hora de inicio/fin
- Hardware (CPU, RAM, OS)
- Version de Rust nightly usada
- Commit del repositorio
- Resultado: sin crashes / lista de crashes encontrados

## Targets

| Target | Invariante verificado |
|--------|----------------------|
| fuzz_lexer | El lexer termina sin panic en cualquier UTF-8 |
| fuzz_parser | El parser termina sin panic o retorna Err controlado |
| fuzz_compile | El pipeline completo (lint + compile + generate) no panics |

## Reproducir un crash

Si el CI sube artefactos en `fuzz/artifacts/`:

    cd fuzz
    cargo +nightly fuzz run fuzz_compile artifacts/fuzz_compile/<crash-file>
```

- [ ] **Step 3: Verificar que el YAML es valido**

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/quality.yml'))" && echo "YAML OK"
```

Resultado esperado: `YAML OK`

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/quality.yml docs/fuzz/
git commit -m "ci(fuzz): smoke test 60s por target en quality.yml; docs en docs/fuzz/"
```

---

## PHASE 2: ag-lsp

---

### Task 5: Registrar ag-lsp en el workspace y crear Cargo.toml

**Files:**
- Modify: `Cargo.toml` (root)
- Create: `crates/ag-lsp/Cargo.toml`

- [ ] **Step 1: Agregar `ag-lsp` y `tower-lsp` al workspace**

En `Cargo.toml` (root), agregar `"crates/ag-lsp"` a la lista `members` (despues de `"crates/ag-wasm-host"`):

```toml
    "crates/ag-wasm-host",
    "crates/ag-lsp",
    "examples/todo-api",
```

Y agregar a `[workspace.dependencies]`:

```toml
tower-lsp = "0.20"
```

- [ ] **Step 2: Crear `crates/ag-lsp/Cargo.toml`**

```toml
[package]
name = "ag-lsp"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
homepage.workspace = true
documentation.workspace = true
readme = "README.md"
description = "Servidor LSP del Anti-DSL: diagnostics y autocompletado para archivos .ag"
keywords.workspace = true
categories.workspace = true
publish = false

[[bin]]
name = "ag-lsp"
path = "src/main.rs"

[lints]
workspace = true

[dependencies]
ag-dsl    = { workspace = true }
tower-lsp = { workspace = true }
tokio     = { workspace = true }
```

- [ ] **Step 3: Verificar que el workspace resuelve dependencias**

```bash
cargo check -p ag-lsp 2>&1 | head -10
```

Resultado esperado: errores de codigo faltante pero NO errores de dependencias.
`error[E0601]: main function not found` es esperado en este paso.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml crates/ag-lsp/Cargo.toml
git commit -m "chore(lsp): registra ag-lsp en workspace; agrega tower-lsp 0.20"
```

---

### Task 6: Crear `backend.rs` — Backend LSP completo

**Files:**
- Create: `crates/ag-lsp/src/backend.rs`

- [ ] **Step 1: Escribir los tests de backend primero**

Crear `crates/ag-lsp/src/backend.rs` con solo el bloque de tests:

```rust
//! Backend del servidor LSP Anti-Gravital.

#[cfg(test)]
mod tests {
    #[test]
    fn byte_to_position_first_line() {
        // placeholder — implementar junto con la funcion
    }
}
```

- [ ] **Step 2: Verificar que compila (aunque los tests sean placeholder)**

```bash
cargo test -p ag-lsp 2>&1 | head -10
```

- [ ] **Step 3: Implementar `backend.rs` completo**

Reemplazar el contenido de `crates/ag-lsp/src/backend.rs`:

```rust
//! Backend del servidor LSP Anti-Gravital.
//!
//! Implementa el trait `LanguageServer` de tower-lsp. Cada documento `.ag`
//! abierto se mantiene en memoria para publicar diagnostics en tiempo real.

use std::collections::HashMap;

use ag_dsl::Diagnostic as AgDiag;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    Hover, HoverContents, HoverParams, HoverProviderCapability, InitializeParams, InitializeResult,
    InitializedParams, MarkupContent, MarkupKind, MessageType, Position, Range, ServerCapabilities,
    ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer};

/// Estado interno del servidor LSP.
pub struct Backend {
    client: Client,
    /// Texto actual de cada documento abierto, indexado por URI.
    documents: Mutex<HashMap<Url, String>>,
}

impl Backend {
    /// Crea un nuevo Backend con el cliente LSP dado.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Mutex::new(HashMap::new()),
        }
    }

    async fn publish_diagnostics(&self, uri: Url, source: &str) {
        let lsp_diags: Vec<Diagnostic> = ag_dsl::lint(source)
            .iter()
            .map(|d| ag_diag_to_lsp(source, d))
            .collect();
        self.client.publish_diagnostics(uri, lsp_diags, None).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["@".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "ag-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "ag-lsp inicializado")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.documents.lock().await.insert(uri.clone(), text.clone());
        self.publish_diagnostics(uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text;
            self.documents.lock().await.insert(uri.clone(), text.clone());
            self.publish_diagnostics(uri, &text).await;
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let source = self.documents.lock().await.get(&uri).cloned().unwrap_or_default();

        let mut items = static_completion_items();
        if let Ok(schema) = ag_dsl::compile(&source) {
            for model in &schema.models {
                items.push(CompletionItem {
                    label: model.name.value.clone(),
                    kind: Some(CompletionItemKind::CLASS),
                    detail: Some("Modelo AG".into()),
                    ..Default::default()
                });
            }
        }
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let source = match self.documents.lock().await.get(&uri).cloned() {
            Some(s) => s,
            None => return Ok(None),
        };

        let content = word_at_position(&source, &pos).and_then(hover_content_for_word);
        Ok(content.map(|text| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: text,
            }),
            range: None,
        }))
    }
}

// ---- helpers internos ------------------------------------------------

fn ag_diag_to_lsp(source: &str, d: &AgDiag) -> Diagnostic {
    let range = span_to_range(source, d.span.start, d.span.end);
    let severity = if d.is_error() {
        DiagnosticSeverity::ERROR
    } else {
        DiagnosticSeverity::WARNING
    };
    let mut message = d.message.clone();
    if let Some(hint) = &d.hint {
        message.push_str(&format!("\nAyuda: {hint}"));
    }
    Diagnostic {
        range,
        severity: Some(severity),
        message,
        source: Some("ag-lsp".to_string()),
        ..Default::default()
    }
}

fn span_to_range(source: &str, start: usize, end: usize) -> Range {
    Range {
        start: byte_to_position(source, start),
        end: byte_to_position(source, end),
    }
}

fn byte_to_position(source: &str, offset: usize) -> Position {
    let offset = offset.min(source.len());
    let before = &source[..offset];
    let line = before.bytes().filter(|&b| b == b'\n').count() as u32;
    let last_nl = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let character = before[last_nl..].chars().count() as u32;
    Position { line, character }
}

fn word_at_position<'a>(source: &'a str, pos: &Position) -> Option<&'a str> {
    let line = source.lines().nth(pos.line as usize)?;
    let ch = (pos.character as usize).min(line.len());
    let start = line[..ch]
        .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != '@')
        .map(|i| i + 1)
        .unwrap_or(0);
    let end = line[ch..]
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| ch + i)
        .unwrap_or(line.len());
    if start >= end { None } else { Some(&line[start..end]) }
}

fn hover_content_for_word(word: &str) -> Option<String> {
    let s = match word {
        "UUID"         => "**UUID** — Identificador unico universal v4. SQL: `UUID`.",
        "String"       => "**String** — Cadena de texto UTF-8. SQL: `TEXT`.",
        "Int"          => "**Int** — Entero de 64 bits con signo. SQL: `BIGINT`.",
        "Float"        => "**Float** — Punto flotante 64 bits. SQL: `DOUBLE PRECISION`.",
        "Bool"         => "**Bool** — Valor booleano. SQL: `BOOLEAN`.",
        "DateTime"     => "**DateTime** — Fecha y hora UTC. SQL: `TIMESTAMPTZ`.",
        "@primary"     => "**@primary** — Clave primaria de la tabla.",
        "@unique"      => "**@unique** — Restriccion UNIQUE en la columna.",
        "@auto"        => "**@auto** — Valor generado automaticamente (UUID o serial).",
        "@auto_update" => "**@auto_update** — Se actualiza automaticamente al modificar la fila.",
        "@default"     => "**@default(valor)** — Valor por defecto de la columna.",
        "@min"         => "**@min(n)** — En strings: longitud minima. En numeros: valor minimo.",
        "@max"         => "**@max(n)** — En strings: longitud maxima. En numeros: valor maximo.",
        "@email"       => "**@email** — Valida formato de correo electronico (RFC 5322).",
        "@regex"       => "**@regex(\"patron\")** — Valida contra expresion regular.",
        "@length"      => "**@length(n)** — Longitud exacta del string.",
        "@relation"    => "**@relation(campo_fk)** — Relacion virtual. No genera columna SQL.",
        "@references"  => "**@references(Modelo.campo)** — Clave foranea hacia otro modelo.",
        "@on_delete"   => "**@on_delete(CASCADE|SET_NULL|RESTRICT)** — Comportamiento al eliminar fila referenciada.",
        "@on_update"   => "**@on_update(CASCADE|SET_NULL|RESTRICT)** — Comportamiento al actualizar fila referenciada.",
        _              => return None,
    };
    Some(s.to_string())
}

fn static_completion_items() -> Vec<CompletionItem> {
    let keywords: &[(&str, &str)] = &[
        ("model",    "Definicion de modelo de datos"),
        ("endpoint", "Definicion de endpoint HTTP"),
        ("request",  "Tipo de cuerpo de peticion"),
        ("response", "Tipo de cuerpo de respuesta"),
        ("error",    "Tipo de error HTTP"),
        ("config",   "Bloque de configuracion del proyecto"),
        ("GET",      "Metodo HTTP GET"),
        ("POST",     "Metodo HTTP POST"),
        ("PUT",      "Metodo HTTP PUT"),
        ("PATCH",    "Metodo HTTP PATCH"),
        ("DELETE",   "Metodo HTTP DELETE"),
    ];
    let types: &[(&str, &str)] = &[
        ("UUID",     "Identificador unico universal"),
        ("String",   "Cadena de texto UTF-8"),
        ("Int",      "Entero de 64 bits con signo"),
        ("Float",    "Punto flotante de 64 bits"),
        ("Bool",     "Valor booleano"),
        ("DateTime", "Fecha y hora UTC"),
    ];
    let annotations: &[(&str, &str)] = &[
        ("@primary",     "Clave primaria"),
        ("@unique",      "Indice unico"),
        ("@auto",        "Valor generado automaticamente"),
        ("@auto_update", "Actualizado automaticamente"),
        ("@default",     "@default(valor) — valor por defecto"),
        ("@min",         "@min(n) — valor minimo o longitud minima"),
        ("@max",         "@max(n) — valor maximo o longitud maxima"),
        ("@email",       "Valida formato de email"),
        ("@regex",       "@regex(\"patron\") — expresion regular"),
        ("@length",      "@length(n) — longitud exacta"),
        ("@relation",    "@relation(campo_fk) — relacion virtual"),
        ("@references",  "@references(Modelo.campo) — clave foranea"),
        ("@on_delete",   "@on_delete(CASCADE|SET_NULL|RESTRICT)"),
        ("@on_update",   "@on_update(CASCADE|SET_NULL|RESTRICT)"),
    ];

    let mut items = Vec::new();
    for (label, detail) in keywords {
        items.push(CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(detail.to_string()),
            ..Default::default()
        });
    }
    for (label, detail) in types {
        items.push(CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some(detail.to_string()),
            ..Default::default()
        });
    }
    for (label, detail) in annotations {
        items.push(CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some(detail.to_string()),
            ..Default::default()
        });
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_to_position_first_line() {
        let src = "model User {}";
        let pos = byte_to_position(src, 0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn byte_to_position_second_line() {
        // 14 bytes = inicio de la segunda linea (tras '\n' en indice 13)
        let src = "model User {\n    id UUID\n}";
        let pos = byte_to_position(src, 14);
        assert_eq!(pos.line, 1);
    }

    #[test]
    fn word_at_position_returns_type_name() {
        let src = "    id UUID @primary";
        //                   ^ character 7 cae dentro de "UUID"
        let pos = Position { line: 0, character: 8 };
        assert_eq!(word_at_position(src, &pos), Some("UUID"));
    }

    #[test]
    fn word_at_position_returns_annotation() {
        let src = "    id UUID @primary";
        //                        ^ character 13 cae dentro de "@primary"
        let pos = Position { line: 0, character: 14 };
        assert_eq!(word_at_position(src, &pos), Some("@primary"));
    }

    #[test]
    fn hover_content_known_type() {
        assert!(hover_content_for_word("UUID").is_some());
        assert!(hover_content_for_word("DateTime").is_some());
    }

    #[test]
    fn hover_content_known_annotation() {
        assert!(hover_content_for_word("@primary").is_some());
        assert!(hover_content_for_word("@references").is_some());
    }

    #[test]
    fn hover_content_unknown_is_none() {
        assert!(hover_content_for_word("foobar").is_none());
    }

    #[test]
    fn static_items_include_keywords_types_annotations() {
        let items = static_completion_items();
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"model"));
        assert!(labels.contains(&"UUID"));
        assert!(labels.contains(&"@primary"));
        assert!(labels.contains(&"@references"));
    }

    #[test]
    fn ag_diag_error_maps_to_lsp_error() {
        let src = "model Bad { id UUID @primary @auto @min(1) }";
        let diags = ag_dsl::lint(src);
        let err = diags.iter().find(|d| d.is_error()).expect("debe haber error");
        let lsp = ag_diag_to_lsp(src, err);
        assert_eq!(lsp.severity, Some(DiagnosticSeverity::ERROR));
    }

    #[test]
    fn ag_diag_warning_maps_to_lsp_warning() {
        let src = "model Tag { name String }";
        let diags = ag_dsl::lint(src);
        let warn = diags.iter().find(|d| !d.is_error()).expect("debe haber warning");
        let lsp = ag_diag_to_lsp(src, warn);
        assert_eq!(lsp.severity, Some(DiagnosticSeverity::WARNING));
    }
}
```

- [ ] **Step 4: Correr tests de backend**

```bash
cargo test -p ag-lsp 2>&1 | tail -20
```

Resultado esperado: todos los tests del modulo `tests` pasan.

- [ ] **Step 5: Commit**

```bash
git add crates/ag-lsp/src/backend.rs
git commit -m "feat(lsp): backend — diagnostics, completion, hover; 10 tests"
```

---

### Task 7: Binario `ag-lsp`, clippy completo y documentacion LSP

**Files:**
- Create: `crates/ag-lsp/src/main.rs`
- Create: `docs/dsl/lsp-roadmap.md`

- [ ] **Step 1: Crear `crates/ag-lsp/src/main.rs`**

```rust
//! Binario del servidor LSP Anti-Gravital.
//!
//! Escucha en stdin/stdout con el protocolo LSP. El cliente (VS Code, etc.)
//! lanza este proceso como hijo y se comunica por stdio.

mod backend;

use backend::Backend;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
```

- [ ] **Step 2: Compilar el binario**

```bash
cargo build -p ag-lsp 2>&1 | tail -5
```

Resultado esperado: `Finished` sin errores.

- [ ] **Step 3: Pasar clippy en el crate**

```bash
cargo clippy -p ag-lsp -- -D warnings
```

Resultado esperado: sin warnings. Si hay `missing_docs` en Backend, ya tiene
doc en el Step 3 de Task 6. Si aparece alguno nuevo, agregar el comentario.

- [ ] **Step 4: Smoke test del binario LSP**

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"rootUri":null,"processId":null}}' \
  | timeout 3 ./target/debug/ag-lsp 2>/dev/null | head -c 200
```

Resultado esperado: respuesta JSON con `"serverInfo":{"name":"ag-lsp"`.

- [ ] **Step 5: Crear `docs/dsl/lsp-roadmap.md`**

```markdown
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
```

- [ ] **Step 6: Correr suite completa del workspace**

```bash
cargo test --workspace 2>&1 | tail -10
```

Resultado esperado: todos los tests pasan. El total de tests es mayor al anterior.

- [ ] **Step 7: Commit**

```bash
git add crates/ag-lsp/src/main.rs docs/dsl/lsp-roadmap.md
git commit -m "feat(lsp): binario ag-lsp stdio; smoke test pasa; lsp-roadmap"
```

---

## PHASE 3: Plugin VS Code

---

### Task 8: Scaffold — package.json, tsconfig, language-configuration

**Files:**
- Create: `tools/vscode-anti-gravital/package.json`
- Create: `tools/vscode-anti-gravital/tsconfig.json`
- Create: `tools/vscode-anti-gravital/language-configuration.json`
- Create: `tools/vscode-anti-gravital/.vscodeignore`

- [ ] **Step 1: Crear directorios**

```bash
mkdir -p tools/vscode-anti-gravital/src
mkdir -p tools/vscode-anti-gravital/syntaxes
```

- [ ] **Step 2: Crear `tools/vscode-anti-gravital/package.json`**

```json
{
  "name": "anti-gravital",
  "displayName": "Anti-Gravital DSL",
  "description": "Language support for .ag schema files — syntax highlighting, diagnostics, and IntelliSense via ag-lsp",
  "version": "0.1.0",
  "publisher": "gravital-labs",
  "engines": { "vscode": "^1.75.0" },
  "categories": ["Programming Languages"],
  "repository": {
    "type": "git",
    "url": "https://github.com/anti-gravital/anti-gravital"
  },
  "activationEvents": ["onLanguage:anti-gravital"],
  "main": "./out/extension.js",
  "contributes": {
    "languages": [
      {
        "id": "anti-gravital",
        "aliases": ["Anti-Gravital DSL", "anti-gravital"],
        "extensions": [".ag"],
        "configuration": "./language-configuration.json"
      }
    ],
    "grammars": [
      {
        "language": "anti-gravital",
        "scopeName": "source.anti-gravital",
        "path": "./syntaxes/anti-gravital.tmLanguage.json"
      }
    ]
  },
  "scripts": {
    "compile": "tsc -p ./",
    "watch": "tsc -watch -p ./",
    "package": "vsce package"
  },
  "dependencies": {
    "vscode-languageclient": "^8.1.0"
  },
  "devDependencies": {
    "@types/vscode": "^1.75.0",
    "@vscode/vsce": "^2.22.0",
    "typescript": "^5.3.0"
  }
}
```

- [ ] **Step 3: Crear `tools/vscode-anti-gravital/tsconfig.json`**

```json
{
  "compilerOptions": {
    "module": "commonjs",
    "target": "ES2020",
    "outDir": "out",
    "lib": ["ES2020"],
    "sourceMap": true,
    "rootDir": "src",
    "strict": true,
    "esModuleInterop": true
  },
  "exclude": ["node_modules", ".vscode-test"]
}
```

- [ ] **Step 4: Crear `tools/vscode-anti-gravital/language-configuration.json`**

```json
{
  "comments": { "lineComment": "#" },
  "brackets": [["{", "}"], ["[", "]"], ["(", ")"]],
  "autoClosingPairs": [
    { "open": "{", "close": "}" },
    { "open": "[", "close": "]" },
    { "open": "(", "close": ")" },
    { "open": "\"", "close": "\"" }
  ],
  "wordPattern": "[a-zA-Z_][a-zA-Z0-9_]*|@[a-zA-Z_][a-zA-Z0-9_]*"
}
```

- [ ] **Step 5: Crear `tools/vscode-anti-gravital/.vscodeignore`**

```
.vscode/**
.vscode-test/**
src/**
.gitignore
tsconfig.json
node_modules/**
out/extension.js.map
```

- [ ] **Step 6: Instalar dependencias**

```bash
cd tools/vscode-anti-gravital && npm install
```

Resultado esperado: `node_modules/` creado sin errores npm.

- [ ] **Step 7: Commit**

```bash
cd ../..
git add tools/vscode-anti-gravital/package.json tools/vscode-anti-gravital/package-lock.json tools/vscode-anti-gravital/tsconfig.json tools/vscode-anti-gravital/language-configuration.json tools/vscode-anti-gravital/.vscodeignore
git commit -m "chore(vscode): scaffold plugin — package.json, tsconfig, language-config"
```

---

### Task 9: tmLanguage — syntax highlighting para `.ag`

**Files:**
- Create: `tools/vscode-anti-gravital/syntaxes/anti-gravital.tmLanguage.json`

- [ ] **Step 1: Crear el grammar**

```json
{
  "$schema": "https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json",
  "name": "Anti-Gravital DSL",
  "scopeName": "source.anti-gravital",
  "patterns": [
    { "include": "#comments" },
    { "include": "#strings" },
    { "include": "#numbers" },
    { "include": "#annotations" },
    { "include": "#types" },
    { "include": "#http-methods" },
    { "include": "#keywords" },
    { "include": "#identifiers" }
  ],
  "repository": {
    "comments": {
      "name": "comment.line.number-sign.anti-gravital",
      "match": "#.*$"
    },
    "strings": {
      "name": "string.quoted.double.anti-gravital",
      "begin": "\"",
      "end": "\"",
      "patterns": [
        { "name": "constant.character.escape.anti-gravital", "match": "\\\\." }
      ]
    },
    "numbers": {
      "name": "constant.numeric.anti-gravital",
      "match": "\\b[0-9]+(\\.[0-9]+)?\\b"
    },
    "annotations": {
      "name": "entity.other.attribute-name.anti-gravital",
      "match": "@(primary|unique|auto_update|auto|default|min|max|email|regex|length|on_delete|on_update|references|relation)\\b"
    },
    "types": {
      "name": "support.type.anti-gravital",
      "match": "\\b(UUID|String|Int|Float|Bool|DateTime)\\b"
    },
    "http-methods": {
      "name": "keyword.operator.anti-gravital",
      "match": "\\b(GET|POST|PUT|PATCH|DELETE)\\b"
    },
    "keywords": {
      "name": "keyword.control.anti-gravital",
      "match": "\\b(model|endpoint|request|response|error|config|field|method|path|body|errors|status|message|project_name|database)\\b"
    },
    "identifiers": {
      "name": "entity.name.type.anti-gravital",
      "match": "\\b[A-Z][a-zA-Z0-9_]*\\b"
    }
  }
}
```

- [ ] **Step 2: Verificar que el JSON es valido**

```bash
node -e "JSON.parse(require('fs').readFileSync('tools/vscode-anti-gravital/syntaxes/anti-gravital.tmLanguage.json','utf8')); console.log('JSON OK')"
```

Resultado esperado: `JSON OK`

- [ ] **Step 3: Commit**

```bash
git add tools/vscode-anti-gravital/syntaxes/
git commit -m "feat(vscode): tmLanguage grammar — syntax highlighting .ag"
```

---

### Task 10: `extension.ts` — LSP client, deteccion PATH, fallback cargo install

**Files:**
- Create: `tools/vscode-anti-gravital/src/extension.ts`

Nota de seguridad: usar `execFile` (no `exec`) para evitar inyeccion de shell.
El nombre del binario `'ag-lsp'` es una constante interna, pero `execFile`
es la practica correcta con child_process.

- [ ] **Step 1: Crear `tools/vscode-anti-gravital/src/extension.ts`**

```typescript
import { execFile } from 'child_process';
import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const serverPath = await resolveServerPath();
  if (!serverPath) {
    return;
  }

  const serverOptions: ServerOptions = {
    run:   { command: serverPath, transport: TransportKind.stdio },
    debug: { command: serverPath, transport: TransportKind.stdio },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'anti-gravital' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.ag'),
    },
  };

  client = new LanguageClient(
    'anti-gravital-lsp',
    'Anti-Gravital LSP',
    serverOptions,
    clientOptions,
  );

  context.subscriptions.push(client);
  await client.start();
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
  }
}

async function resolveServerPath(): Promise<string | undefined> {
  const found = await findInPath('ag-lsp');
  if (found) {
    return found;
  }

  const choice = await vscode.window.showInformationMessage(
    'ag-lsp not found. Install it with cargo to enable IntelliSense for .ag files.',
    'Install via cargo',
    'Dismiss',
  );

  if (choice !== 'Install via cargo') {
    return undefined;
  }

  const terminal = vscode.window.createTerminal('Install ag-lsp');
  terminal.show();
  terminal.sendText('cargo install ag-lsp');

  const reload = await vscode.window.showInformationMessage(
    'Installing ag-lsp in the terminal. Reload the window after installation completes.',
    'Reload Window',
  );
  if (reload === 'Reload Window') {
    await vscode.commands.executeCommand('workbench.action.reloadWindow');
  }

  return undefined;
}

// Usa execFile (no exec) para evitar interpretacion de shell.
// El argumento binary es siempre una constante interna ('ag-lsp').
function findInPath(binary: string): Promise<string | undefined> {
  const cmd = process.platform === 'win32' ? 'where' : 'which';
  return new Promise((resolve) => {
    execFile(cmd, [binary], (error, stdout) => {
      if (error || !stdout.trim()) {
        resolve(undefined);
      } else {
        resolve(stdout.trim().split('\n')[0].trim());
      }
    });
  });
}
```

- [ ] **Step 2: Compilar TypeScript**

```bash
cd tools/vscode-anti-gravital && npm run compile 2>&1
```

Resultado esperado: `out/extension.js` creado sin errores.

- [ ] **Step 3: Verificar tipos sin emitir**

```bash
npx tsc --noEmit && echo "TypeScript OK"
```

Resultado esperado: `TypeScript OK`

- [ ] **Step 4: Commit**

```bash
cd ../..
git add tools/vscode-anti-gravital/src/
git commit -m "feat(vscode): extension.ts — LSP client, deteccion PATH, fallback cargo install"
```

---

### Task 11: README, vsce package, PR draft y README principal

**Files:**
- Create: `tools/vscode-anti-gravital/README.md`
- Create: `tools/vscode-anti-gravital/.gitignore`
- Modify: `README.md` (root)
- Modify: `docs/pr-drafts/fase-3.md`

- [ ] **Step 1: Crear `tools/vscode-anti-gravital/README.md`**

```markdown
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
```

- [ ] **Step 2: Crear `tools/vscode-anti-gravital/.gitignore`**

```
out/
node_modules/
*.vsix
```

- [ ] **Step 3: Empaquetar el plugin**

```bash
cd tools/vscode-anti-gravital && npx vsce package --no-dependencies 2>&1 | tail -5
```

Resultado esperado: `Packaged: .../anti-gravital-0.1.0.vsix` sin errores.

- [ ] **Step 4: Actualizar `README.md` (root)**

Localizar la tabla de estado de Fase 3 en el README y actualizar los tres
entregables pendientes. El estado debe reflejar:

- cargo-fuzz: harness completo, smoke CI activo; gate manual 24h pendiente de ejecucion
- ag-lsp: completo, binario disponible
- Plugin VS Code: completo, publicacion en marketplace pendiente de repo publico

- [ ] **Step 5: Actualizar `docs/pr-drafts/fase-3.md`**

Leer el archivo actual y agregar/actualizar la seccion de entregables con:

```
Titulo: feat(fase-3): completa cargo-fuzz, ag-lsp y plugin VS Code
Fase: 3 — Anti-DSL alpha
Tipo: feature
Documentos relacionados:
  - docs/superpowers/specs/2026-05-21-fase3-pendientes-design.md
  - docs/fuzz/README.md
  - docs/dsl/lsp-roadmap.md
Plan de prueba:
  - cargo test --workspace (todos los tests pasan)
  - cargo clippy --workspace -- -D warnings (sin warnings)
  - cargo fuzz run fuzz_compile -- -max_total_time=60 (en CI)
  - tsc --noEmit en tools/vscode-anti-gravital (sin errores)
  - vsce package genera .vsix sin errores
Criterios de salida avanzados:
  - Servidor LSP responde correctamente a initialize
  - Diagnostics se publican al abrir un .ag con errores
  - Completion retorna keywords y anotaciones
Pendiente (comunitario):
  - Publicacion en VS Code marketplace (requiere repo publico)
  - Gate manual 24h de fuzzing antes de cerrar Fase 3
  - 100 instalaciones del plugin (criterio de salida comunitario)
```

- [ ] **Step 6: Commit**

```bash
cd ../..
git add tools/vscode-anti-gravital/README.md tools/vscode-anti-gravital/.gitignore README.md docs/pr-drafts/fase-3.md
git commit -m "docs(fase-3): README plugin, actualiza README principal y PR draft"
```

---

### Task 12: Verificacion final

- [ ] **Step 1: cargo fmt**

```bash
cargo fmt --all
```

- [ ] **Step 2: cargo clippy workspace**

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Resultado esperado: sin warnings ni errores.

- [ ] **Step 3: cargo test workspace**

```bash
cargo test --workspace 2>&1 | tail -15
```

Resultado esperado: todos los tests pasan.

- [ ] **Step 4: Verificar binario ag-lsp**

```bash
cargo build -p ag-lsp --release && ls -lh target/release/ag-lsp
```

- [ ] **Step 5: Smoke test del protocolo LSP**

```bash
printf 'Content-Length: 119\r\n\r\n{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"rootUri":null,"processId":null}}' \
  | timeout 3 ./target/release/ag-lsp 2>/dev/null | grep -o '"name":"ag-lsp"'
```

Resultado esperado: `"name":"ag-lsp"`

- [ ] **Step 6: tsc sin emitir**

```bash
cd tools/vscode-anti-gravital && npx tsc --noEmit && echo "TypeScript OK"
```

- [ ] **Step 7: Commit de formato si aplica**

```bash
cd ../..
git diff --quiet || git commit -am "style: cargo fmt post-implementacion fase-3 pendientes"
```

---

## Checklist de cierre de Fase 3

- [ ] `cargo test --workspace` pasa
- [ ] `cargo clippy --workspace -- -D warnings` pasa
- [ ] `cargo fmt --all -- --check` pasa
- [ ] CI fuzz-smoke pasa (60s por target)
- [ ] `ag-lsp` responde correctamente a `initialize`
- [ ] `tsc --noEmit` en el plugin sin errores
- [ ] `vsce package` genera `.vsix` sin errores
- [ ] `docs/fuzz/README.md` documenta gate manual de 24h
- [ ] `docs/dsl/lsp-roadmap.md` documenta capacidades futuras
- [ ] `README.md` refleja estado actual de Fase 3
- [ ] `docs/pr-drafts/fase-3.md` esta actualizado
