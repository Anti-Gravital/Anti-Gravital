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

### 8.8 `ag-mail` — Comunicación transaccional (estándar diferido)

Introducido por `ADR-0007` en la Fase 4.5. `ag-mail` es un módulo estándar
**diferido**: tiene la madurez y el alcance de un estándar, pero NO se instala
por defecto en los templates oficiales. Se incorpora cuando el proyecto
requiere correo transaccional outbound (verificación de cuentas, magic links,
recuperación de contraseña, alertas, notificaciones).

El alcance v1 es **exclusivamente outbound**. `ag-mail` NO es un MTA, NO
recibe correo (sin IMAP/POP), NO ofrece buzones persistentes, NO implementa
antispam, filtrado ni gestión de reputación de IP. Esta restricción es
deliberada y está fijada en el ADR.

> **Actualización de alcance (`ADR-0010`, 2026-06-03).** La restricción "NO es
> un MTA / inbound nunca" queda **superseded**: `ag-mail` se expande a un MTA
> outbound nativo (resolución MX, entrega ESMTP+STARTTLS, firma DKIM,
> clasificación de bounces) para enviar correo autenticado sin terceros en la
> ruta de envío. Es por fases y opt-in tras features de Cargo (`mta`, `api`,
> `queue-jetstream`); el baseline de relay outbound sigue siendo el modo por
> defecto. Buzones, IMAP/POP/JMAP e inbound general siguen fuera de alcance;
> inbound solo como parsing de DSN/ARF para bounces. Plan técnico: `RFC-0009`.
> Trabajo futuro, no implementado aún.

El stack técnico es `lettre` con transporte async Tokio y `rustls` para el
sender SMTP nativo. Los adapters de proveedor se declaran como features de
del relay SMTP nativo (apuntable a cualquier proveedor externo); la via sin terceros es el MTA nativo (`mta`). Cada sender implementa el trait
`MailSender`. El patrón Native | Adapter es idéntico al usado por
`ag-storage` (`Native | S3`) y `ag-cache` (`moka | Redis`).

Los **templates** se construyen con `askama` (ya utilizado por `ag-ui`) y se
validan en build-time contra el `schema.ag`: si el `from` declarado no
referencia un `domain` válido, si el archivo del template no existe o si
las variables del HTML no coinciden con las `vars` tipadas declaradas, el
compilador del DSL rechaza el build. Un correo mal formado deja de ser un
bug de runtime y se convierte en un error de compilación. Este es el
diferenciador real (correo correcto en build-time), no la entregabilidad.

La **cola asíncrona** acepta jobs con reintentos y backoff exponencial.
Backend por defecto en memoria; backend opcional persistente vía `ag-data`
(tabla de jobs) para sobrevivir reinicios. Cada job emite métricas hacia
`ag-observe`: `ag_mail_sent_total`, `ag_mail_failed_total`,
`ag_mail_retry_total`, histograma de latencia.

La **integración con `ag-auth`** es estrictamente unidireccional: `ag-auth`
consume `ag-mail` invocando un trait pequeño que `ag-auth` define. `ag-mail`
NO conoce a `ag-auth`. La sexta regla del capítulo 5 documenta esta
direccionalidad.

Detalle completo en `docs/modules/ag-mail/`.

### 8.9 `ag-domains` — Gestión de dominios y TLS (opcional infra)

Introducido por `ADR-0007` en la Fase 4.5. `ag-domains` es un módulo
**opcional de infraestructura**: no todo backend administra DNS, pero cuando
un proyecto quiere que `ag deploy` entregue una URL con certificado válido
en un comando, `ag-domains` es el módulo responsable.

El módulo **NO es un registrador de dominios**: el dominio se compra
externamente (Namecheap, Cloudflare Registrar, etc.) y se delega vía
nameservers al proveedor configurado. `ag-domains` tampoco reemplaza
Terraform ni Pulumi: para infraestructura compleja multi-cloud o gestión
centralizada de zonas DNS arbitrarias, el proyecto debe usar las
herramientas dominantes.

El núcleo del módulo es el trait `DnsProvider` (pequeño, versionado, con
**tests de contrato** que todo adapter debe pasar). Adapter inicial:
Cloudflare. Diseñado para añadir Route53, Namecheap, DigitalOcean, etc.

