# DSL compilation pipeline

The Anti-DSL compiler stages and codegen targets (CLAUDE.md rule 20). The
`schema.ag` source flows through the `ag-dsl` crate; codegen emits the
artifacts that the rest of the ecosystem consumes.

```mermaid
graph TD
  schema[schema.ag] --> lexer[Lexer]
  lexer --> parser[Parser]
  parser --> ast[AST]
  ast --> sema[Semantic analysis]
  sema --> diag[Diagnostics]
  diag --> codegen[Codegen]
  codegen --> rust[Rust]
  codegen --> sql[SQL]
  codegen --> openapi[OpenAPI]
  codegen --> ts[TypeScript]
  codegen --> dart[Dart]
  codegen --> migrations[Migrations]
  codegen --> sdks[SDKs]
  codegen --> kg[Knowledge graph]
```

The diagnostics stage gates codegen: a schema with errors produces typed
diagnostics and no output. Codegen targets land incrementally across DSL
versions per the roadmap.
