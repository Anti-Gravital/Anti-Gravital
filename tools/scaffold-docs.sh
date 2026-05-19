#!/usr/bin/env bash
# Genera los README de docs/modules/<crate>/, ademas de las vistas
# verbatim derivadas para docs/dsl/, docs/security/, docs/governance/
# y docs/benchmarks/. Idempotente.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ARCH="$ROOT/docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md"

write_module() {
  local crate="$1" criticidad="$2" capitulo="$3" fase="$4" dependencias="$5" resumen="$6"
  cat > "$ROOT/docs/modules/$crate/README.md" <<EOF
# $crate

> Capitulo de arquitectura: \`docs/architecture/$capitulo\`.
> README del crate: \`crates/$crate/README.md\`.
> Criticidad: $criticidad.
> Fase de implementacion: $fase.

## Dominio

$resumen

## Dependencias internas permitidas

$dependencias

## Reglas aplicables

Vease las reglas 14 y 15 de \`CLAUDE.md\` para la separacion entre
nucleo, estandar y opcional, y para la politica de dependencias entre
crates Anti-Gravital.

## Estado

Fase 0: el crate \`crates/$crate/\` esta declarado en el workspace con
\`src/lib.rs\` vacio y \`README.md\` propio. No contiene codigo
funcional. La implementacion comienza en la fase indicada arriba.
EOF
}

write_module "ag-core"      "Nucleo"   "06-nucleo-shield-y-core.md"        "Fase 1 (Shield) y Fase 2 (Core)" \
  "No depende de ningun crate Anti-Gravital. Es la base sobre la que todo lo demas se construye." \
  "Runtime HTTP de alto rendimiento. Shield (Tower middleware) mas Core (Axum router) en un mismo proceso. Extractores tipados, sistema de errores tipado, runtime Tokio multi-thread."

write_module "ag-dsl"       "Nucleo"   "07-anti-dsl.md"                    "Fase 3 a Fase 10" \
  "No depende de ningun crate Anti-Gravital." \
  "Lexer, parser, AST, analisis semantico y codegen del Anti-DSL. Implementacion incremental v0.1..v1.0."

write_module "ag-cli"       "Nucleo"   "05-ecosistema-modulos.md"          "Fase 2 a Fase 9" \
  "Depende de todos los crates a traves de Cargo features." \
  "Binario \`ag\`: orquesta creacion de proyectos, generacion DSL, hot reload, build, deploy, migracion y administracion de plugins."

write_module "ag-wasm-host" "Nucleo"   "09-plugins-wasi.md"                "Fase 9" \
  "Depende de ag-core." \
  "Host de plugins WASI sobre wasmtime. Sandbox de memoria, fuel y timeout. ABI estable via WIT."

write_module "ag-auth"      "Estandar" "08-modulos-batteries-included.md"  "Fase 4" \
  "Depende de ag-core. Puede depender de ag-data para persistencia de sesiones." \
  "Autenticacion y autorizacion: WebAuthn, JWT Ed25519, OAuth2, RBAC declarativo, rate limiting."

write_module "ag-data"      "Estandar" "08-modulos-batteries-included.md"  "Fase 2 a Fase 4" \
  "Depende de ag-core. No depende de ag-auth." \
  "Capa de datos con sqlx, migraciones embebidas y ORM tipado generado por el DSL."

write_module "ag-realtime"  "Estandar" "08-modulos-batteries-included.md"  "Fase 4" \
  "Depende de ag-core. Puede depender de ag-auth para suscripciones autenticadas." \
  "WebSocket binario, SSE fallback, NATS pub/sub. Presence, replay, broadcasting selectivo."

write_module "ag-cache"     "Estandar" "08-modulos-batteries-included.md"  "Fase 4" \
  "Depende de ag-core." \
  "Cache de dos niveles: moka en memoria y Redis con fred. Invalidacion por evento."

write_module "ag-storage"   "Estandar" "08-modulos-batteries-included.md"  "Fase 4" \
  "Depende de ag-core. Puede depender de ag-auth para URLs firmadas autenticadas." \
  "Adaptadores S3, MinIO, filesystem. URLs firmadas. Procesamiento basico de imagenes."

write_module "ag-observe"   "Estandar" "14-observabilidad-ag-observe.md"   "Fase 4" \
  "Depende de ag-core." \
  "Tracing estructurado, OpenTelemetry, Prometheus, tokio-console, dashboards Grafana."

write_module "ag-ui"        "Opcional" "05-ecosistema-modulos.md"          "Fase 4 o posterior" \
  "Depende de ag-core." \
  "SSR con askama, hidratacion selectiva, integracion HTMX. No compite con frameworks SPA."

write_module "ag-cloud"     "Opcional" "10-despliegue-ag-cloud.md"         "Fase 5" \
  "Depende de ag-cli." \
  "Despliegue simplificado: docker-compose, Fly.io, Railway, Kubernetes. Genera Dockerfile multi-stage."

write_module "ag-ai"        "Opcional" "11-ai-knowledge-graph.md"          "Fase 6" \
  "Depende de ag-dsl." \
  "Knowledge graph desde el AST del DSL. Documentacion arquitectonica. Sugerencias asistidas con proveedores configurables."

write_module "ag-mobile"    "Opcional" "13-mobile-ag-mobile.md"            "Fase 8" \
  "Depende de ag-dsl. Puede depender de ag-ai para generacion asistida." \
  "Generador Dart para Flutter: tipos freezed, cliente dio, WebSocket, SSE, mocks, widgets de auth nativa."

write_module "ag-migrate"   "Opcional" "12-migracion-ag-migrate.md"        "Fase 7" \
  "Depende de ag-dsl." \
  "Importadores: OpenAPI, Prisma, Django, FastAPI, Sequelize, GraphQL SDL. Genera schema.ag."

# Indice de modulos
cat > "$ROOT/docs/modules/README.md" <<'EOF'
# Modulos de Anti-Gravital - Indice

> Fuente verbatim: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`,
> seccion 5.

Esta carpeta describe los 15 crates del workspace. Cada subcarpeta
contiene un README con el dominio del crate, su criticidad, sus
dependencias internas permitidas, la fase en la que se implementa y
un puntero al capitulo correspondiente de la arquitectura tecnica.

## Mapa del ecosistema

| Crate | Criticidad | Capitulo | Fase de implementacion |
| --- | --- | --- | --- |
| `ag-core` | Nucleo | 6 | Fase 1 y 2 |
| `ag-dsl` | Nucleo | 7 | Fase 3 a 10 |
| `ag-cli` | Nucleo | 5 | Fase 2 a 9 |
| `ag-wasm-host` | Nucleo | 9 | Fase 9 |
| `ag-auth` | Estandar | 8 | Fase 4 |
| `ag-data` | Estandar | 8 | Fase 2 a 4 |
| `ag-realtime` | Estandar | 8 | Fase 4 |
| `ag-cache` | Estandar | 8 | Fase 4 |
| `ag-storage` | Estandar | 8 | Fase 4 |
| `ag-observe` | Estandar | 14 | Fase 4 |
| `ag-ui` | Opcional | 5 | Fase 4 o posterior |
| `ag-cloud` | Opcional | 10 | Fase 5 |
| `ag-ai` | Opcional | 11 | Fase 6 |
| `ag-mobile` | Opcional | 13 | Fase 8 |
| `ag-migrate` | Opcional | 12 | Fase 7 |

## Reglas

- Las cinco reglas de dependencia entre crates estan en el capitulo 5
  del maestro de arquitectura y en `docs/architecture/05-ecosistema-modulos.md`.
- La regla operativa fundamental: `ag-core` no depende de ningun otro
  crate Anti-Gravital.
EOF

# Vistas verbatim adicionales: dsl, security, governance, benchmarks.
extract_block() {
  # Extrae el bloque entre las lineas (inclusive) [start,end] del maestro
  # de arquitectura y lo escribe con un breadcrumb al inicio.
  local out="$1" titulo="$2" capitulo_fmt="$3" start="$4" end="$5"
  {
    printf '# %s\n\n' "$titulo"
    printf '> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion %s.\n\n' "$capitulo_fmt"
    sed -n "${start},${end}p" "$ARCH"
  } > "$out"
}

# Limites obtenidos con grep -n '^## ' del maestro de arquitectura.
extract_block "$ROOT/docs/security/README.md"   "Modelo de seguridad"                                  "15" 915 942
extract_block "$ROOT/docs/governance/README.md" "Modelo de gobernanza Open Source"                    "17" 981 1004
extract_block "$ROOT/docs/benchmarks/README.md" "Objetivos de rendimiento y metodologia de validacion" "16" 943 980

# DSL: subsecciones del capitulo 7.
mkdir -p "$ROOT/docs/dsl"
cat > "$ROOT/docs/dsl/README.md" <<'EOF'
# Anti-DSL - Indice

> Fuente verbatim: `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`,
> seccion 7. Capitulo navegable completo en
> `docs/architecture/07-anti-dsl.md`.

Esta carpeta contiene subsecciones extraidas del capitulo 7 de
arquitectura, dedicadas al Anti-DSL.

| Subseccion | Archivo |
| --- | --- |
| Filosofia del lenguaje | vease `docs/architecture/07-anti-dsl.md` |
| Implementacion incremental por versiones | `versionado.md` |
| Ejemplo completo de schema (v1.0 target) | `ejemplo-completo.md` |
| Anotaciones, tipos y diagnostics | vease `docs/architecture/07-anti-dsl.md` |

Para el contenido completo y verbatim, lea siempre el capitulo 7 del
maestro y su archivo derivado.
EOF

# Subsecciones del DSL (rangos identificados con grep '^### ' del cap 7).
# 7.2 Implementacion incremental por versiones: lineas 432-447.
# 7.3 Ejemplo completo de schema (v1.0 target): lineas 449-585 (hasta fin del cap 7).
{
  printf '# Anti-DSL - Implementacion incremental por versiones\n\n'
  printf '> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 7.2.\n\n'
  sed -n '432,447p' "$ARCH"
} > "$ROOT/docs/dsl/versionado.md"

{
  printf '# Anti-DSL - Ejemplo completo de schema (v1.0 target)\n\n'
  printf '> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 7.3 y siguientes.\n\n'
  sed -n '449,585p' "$ARCH"
} > "$ROOT/docs/dsl/ejemplo-completo.md"

echo "OK: docs/modules/, docs/dsl/, docs/security/, docs/governance/, docs/benchmarks/ generados."
