#!/usr/bin/env bash
# Genera los archivos esqueleto de los 15 crates del workspace Anti-Gravital.
# Cada crate recibe: Cargo.toml, src/lib.rs (o src/main.rs para ag-cli) y README.md.
# Idempotente: sobrescribe.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

write_cargo() {
  local crate="$1" description="$2"
  cat > "$ROOT/crates/$crate/Cargo.toml" <<EOF
[package]
name = "$crate"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
homepage.workspace = true
documentation.workspace = true
readme = "README.md"
description = "$description"
keywords.workspace = true
categories.workspace = true
publish = false

[lints]
workspace = true
EOF
}

write_lib() {
  local crate="$1" doc="$2"
  cat > "$ROOT/crates/$crate/src/lib.rs" <<EOF
//! $doc
//!
//! Estado: Fase 0 (Fundaciones y Gobernanza). Este crate aun no contiene
//! codigo funcional. La implementacion se entrega en la fase declarada
//! en el README del crate y en la Hoja de Ruta del proyecto.
EOF
}

write_main() {
  local crate="$1" doc="$2"
  cat > "$ROOT/crates/$crate/src/main.rs" <<EOF
//! $doc
//!
//! Estado: Fase 0 (Fundaciones y Gobernanza). El binario \`ag\` aun no
//! contiene comandos operativos. La implementacion incremental comienza
//! en Fase 2 con \`ag new\`, \`ag dev\` y \`ag build\`.

fn main() {
    // Placeholder de Fase 0: deliberadamente vacio.
}
EOF
}