El **cliente ACME** (`instant-acme`) emite y renueva certificados de
Let's Encrypt. Soporta DNS-01 (preferido) y HTTP-01. Renovación automática
en background con vigilancia de expiración. Almacenamiento por defecto en
filesystem; opcionalmente `ag-storage`.

La cooperación con `ag-mail` se materializa en `generate_mail_records`:
`ag-mail` declara sus requisitos vía `MailSender::dns_requirements` y
`ag-domains` los materializa como registros (SPF, DKIM, DMARC). La
**verificación de propagación** usa `hickory-resolver` contra múltiples
resolvers públicos antes de marcar una operación como exitosa, bloqueando
`ag deploy` hasta que el dominio responde.

Detalle completo en `docs/modules/ag-domains/`.

### 8.10 `ag-workers` — Ejecución en segundo plano (estándar diferido)

`ag-workers` es el motor de ejecución en segundo plano del ecosistema. Su trabajo es
sacar el trabajo que no pertenece al ciclo de request fuera de los handlers HTTP —
jobs en background, reintentos, dead-letter, tareas programadas y worker pools —
preservando las propiedades del framework: ejecución nativa en Rust sin runtime
externo obligatorio, contratos schema-first, fronteras de crate acíclicas,
observabilidad nativa y simplicidad operacional (un solo binario por defecto).

La decisión arquitectónica central es **extraer un patrón ya probado, no inventar uno
nuevo**. `ag-mail` ya implementa cola con reintentos, backoff exponencial, backend
persistente sobre `ag-data` y ejecución de workers (`crates/ag-mail/src/queue/`).
`ag-workers` generaliza ese patrón a un crate compartido para que cada futuro
consumidor (entrega de webhooks, renovación ACME, post-procesado de subidas,
notificaciones, reportes) construya sobre un único substrato aburrido, durable y
observable, en lugar de reimplementar su propia cola.

El modelo mental es **el job**: una unidad tipada de trabajo diferido con identidad,
cola, payload versionado (`rmp-serde`) y política de reintentos. El runtime lo ejecuta
vía estrategias distintas (async sobre Tokio; CPU-bound sobre `spawn_blocking` acotado
por semáforo, nunca `rayon`) pero el contrato es uniforme. Hay dos backends de primera
clase: en memoria (por defecto, nativo) y PostgreSQL (durable, vía `ag-data`, feature
`postgres`, con `FOR UPDATE SKIP LOCKED` para leasing concurrente). El **enqueue
transaccional** (`enqueue_in_tx`) inserta el job dentro de la misma transacción que las
escrituras del llamador, dando la propiedad de transactional-outbox sin tabla de outbox
separada.

Dos propiedades son críticas. Primera: el perfil de release usa `panic = "abort"`, así
que el motor **no** promete aislamiento total de pánico; un job que entra en pánico
aborta el proceso, y el backend durable más la expiración de lease lo hacen recuperable.
Segunda, derivada de la anterior: un **circuit breaker de poison-job** incrementa el
contador de intentos en el momento del lease y enruta directamente al DLQ cualquier job
que supere `panic_guard_attempts`, convirtiendo un crash-loop infinito en una entrada
acotada y observable del DLQ. El scheduling de intervalos usa un claim singleton
(`FOR UPDATE SKIP LOCKED` sobre `ag_worker_schedules`) para disparar una sola vez bajo
escalado horizontal. Las etiquetas de métricas están acotadas; `tenant_id` nunca es una
etiqueta.

`ag-workers` se declara en el schema con el bloque `worker` (paralelo a `event`,
v0.6), que genera payloads tipados, stubs de `JobHandler`, el registro cerrado y, con la
feature `postgres`, migraciones SQL. Es **estándar diferido** (precedente `ag-mail`,
`ADR-0007`): producción-grade pero no instalado por defecto en los templates oficiales;
se incorpora cuando el `schema.ag` declara un `worker` o una feature lo habilita.

Decisión en `RFC-0012` y `ADR-0013`. Detalle completo en `docs/modules/ag-workers/`.


---

