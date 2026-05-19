# Capitulo 8. Modulos batteries-included

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 8
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [07-anti-dsl.md](./07-anti-dsl.md)
> Siguiente: [09-plugins-wasi.md](./09-plugins-wasi.md)

## 8. Módulos batteries-included

Esta sección especifica cada uno de los módulos estándar del ecosistema. Cada subsección documenta el propósito, el stack técnico, las decisiones de diseño y los puntos de extensión.

### 8.1 `ag-auth` — Autenticación y autorización

El módulo de autenticación implementa los esquemas modernos de identidad. La decisión arquitectónica central es soportar Passkeys/WebAuthn como primera clase, no como afterthought; las passwords son un mecanismo legacy soportado pero no recomendado.

El stack técnico es `webauthn-rs` para FIDO2, `jsonwebtoken` para JWT, `ring` para criptografía, `argon2` para hashing de passwords (cuando se usan), y `oauth2` como cliente OAuth2.

Los flujos soportados son: registro y autenticación con passkey, autenticación con email + password (legacy), OAuth2 con providers preconfigurados (Google, GitHub, Microsoft, Gravital ID), API keys para integraciones servidor-servidor, y refresh tokens con rotación.

Los JWT se firman con Ed25519 por defecto (curva Edwards25519, más rápida que RSA y más segura que ECDSA-P256 contra ataques de canal lateral). La clave privada vive en un secret manager externo (HashiCorp Vault, AWS Secrets Manager, GCP Secret Manager) o en variables de entorno con rotación documentada.

El RBAC se declara en el schema y se compila a expresiones evaluables. La política se evalúa una vez por request en la Shield, antes de llegar al handler. Las políticas pueden referenciar claims del JWT, parámetros de path, y consultar la base de datos si se declara explícitamente (con cache para evitar el N+1).

### 8.2 `ag-data` — Acceso a datos y migraciones

El módulo de datos se construye sobre sqlx, con verificación de queries SQL en tiempo de compilación. Esto significa que cuando se ejecuta `cargo build`, sqlx conecta a una base de datos de desarrollo (configurable por variable de entorno) y verifica que cada query sea sintácticamente válida y que los tipos de columnas devueltas coincidan con los structs Rust que las reciben. Un error de SQL deja de ser un error de runtime; se convierte en un error de compilación.

Los backends soportados son PostgreSQL (recomendado para producción), SQLite (para desarrollo, tests, y aplicaciones edge), y MySQL (para entornos heredados).

Las migraciones se embeben en el binario con `sqlx::migrate!`. Esto significa que el binario contiene en sí mismo el historial completo de migraciones, y al arrancar puede aplicar automáticamente las pendientes. Para entornos donde esto no es deseable (despliegues blue-green con migración como step separado), el comando `ag migrate apply` ejecuta las migraciones sin levantar el servidor.

Para arquitecturas multi-tenant, `ag-data` soporta nativamente schema-per-tenant en PostgreSQL: cada inquilino tiene su propio schema con las mismas tablas, y el router de conexión selecciona el schema en función del claim del JWT. También soporta Row-Level Security (RLS) para casos donde el aislamiento por schema es excesivo.

Las read replicas se configuran declarativamente; el módulo enruta queries de solo lectura al replica más cercano y queries de escritura al primario.

### 8.3 `ag-realtime` — Eventos y comunicación en tiempo real

`ag-realtime` ofrece tres modalidades de comunicación bidireccional: WebSocket binario, Server-Sent Events para flujos unidireccionales, y un bus de eventos pub/sub.

El bus de eventos usa NATS como broker. Para casos pequeños, NATS se ejecuta embebido en el mismo binario Anti-Gravital (modo edge). Para casos a escala, el binario se conecta a un clúster NATS externo. Esta dualidad permite arrancar simple y escalar sin reescribir.

Para WebSocket, el protocolo binario interno (basado en msgpack) reduce el overhead frente a JSON. Los handlers de WebSocket se declaran en el schema y reciben mensajes ya deserializados a structs Rust.

Para SSE, se usa como fallback automático en navegadores que no soportan WebSocket o están detrás de proxies que lo bloquean. La negociación es transparente.

La persistencia de eventos usa JetStream (componente de NATS) cuando está disponible, lo que permite replay de eventos para nuevos consumidores y durabilidad ante caídas del broker.

### 8.4 `ag-cache` — Caché multinivel

El módulo de caché ofrece dos niveles. El nivel L1 es caché en memoria con `moka`, una implementación concurrente sin locks contenciosos basada en TinyLFU. El nivel L2 es Redis (con `fred` como cliente), opcional, para caché distribuida entre instancias.

La invalidación se hace por eventos. Cuando un endpoint emite un evento (`user.updated`), `ag-cache` invalida automáticamente las entradas relacionadas en ambos niveles. La política de invalidación se declara en el schema.

El caché de queries SQL es automático: las queries marcadas con `@cache(ttl: 5m)` en el schema se cachean transparentemente, y la invalidación se dispara cuando un evento toca alguna de las tablas involucradas.

### 8.5 `ag-storage` — Almacenamiento de objetos

`ag-storage` ofrece una abstracción sobre tres backends: S3 (AWS y compatibles), MinIO (self-hosted), y filesystem local (para desarrollo). El backend se selecciona por configuración; el código de aplicación no se entera.

Las URLs firmadas para descarga y subida directa se generan con un solo call: `storage.signed_url(key, Duration::from_mins(15), Permission::Write)`.

El procesamiento de imágenes (resize, compress, format conversion) se hace con el crate `image`, soportando JPEG, PNG, WebP y AVIF. Los thumbnails se generan automáticamente en upload si se declara la política en el schema.

### 8.6 `ag-observe` — Trazabilidad, métricas y logging

La observabilidad es una preocupación de primer nivel y no un módulo opcional para producción. Su stack es `tracing` para spans estructurados, `opentelemetry-rust` para exportación a backends compatibles (Jaeger, Tempo, Datadog, Honeycomb), `metrics` para métricas con backend Prometheus, y dashboards Grafana pre-configurados que se incluyen como JSON en el repositorio.

Cada request atraviesa todo el sistema con un correlation ID único que aparece en todos los logs estructurados, todos los spans de tracing, y todos los errores devueltos al cliente. Esto resuelve el problema del debugging en producción: dado un ticket de soporte con un correlation ID, el operador puede reconstruir el camino completo del request.

`tokio-console` se integra en modo desarrollo para inspección en vivo de las tareas Tokio.

### 8.7 `ag-ui` — Server-Side Rendering opcional

El módulo SSR existe para casos donde un frontend SPA es excesivo: dashboards internos, páginas de marketing, formularios simples, e interfaces administrativas. Está basado en `askama` (templating compilado en build time, con tipos verificados) e integración nativa con HTMX para interactividad sin frameworks JavaScript pesados.

Este módulo es explícitamente *no* un competidor de React, Vue, Svelte o Next.js. Para aplicaciones SPA o SSR ricas, el patrón recomendado es Anti-Gravital como backend con un frontend Next.js (u otro) que consume el cliente TypeScript generado.


---