write_readme() {
  local crate="$1" criticidad="$2" dominio="$3" fase="$4" capitulo="$5"
  cat > "$ROOT/crates/$crate/README.md" <<EOF
# $crate

> Estado: Fase 0 - Vacio. La implementacion comienza en $fase.
> Criticidad: $criticidad.
> Capitulo de arquitectura: docs/architecture/$capitulo

## Dominio

$dominio

## Referencias

- Documento maestro de arquitectura: \`docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md\`.
- Capitulo navegable: \`docs/architecture/$capitulo\`.
- Hoja de ruta del crate: \`docs/modules/$crate/README.md\`.
- Constitucion tecnica: \`CLAUDE.md\`.

## Reglas aplicables

- Este crate cumple las reglas 14 y 15 de \`CLAUDE.md\` sobre crates y
  dependencias.
- Versionado semantico independiente del resto del workspace una vez
  publicado.
- Sin \`unsafe\` salvo justificacion via RFC bajo \`docs/rfc/\`.
EOF
}

# (crate criticidad dominio fase capitulo)
write_cargo "ag-core"      "Runtime HTTP, router y middleware del nucleo Anti-Gravital"
write_lib   "ag-core"      "Nucleo de Anti-Gravital: Shield, Core, runtime Tokio y extractores tipados."
write_readme "ag-core"     "Nucleo"    "Runtime HTTP de alto rendimiento. Combina la capa Shield (Tower middleware con TLS 1.3, autenticacion JWT Ed25519, rate limiting, validacion, CORS, CSRF, RBAC) y la capa Core (router Axum con extractores tipados, sistema de errores y estado compartido). Es la unica dependencia obligatoria del ecosistema."  "Fase 1 (Shield MVP) y Fase 2 (Core MVP)"  "06-nucleo-shield-y-core.md"

write_cargo "ag-dsl"       "Lexer, parser, analisis semantico y codegen del Anti-DSL"
write_lib   "ag-dsl"       "Compilador del Anti-DSL: lexer, parser, AST, analisis semantico y generadores de codigo."
write_readme "ag-dsl"      "Nucleo"    "Compilador del Anti-DSL (archivos .ag). Implementa lexer con logos, parser con chumsky, sistema de tipos, diagnostics legibles, y generadores de codigo a Rust, SQL, OpenAPI, TypeScript y Dart. Entregado por subversiones v0.1..v1.0."  "Fase 3 (alpha v0.1..v0.4) a Fase 10 (v1.0)"  "07-anti-dsl.md"

write_cargo "ag-cli"       "Binario unificado ag: new, generate, dev, build, deploy"
write_main  "ag-cli"       "Binario unificado \`ag\` del ecosistema Anti-Gravital."
write_readme "ag-cli"      "Nucleo"    "Binario \`ag\`. Orquesta todos los comandos del ecosistema: creacion de proyectos (\`ag new\`), generacion desde DSL (\`ag generate\`), servidor de desarrollo con hot reload (\`ag dev\`), build (\`ag build\`), despliegue (\`ag deploy\`), migraciones (\`ag migrate\`), administracion de plugins (\`ag plugin\`) y operaciones de IA (\`ag ai\`)."  "Fase 2 (new/dev/build), Fase 3 (generate), Fase 5 (deploy), Fase 6 (ai), Fase 7 (migrate), Fase 9 (plugin)"  "05-ecosistema-modulos.md"

write_cargo "ag-wasm-host" "Runtime de plugins WASI sobre wasmtime"
write_lib   "ag-wasm-host" "Host de plugins WASI: descubrimiento, validacion, sandbox y ciclo de vida."
write_readme "ag-wasm-host" "Nucleo"   "Sistema de plugins WASI sobre wasmtime. Define interfaces WIT, especifica \`plugin.toml\`, implementa el ciclo de vida de plugin (descubrimiento, validacion, carga, activacion, descarga) con sandbox de memoria, fuel y timeout. Soporta plugins escritos en Rust, Go (TinyGo) y AssemblyScript."  "Fase 9"  "09-plugins-wasi.md"

write_cargo "ag-auth"      "Autenticacion y autorizacion: WebAuthn, JWT Ed25519, OAuth2, RBAC"
write_lib   "ag-auth"      "Autenticacion y autorizacion del ecosistema Anti-Gravital."
write_readme "ag-auth"     "Estandar"  "Autenticacion y autorizacion: WebAuthn/Passkeys con FIDO2, JWT firmado con Ed25519, OAuth2 (Google, GitHub), API keys, refresh tokens con rotacion, rate limiting por usuario, RBAC declarativo con politicas evaluadas en el Shield."  "Fase 4"  "08-modulos-batteries-included.md"

write_cargo "ag-data"      "Capa de datos sqlx con verificacion compile-time y migraciones"
write_lib   "ag-data"      "Capa de datos con sqlx, ORM tipado generado por el DSL y migraciones embebidas."
write_readme "ag-data"     "Estandar"  "Capa de datos: pool de conexiones PostgreSQL con sqlx (queries verificadas en tiempo de compilacion), migraciones embebidas con sqlx::migrate!, ORM tipado generado por el DSL, transacciones declarativas, soporte para row-level security y multi-tenancy."  "Fase 2 (CRUD basico) a Fase 4 (avanzado)"  "08-modulos-batteries-included.md"

write_cargo "ag-realtime"  "WebSocket binario, SSE fallback y pub/sub con NATS"
write_lib   "ag-realtime"  "Tiempo real: WebSocket binario, SSE fallback y pub/sub con NATS."
write_readme "ag-realtime" "Estandar"  "Tiempo real: WebSocket binario, SSE como fallback, NATS embebido para casos pequenos y cliente NATS externo para produccion. Soporta pub/sub, suscripciones por canal, presence, broadcasting selectivo y reconexion con replay."  "Fase 4"  "08-modulos-batteries-included.md"

write_cargo "ag-cache"     "Cache L1 con moka y L2 con Redis"
write_lib   "ag-cache"     "Cache de dos niveles: moka en memoria como L1 y Redis como L2 con invalidacion por evento."
write_readme "ag-cache"    "Estandar"  "Cache de dos niveles: L1 con moka en memoria del proceso, L2 opcional con Redis via fred. Invalidacion por evento, estampillas de version, y soporte para patrones cache-aside y read-through declarados desde el DSL."  "Fase 4"  "08-modulos-batteries-included.md"

write_cargo "ag-storage"   "Almacenamiento de objetos: S3, MinIO, filesystem"
write_lib   "ag-storage"   "Almacenamiento de objetos con adaptadores S3, MinIO y filesystem local."
write_readme "ag-storage"  "Estandar"  "Almacenamiento de objetos con adaptadores S3, MinIO y filesystem local. URLs firmadas pre-signed, procesamiento basico de imagenes, deduplicacion por hash y politicas de retencion. API unica independiente del backend."  "Fase 4"  "08-modulos-batteries-included.md"

write_cargo "ag-observe"   "Observabilidad: tracing, OpenTelemetry, Prometheus, tokio-console"
write_lib   "ag-observe"   "Observabilidad nativa: tracing, OpenTelemetry exporter, metricas Prometheus y tokio-console."
write_readme "ag-observe"  "Estandar"  "Observabilidad nativa: tracing estructurado, exporter OpenTelemetry (OTLP), metricas Prometheus, integracion con tokio-console en modo dev y dashboards Grafana incluidos como JSON. Cero configuracion para casos comunes."  "Fase 4"  "14-observabilidad-ag-observe.md"

write_cargo "ag-ui"        "SSR con askama e integracion HTMX"
write_lib   "ag-ui"        "Renderizado del lado del servidor con askama e integracion progresiva HTMX."
write_readme "ag-ui"       "Opcional"  "Renderizado del lado del servidor con askama, hidratacion selectiva e integracion HTMX. No compite con frameworks SPA: cubre el rango donde un stack JS completo es excesivo. Para SPA o SSR ricas, el patron recomendado es Anti-Gravital como backend mas Next.js como frontend."  "Fase 4 o posterior"  "05-ecosistema-modulos.md"

write_cargo "ag-cloud"     "Despliegue simplificado al estilo Railway/Fly.io"
write_lib   "ag-cloud"     "Subsistema de despliegue simplificado: docker-compose, Fly.io, Railway y Kubernetes."
write_readme "ag-cloud"    "Opcional"  "Despliegue simplificado al estilo Railway/Fly.io. Soporta cuatro targets: docker-compose con Caddy como reverse proxy y TLS automatico, Fly.io via flyctl, Railway via su API, y Kubernetes con generacion de manifests estandar. Genera Dockerfiles multi-stage optimizados."  "Fase 5"  "10-despliegue-ag-cloud.md"

write_cargo "ag-ai"        "Knowledge graph y capacidades asistidas por IA"
write_lib   "ag-ai"        "Knowledge graph derivado del AST del DSL y capacidades asistidas por proveedores de IA."
write_readme "ag-ai"       "Opcional"  "Genera el knowledge graph desde el AST del DSL, produce documentacion arquitectonica y diagramas C4 en Mermaid, ofrece sugerencias de schema, revision de migraciones y analisis arquitectonico. Soporta proveedores Anthropic Claude, OpenAI, Ollama y vLLM. Funciona en modo offline con las features IA deshabilitadas."  "Fase 6"  "11-ai-knowledge-graph.md"

write_cargo "ag-mobile"    "Generador Dart para Flutter y clientes nativos"
write_lib   "ag-mobile"    "Puente de aplicaciones nativas: generador Dart con freezed, dio y clientes WebSocket/SSE."
write_readme "ag-mobile"   "Opcional"  "Puente de aplicaciones nativas. Genera SDK Dart completo: tipos con freezed, cliente HTTP con dio mas interceptores, cliente WebSocket, cliente SSE, mocks para tests. Widgets de autenticacion con WebAuthn nativo (Android Credential Manager, iOS Passkeys) y OAuth2. Publicado como paquete \`anti_gravital\` en pub.dev."  "Fase 8"  "13-mobile-ag-mobile.md"

write_cargo "ag-migrate"   "Importadores: OpenAPI, Prisma, Django, FastAPI, Sequelize, GraphQL SDL"
write_lib   "ag-migrate"   "Importadores de migracion desde frameworks legacy hacia esquemas Anti-DSL."
write_readme "ag-migrate"  "Opcional"  "Importadores de migracion desde frameworks legacy. Cubre OpenAPI 3.0 y 3.1, Prisma, Django, FastAPI/Pydantic, Sequelize y GraphQL SDL. Genera el archivo \`schema.ag\` correspondiente; la logica de negocio queda fuera de alcance y se documenta en cada guia."  "Fase 7"  "12-migracion-ag-migrate.md"

echo "OK: scaffolding de 15 crates generado."
