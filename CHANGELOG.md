# Changelog

Formato basado en Keep a Changelog (https://keepachangelog.com/) y
semver (https://semver.org/). El repositorio aun no publica ninguna
version; las entradas viven bajo `[Unreleased]` hasta que se libere la
primera version etiquetada.

## [Unreleased]

### Gobernanza: deuda tecnica en GitHub Issues (CLAUDE.md regla 29)

Cambiado:

- CLAUDE.md regla 29: la deuda tecnica, bugs, fallos y documentacion de
  problemas se registran como GitHub Issues con etiqueta `tech-debt`, no como
  archivos del repositorio. `docs/DEBT.md` queda congelado (historico); se
  prohiben nuevos archivos tipo `BACKLOG.md`/`TODO.md`/`*-DEBT.md`.
- Eliminado `docs/ag-domains/BACKLOG.md`; su contenido vive en el issue #76.

### ag-domains: endpoint verify + capacidades de proveedor + comando diagnose

Anadido:

- `crates/ag-domains/src/api.rs`: `POST /v1/domains/attachments/{id}/verify` con
  `OwnershipVerifier` inyectable (`NullVerifier` por defecto;
  `PropagationOwnershipVerifier` tras feature `propagation`). Emite
  `domain.ownership.verified` y registra fallos de verificacion. Cierra el
  hueco de que el evento estuviera modelado pero nunca emitido.

- `crates/ag-domains/src/provider/capabilities.rs`: registro de capacidades por
  proveedor (`ProviderCapabilities`, `known_provider_capabilities`,
  `capabilities_for`) — eje adaptador (manual/read_apply) vs features del
  proveedor (apex alias, CNAME flattening, DNS-01 auto). Lista conservadora.
- `crates/ag-domains/src/api.rs`: endpoint `GET /v1/domains/provider-capabilities`
  (feature `api`) + test de integracion.
- `crates/ag-cli`: comando `ag domains diagnose` — compara registros esperados vs
  observados en resolvers publicos y reporta hallazgos accionables; usa
  `propagation::lookup_observed` (nuevo, lookup A/AAAA/CNAME/TXT) + `diagnostics`.
- Docs: `reference/provider-capability-matrix.md`, `how-to/connect-providers.md`,
  `how-to/domain-connect.md`, `how-to/configure-wildcard.md`,
  `how-to/troubleshoot.md`; `reference/cli.md` documenta `diagnose`; OpenAPI con
  `/provider-capabilities`.

### ag-domains: orquestacion de emision de certificados (RFC-0011, fase 2)

Anadido:

- `crates/ag-domains/src/issuance.rs`: proteccion de rate-limit del blueprint
  §13.4. `CertIssuer` trait (el emisor ACME real se inyecta), `IssuedCertificate`,
  `san_key` (clave de deduplicacion canonica por conjunto SAN), `IssuanceLimiter`
  (dedup de ordenes en vuelo + contadores por dominio registrable) y
  `issue_certificate` (reserva, emite, cuenta exitos, libera). Logica nativa sin
  dependencias externas; tests con issuer mock.
- `crates/ag-edge/tests/issuance_to_edge.rs`: test E2E que emite via la
  orquestacion (issuer rcgen), carga el PEM en el `CertStore` del edge y lo
  selecciona por SNI; verifica la cadena emision -> serving completa.

### ag-domains: event log de dominio + metricas de control-plane

Anadido:

- `crates/ag-domains/src/events.rs`: `DomainEvent` (vocabulario blueprint
  §16.2: `domain.attachment.created`, `domain.ownership.verified`,
  `domain.detached`) y trait `EventSink` con `NullEventSink` (defecto),
  `InMemoryEventSink` (nativo/tests) y `TracingEventSink`. Sin broker externo
  (ADR-0009). La API REST emite eventos en create/detach (`ApiState::with_events`).
- `crates/ag-domains/src/metrics.rs`: contadores `ag_domains_attachments_total`,
  `ag_domains_detached_total`, `ag_domains_verification_failures_total`.
- Tests: unitarios del event log + test de integracion que captura los eventos
  emitidos por la API real.

### ag-domains: store SQL Postgres (RFC-0011, fase D)

Anadido:

- `crates/ag-domains` feature `sql-store`: `SqlAttachmentStore` (Postgres via
  sqlx) que implementa `AttachmentStore`. Almacena el attachment como JSONB con
  columnas indexadas (id, hostname, lifecycle) y tabla de tombstones; esquema
  embebido + `migrations/0001_ag_domains_attachments.sql`. Mapea violacion de
  unicidad a `AlreadyExists` y respeta tombstones en `create`. El store nativo
  (memoria/JSON) sigue siendo el predeterminado (ADR-0009). Tests de integracion
  `#[ignore]` que requieren `DATABASE_URL`.

Cambios internos (no rompen la API publica del crate):

- El trait `AttachmentStore` pasa a asincrono y `&self` (mutabilidad interior),
  para poder compartir un store como `Arc<dyn AttachmentStore>` entre tareas y
  permitir backends async. `InMemoryStore`/`JsonFileStore` adaptados; la API REST
  ya no envuelve el store en un `Mutex` externo; la CLI ejecuta los comandos de
  dominios sobre el runtime tokio.

### ag-domains/ag-edge: edge en vivo + API REST (RFC-0011, fases B y C)

Anadido:

- `crates/ag-edge` feature `server`: listener HTTP ejecutable (`serve_http`,
  axum) que sirve el challenge ACME HTTP-01 (`Http01ChallengeStore` +
  `acme_challenge_token`, con proteccion ante path traversal), aplica politica
  canonica/redireccion y enruta por `Host`/`:authority` con fail-closed para
  hosts desconocidos. Tests de integracion sobre TCP real.
- `crates/ag-edge` feature `tls`: almacen `CertStore` de certificados rustls
  (exacto + wildcard) con puente PEM desde la salida de emision ACME
  (`insert_pem`), `SniCertResolver` (RFC 6066) y `serve_https`. Test de
  integracion con handshake TLS real verificando seleccion por SNI.
- `crates/ag-domains` feature `api`: API REST `/v1/domains/attachments`
  (crear/listar/obtener/instructions/status/detach) respaldada por cualquier
  `AttachmentStore`, con mapeo de errores a codigos HTTP. Tests de integracion
  sobre HTTP real (`tests/api_rest.rs`).
- `openapi/ag-domains.v1.yaml` actualizado al contrato implementado.

Cambios de workspace:

- `hyper-util`: features `server-auto` y `service` anadidas (para el listener
  HTTPS del edge).

### ag-domains plano de control + ag-edge (ADR-0012 / RFC-0011, fase A)

Anadido:

- `crates/ag-domains`: capa de plano de control aditiva. Modulos `hostname`
  (normalizacion/clasificacion IDN via `idna`), `attachment` (maquina de estados
  con dimensiones ownership/dns/tls/routing), `store` (`AttachmentStore` trait +
  `InMemoryStore` + `JsonFileStore` nativos), `ownership` (token TXT + verificacion
  via propagacion), `instructions` (motor de registros DNS + exportacion zona BIND),
  `caa` (preflight CAA antes de ACME) y `diagnostics` (esperado vs observado). La
  libreria declarativa DNS+TLS previa queda intacta. Dependencias nuevas `idna`,
  `uuid`. 80 tests.

- `crates/ag-edge` (nuevo crate, plano de datos): `router::resolve_hostname`
  (precedencia exacto/wildcard/legacy con fail-closed para hostnames
  desconocidos), `tls::SniCertStore` + `allow_on_demand` (emision on-demand
  restringida), `redirect::CanonicalPolicy` (www<->apex preservando path/query).
  Libreria pura y probada; sin listeners en vivo todavia. 17 tests.

- `crates/ag-cli`: subcomandos `ag domains attach`, `instructions`, `export-zone`,
  `status`, `list`, `verify`, `detach`. Flujo manual sin credenciales de proveedor.

- Gobernanza: `docs/rfc/RFC-0011-ag-domains-control-plane.md`,
  `docs/adr/0012-ag-domains-control-plane.md`. Documentacion en
  `docs/ag-domains/` y esqueleto OpenAPI en `openapi/ag-domains.v1.yaml`.

Estado real tras las fases A-C: los listeners HTTP/HTTPS, el responder HTTP-01
y la API REST ya estan implementados. Permanecen diferidos el almacen SQL
(Postgres, fase D), Domain Connect y adaptadores de proveedor adicionales
(fase E; Cloudflare y exportacion BIND ya existen), wildcards DNS-01 y el modulo
`ag-registrars` (fase F).


### ag-mail - Motor de plantillas externo (minijinja) (2026-06-04)

Aditivo, tras la feature `minijinja` (apagada por defecto). Cierra DEBT-003.

- `template::jinja::MinijinjaTemplate`: implementa el trait `MailTemplate` sobre
  el motor `minijinja` (loops, condicionales, filtros), alternativa drop-in al
  `StringTemplate` por defecto. Dep `minijinja` tras la feature.

### Fase 4.6-C - MTA: webhooks firmados (2026-06-04)

Aditivo, tras la nueva feature `api` (apagada por defecto). Avanza DEBT-021.

- `api::webhook`: firma y verificacion de webhooks HMAC-SHA256 sobre
  `{id}.{timestamp}.{payload}` con secretos `whsec_`, cabecera `v1,<base64>`,
  soporte de multiples firmas (rotacion) y ventana de tolerancia anti-replay;
  verificacion en tiempo constante. Deps `hmac`/`sha2`/`base64` tras `api`.
- `.github/workflows/ci.yml`: el job `mail-mta` ahora ejercita `mta,api`.

### Fase 4.6-B - MTA: colas, shaping, egress y suppression (2026-06-04)

Aditivo, tras la feature `mta` (apagada por defecto). Cierra DEBT-019 (núcleo) y
avanza DEBT-020. Persistencia durable del spool queda como DEBT-023.

- `sender::mta::egress`: `EgressSource`/`EgressPool` con weighted round-robin
  suave (SWRR) para IP warming; `MtaSender` elige una fuente por mensaje y
  aplica EHLO + IP de origen al conectar.
- `sender::mta::shaping`: `ShapingLimits`/`ShapingConfig` por `site_name`
  (rate token-bucket determinista + cap de conexiones por semáforo).
- `sender::mta::queue`: cola de dos niveles en memoria (scheduled min-heap +
  ready batch `max_ready`) con `MtaRetryPolicy` (backoff exponencial, max-age),
  trait `DeliveryBackend` (lo implementa `MtaSender`), worker `run`, y reportes
  por ciclo. `MtaSender::build_jobs` genera jobs por dominio desde un `Email`.
- `sender::mta::suppress`: `SuppressionList` automática (hard bounce 5xx,
  límite de reintentos), case-insensitive; la cola la consulta y la alimenta.
- `sender::mta::dsn`: intake asíncrono de DSN (RFC 3464) y ARF (RFC 5965) vía
  `mail-parser`; `process_dsn`/`process_arf` alimentan la suppression list
  (cierra DEBT-020).
- `MtaSender` clasifica los fallos SMTP: solo un 5xx del destino es permanente
  (suprime); fallos propios (DKIM, MX, TLS) son transitorios (reintentables).
- Métricas de profundidad de cola; tests deterministas (reloj explícito).

### Fase 4.6-A - Endurecimiento: RSA DKIM, metricas y CI del MTA (2026-06-04)

Aditivo, cierra deuda de la 4.6-A (DEBT-017/018).

- `sender::mta::dkim`: firma DKIM ahora soporta Ed25519 **y RSA-SHA256**
  (PKCS#8 DER y PEM) via `mail_auth ... RsaKey::from_key_der` (no deprecado);
  tests positivos para ambos algoritmos. Nueva dep `rustls-pki-types` tras `mta`.
- `MtaSender::send`: emite metricas `ag_mail_sent_total` /
  `ag_mail_send_latency_seconds` y `ag_mail_retry_total` en failover de MX.
- `.github/workflows/ci.yml`: nuevo job `mail-mta` (build + test + clippy
  `--features mta`), antes el MTA no se compilaba en CI (RFC-0009 §4.8).
- `docs/DEBT.md`: renumeradas las entradas de MTA (colision de DEBT-012/013/014);
  DEBT-017/018 cerradas; 4.6-B/C/D y el test de entrega en vivo quedan como
  DEBT-019..022.

### Fase 4.6-A - Implementacion: motor MTA outbound nativo de ag-mail (opt-in)

Aditivo. Nueva feature de Cargo `mta` en `ag-mail`, apagada por defecto; el
sender por defecto y los adapters no cambian (ADR-0010 / RFC-0009).

Anadido:

- `crates/ag-mail` feature `mta`: `sender::mta::MtaSender` (implementa
  `MailSender`) con entrega directa al MX destino via ESMTP+STARTTLS
  (`mail-send`), agrupacion de destinatarios por dominio y construccion MIME
  RFC 5322 (`mail-builder`).
- `sender::mta::resolve`: resolucion MX (`hickory-resolver`) con orden por
  preferencia, rollup `site_name` y fallback de MX implicito (RFC 5321).
- `sender::mta::dkim`: firma DKIM Ed25519 (RFC 8463) aplicada al final;
  el material de clave lo aporta el llamador / `ag-domains`.
- `sender::mta::bounce`: clasificador de bounces SMTP/RFC 3463 (transitorio
  vs permanente), puro y unit-testeado.
- Nuevas variantes de error `AgMailError::{Dns, NoMailHost, Dkim}`.
- Dependencias opcionales tras `mta`: `mail-send 0.6` (features `builder`,
  `ring`, `tls12`), `mail-auth 0.9` (`ring`), `hickory-resolver 0.26`.

Notas: la ruta de entrega en vivo se cubre con tests `#[ignore]` (requiere
red/puerto 25). Deuda registrada en `docs/DEBT.md` (DEBT-012 RSA DKIM,
DEBT-013 metricas+CI, DEBT-014 colas/shaping/DSN-FBL).

### Gobernanza - Pivot ag-mail a MTA outbound nativo (2026-06-03)

Solo documentacion y gobernanza; sin codigo funcional nuevo.

- `docs/adr/0010-ag-mail-native-mta-pivot.md`: nuevo ADR que supersede el
  alcance v1 "NO es un MTA / inbound nunca" de `ADR-0007` y expande `ag-mail`
  a un MTA outbound nativo (resolucion MX, ESMTP+STARTTLS, firma DKIM,
  clasificacion de bounces), por fases y opt-in tras features de Cargo,
  conservando el patron Native | Adapter y la direccionalidad `ag-auth ->
  ag-mail`. `ADR-0007` sigue vigente para `ag-domains` y la clasificacion.
  Principio aditivo-only vinculante: el MTA se anade como `MtaSender` opt-in
  sin degradar el baseline (default `SmtpSender`, adapters de proveedor
  existentes y colas permanecen sin cambios). Se rechaza el "overwrite / MTA
  por defecto / degradar adapters / migrar la cola" del blueprint. Las
  plataformas externas citadas en la investigacion son solo referencia, no
  dependencias ni objetivos de integracion del proyecto.
- `docs/rfc/RFC-0009-ag-mail-native-mta.md`: plan tecnico (6 subsistemas,
  dependencias `mail-send`/`mail-builder`/`mail-auth`/`mail-parser`/
  `hickory-resolver` tras feature `mta`, cola de dos niveles, modelo de datos,
  superficie REST, webhooks firmados HMAC-SHA256, plan por fases 4.6-A..D + Fase 5+).
  Supersede a `RFC-0006` para el alcance de `ag-mail`.
- Alineacion documental: `docs/modules/ag-mail/README.md` reescrito (baseline
  implementado real + direccion MTA, fin del estado "Pendiente"); notas de
  actualizacion de alcance en Arquitectura `§8.8` (maestro EN+ES y derivado) y
  nota futura `§4.5.5` en Hoja-de-Ruta (maestro EN+ES y derivado de fase);
  README raiz (EN+ES); indices de ADR y RFC; `RFC-0006`/`ADR-0007` marcados
  superseded para el alcance de `ag-mail`.
- `docs/master/VERSION.md` y `.github/workflows/docs.yml`: hashes SHA-256 de
  los dos maestros recalculados.

### Fase 4.5 - Implementacion tecnica: ag-mail + ag-domains (2026-05-24)

Anadido:

- `crates/ag-mail` (estandar diferido): `MailSender` trait + `SmtpSender` (lettre + rustls).
  `InMemoryQueue` con reintentos y backoff exponencial.
  `StringTemplate` + `MailTemplate` trait. `NullSender` (feature `test-utils`). 38 tests.

- `crates/ag-domains` (opcional infra): `DnsProvider` trait + `CloudflareProvider` (reqwest).
  `apply_mail_records`: upsert idempotente de SPF/DKIM/DMARC. `PropagationChecker`
  con hickory-resolver. Soporte ACME completo via instant-acme + rcgen (DNS-01,
  renovacion automatica). 28 tests.

- `crates/ag-dsl` v0.7: tokens `mail`, `domain`, `template` en el lexer; nodos
  `MailBlock`, `DomainBlock`, `MailTemplateDef` en el AST; parsers y analisis
  semantico para los nuevos bloques. 151 tests.

- `crates/ag-auth` (feature `mail`): `AuthMailer` con `send_verification`,
  `send_password_reset`, `send_magic_link`. `AgAuth::with_mail()` builder. 37 tests.

- `crates/ag-cli`: subcomandos `ag domains check`, `ag domains sync`, `ag mail test`.

- `examples/auth-mail-demo`: ejemplo ejecutable de los tres flujos con `NullSender`.

- `tests/integration/tests/fase45_e2e.rs`: 7 tests E2E cross-module (total: 14).

### Fase 4.5 - Pendientes: ag-lsp v0.7 + DSL from->domain + ACME fix + docs (2026-05-24)

Anadido:

- `crates/ag-lsp/src/backend.rs`: hover y completions para los bloques DSL v0.7.
  `mail`, `domain`, `template` con ejemplos de codigo; propiedades `provider`,
  `from`, `subject`, `vars`, `dkim_selector`, `dmarc_policy`, `dmarc_rua`. 6 tests nuevos (15 total).

- `docs/manual/03-dominio-tls-correo.md`: guia completa "Configurar dominio, TLS
  y correo transaccional con Anti-Gravital". Seis secciones: declaracion DSL,
  DNS sync, verificacion de propagacion, ACME, correo transaccional, ag-auth+ag-mail.

Corregido:

- `crates/ag-dsl/src/semantic.rs`: warning cuando el hostname del campo `from` de
  un bloque `mail` no coincide con ningun `domain_name` declarado en los bloques
  `domain` del schema. 2 tests nuevos (153 total en ag-dsl).

- `crates/ag-domains/src/acme/renewal.rs`: bug de underflow en `spawn_renewal_task`.
  `check_interval - renew_threshold` panickea en debug mode si `renew_before_days >= 1`.
  Nuevo calculo: `sleep_secs = renew_before_days.max(1) * 86400`. 4 tests nuevos.

- `tools/vscode-anti-gravital/syntaxes/anti-gravital.tmLanguage.json`: keywords DSL v0.7
  anadidos al resaltado de sintaxis: `mail`, `domain`, `template`, `provider`, `from`,
  `subject`, `vars`, `dkim_selector`, `dmarc_policy`, `dmarc_rua`.

Cambiado:

- `docs/roadmap/STATUS.md`: Fase 4.5 marcada como completada con todos los criterios
  de salida marcados. Commits de referencia: PR #42 (implementacion) y PR #43 (pendientes).

- `README.md`: banner y tabla de calendario actualizados a "Fase 4.5 implementacion
  tecnica completa".

### Fase 4.5 - Integracion documental: ag-mail + ag-domains (2026-05-23)

Anadido:

- `docs/adr/0007-ag-mail-ag-domains.md`: ADR-0007 oficializa la Fase 4.5
  como fase aditiva entre la Fase 4 (completa) y la Fase 5 (pendiente).
  Introduce `ag-mail` (estandar diferido, outbound + adapters) y
  `ag-domains` (opcional infra, DNS + ACME + SPF/DKIM/DMARC). Alcance,
  restricciones, direccionalidad de dependencias (ag-auth -> ag-mail,
  ag-cloud -> ag-domains), consecuencias documentadas. Estado: Aprobado.

Cambiado:

- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`: insertada fila 4.5 en
  el resumen de fases entre Fase 4 y Fase 5; duracion total
  24-28 meses -> 25-30 meses; nueva seccion "Fase 4.5" completa con
  criterios de entrada, entregables, criterios de salida bloqueantes y
  riesgos. Hito v0.5 BETA permanece al final de la Fase 5.

- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`:
  - § 5.1: tabla del ecosistema actualizada a 17 crates con ag-mail
    (Estandar diferido) y ag-domains (Opcional infra). Parrafo de cierre
    explica la clasificacion "estandar diferido".
  - § 5.2: diagrama del ecosistema redibujado para incluir la columna
    "Estandar diferido" con ag-mail y la flecha ag-cloud -> ag-domains
    dentro de "Modulos opcionales".
  - § 5.3: sexta regla (ag-auth -> ag-mail, no ciclo) y septima regla
    (ag-cloud -> ag-domains, dependencia no rigida) anadidas.
  - § 5.4: estructura del monorepo incluye `crates/ag-mail/` y
    `crates/ag-domains/` con anotaciones de Fase 4.5.
  - § 7.2: tabla del DSL realineada. v0.5/v0.6 marcados "Fin Fase 4
    (entregado)"; v0.7 "Mail y dominios declarativos" (Fin Fase 4.5);
    v0.8 "Plugin hooks" (Fin Fase 9). Multi-tenancy y migracion de
    datos quedan diferidos para RFCs propios.
  - § 8.8 (nueva): especificacion completa de `ag-mail` (estandar
    diferido). Patron Native | Adapter, validacion build-time de
    templates, cola asincrona, integracion ag-auth.
  - § 8.9 (nueva): especificacion completa de `ag-domains` (opcional
    infra). Trait DnsProvider, ACME (instant-acme), cooperacion con
    ag-mail para SPF/DKIM/DMARC, verificacion de propagacion.
  - § 10: renombrado a "Subsistema de despliegue (ag-cloud + ag-domains)".
  - § 10.6 (nueva): integracion ag-cloud <- ag-domains. Flujo de seis
    pasos del deploy con dominio: validar control, configurar DNS,
    emitir/renovar TLS, asociar al target, materializar SPF/DKIM/DMARC,
    verificar propagacion.

- `docs/master/VERSION.md`: nuevos hashes SHA-256 de los dos maestros
  markdown. Blueprint PDF v4.0 registrado como deuda explicita pendiente
  de re-export a v4.1 (los maestros markdown gobiernan segun politica
  pre-existente).

Pendiente (deuda registrada):

- Re-export del Blueprint a `ANTI-GRAVITAL-Blueprint-v4.1.pdf` con los
  cambios de la Fase 4.5. Requiere herramienta de exportacion fuera del
  scope de esta rama documental.

### Fase 4 - Modulos estandar: implementacion tecnica completa (2026-05-23)

Anadido:

- `crates/ag-auth`: implementacion completa de autenticacion y autorizacion.
  JWT Ed25519 con firma/verificacion PEM, API keys con hash BLAKE3 y prefijo
  configurable, WebAuthn/Passkeys con CBOR (ciborium) y verificacion COSE
  (ES256 via p256, EdDSA via ed25519-dalek), OAuthClient para Google y GitHub
  con flujo PKCE, RefreshBlacklist en memoria con RwLock. 32 tests pasan.

- `crates/ag-realtime`: bus de eventos InProcess mas cliente NATS externo.
  EventBus broadcast en proceso, NatsExternalClient con async-nats 0.48,
  TLS en 3 niveles (CAs del sistema, CA custom, mTLS), JetStream con stream
  AG_EVENTS y publish con ACK, ws_handler Axum que conecta WebSocket al bus,
  sse_handler Axum que sirve stream SSE EventSource-compatible via BroadcastStream.
  AgRealtime::new es ahora async y retorna Result.

- `crates/ag-storage`: URLs firmadas HMAC-SHA256 y backend S3/MinIO.
  sign_url y verify_signed_url con comparacion en tiempo constante, token
  en formato {base64url_hmac}_{expires_at}, AgStore convertido de struct a
  enum (Native | S3), S3Store via object_store 0.11 con soporte MinIO
  via endpoint opcional.

- `crates/ag-cache`: cobertura de tests elevada a mas de 80% en todos los
  archivos. Tests adicionales para L1Cache incluyendo invalidacion por tags,
  expiracion, y operaciones concurrentes.

- `crates/ag-observe`: cobertura de tests elevada a mas de 80%. Tests para
  init del layer, metricas handler, y configuracion desde env vars.

- `tests/integration`: nuevo crate de tests E2E cross-module con 7 tests:
  6 tests unitarios por modulo (observe, auth JWT, auth API key, cache,
  realtime, storage) y 1 test E2E de 15 pasos que atraviesa ag-auth,
  ag-cache, ag-realtime, ag-storage y ag-observe en secuencia.

- `docs/rfc/RFC-0005-ag-cache-native-l2.md`: RFC para L2 cache nativo
  Anti-Gravital compatible con protocolo RESP2, sin dependencia de Redis
  como servicio externo. Estado: propuesto, pendiente de aprobacion.

Cambiado:

- `examples/realtime-chat`: actualizado para AgRealtime::new async y
  bus() que retorna Option<Arc<EventBus>>.

- READMEs de modulos actualizados de placeholder "Fase 0 - Vacio" a
  documentacion de uso real: ag-auth, ag-realtime, ag-cache, ag-observe,
  ag-storage.

- `docs/roadmap/STATUS.md`: Fase 4 marcada como completada con todos los
  entregables en [x].

### Benchmarks reales Fase 2 y correccion de routing (2026-05-21)

Corregido:

- `examples/todo-api/src/main.rs`: ruta parametrizada corregida de
  `"/todos/{id}"` (sintaxis Axum 0.8) a `"/todos/:id"` (sintaxis Axum 0.7,
  version en uso). El bug causaba que GET, PUT y DELETE /todos/:id devolvieran
  404 sin tocar la base de datos. Todas las mediciones previas de throughput
  para esas rutas eran invalidas (median velocidad de respuestas 404, no
  operaciones DB reales).

Documentado:

- `docs/benchmarks/measurement-2026-05-21-fase-2-crud-ryzen5-2500u.md`:
  primera medicion valida de CRUD con PostgreSQL real. Hardware: AMD Ryzen 5
  2500U, 4C/8T, PostgreSQL 18.4 nativo. Resultados: GET /todos/:id = 14 478
  req/s mediana (c=100); POST /todos = 8 934 req/s (c=50, synchronous_commit
  off); stack HTTP sin DB = 88 930 req/s. Los criterios de 40K req/s y p99
  <= 5 ms no se alcanzan en este hardware con configuracion estandar de
  PostgreSQL; el cuello de botella es el scheduler de OS con el modelo
  proceso-por-conexion de PG sobre 4 nucleos fisicos.

- `docs/roadmap/STATUS.md`: criterios 2.3 de benchmark y latencia actualizados
  a `[ ]` con numeros reales y nota explicativa del bug de routing invalidado.

### Verificacion y cierre tecnico de Fase 2 (2026-05-21)

Corregido:

- `examples/todo-api/Dockerfile`: base image actualizada de `rust:1.79-slim` a
  `rust:1.95-slim` para coincidir con `rust-toolchain.toml`. `rust-toolchain.toml`
  ahora se copia antes de `rustup target add` para que rustup instale el target
  MUSL contra el canal correcto (1.95.0). Build verificado: binario MUSL estatico
  5.3 MB, imagen `FROM scratch` 2.49 MB, `/health` 200 OK con PostgreSQL.

- `templates/rest/src/main.rs.tmpl`, `templates/realtime/src/main.rs.tmpl`,
  `templates/fullstack/src/main.rs.tmpl`: eliminado import no usado `ShieldConfig`.
  Los tres templates compilan sin warnings con la nueva version del binario `ag`.

- `deny.toml`: restaurado ignore de RUSTSEC-2023-0071 (`rsa` dep transitiva de
  `jsonwebtoken`). El advisory fue reactivado al actualizar jsonwebtoken a v10.
  Anti-Gravital usa exclusivamente EdDSA; el timing attack de RSA no aplica.

Anadido:

- `examples/todo-api/src/main.rs`: soporte de variable de entorno
  `DATABASE_MAX_CONNECTIONS` para ajustar el pool sin recompilar (por defecto 10).
  Con pool=50 el throughput de GET /todos/:id llega a 82K req/s en hardware
  informativo (Ryzen 5 2500U, Docker PostgreSQL).

Seguridad:

- `jsonwebtoken` actualizado de 9.3.1 a 10.4.0. Cierra CVE-2026-25537 /
  GHSA-h395-gr6q-cpjc (Type Confusion en validacion de claims nbf/exp que permite
  bypass de restricciones temporales). Feature `rust_crypto` anadida para activar
  el backend criptografico puro Rust requerido por v10 con default-features = false.
  Sin cambios en src/shield/auth.rs: la API publica de v10 es compatible con v9.

Documentacion:

- `docs/roadmap/STATUS.md` actualizado: criterios de Docker y binario marcados `[x]`,
  benchmarks actualizados con mediciones del 2026-05-21 (pool=50, Docker PostgreSQL).

### Alineacion documental de Fase 1

Cambiado:

- `docs/roadmap/STATUS.md` sincronizado con el estado real de Fase 1
  tras el cierre tecnico de los 11 PRs de RFC-0002 mas los dos
  hotfixes. Marcadas como `[x]` las casillas de `ag-core` operativo,
  HTTP/1.1+HTTP/2, logging estructurado, clippy sin warnings y
  ausencia de `unsafe`. Marcadas como `[/]` con explicacion las
  casillas de cobertura >=80%, CI verde en las cuatro plataformas y
  `cargo audit`, pendientes de validacion oficial. Las metricas
  duras (300K req/s, p99, idle, arranque), blog post y stars siguen
  `[ ]` por requerir hardware de referencia o eventos de comunidad.

### Hotfix de Fase 1 (segundo)

Cambiado:

- Generacion de rutas temporales en tests TLS (`tests/shield_full_pipeline.rs`,
  `tests/shield_tls.rs`, `src/shield/tls.rs::tests`) usa `AtomicUsize`
  + pid del proceso en lugar de `SystemTime::now().as_nanos()`. El
  timestamp en Windows tiene resolucion de ~15ms, lo que producia
  colisiones de archivo entre tests paralelos: dos tests escribian
  cert/key en el mismo path y uno sobreescribia al otro mid-test.
  Sintoma: 3 de 6 tests de `shield_full_pipeline` fallaban
  intermitentemente en `build (windows-x64)`. Verificado con
  `RUST_TEST_THREADS=16`.

### Hotfix de Fase 1

Cambiado:

- `shield::tls::build_acceptor` deja de depender del provider rustls
  global del proceso. Construye `ServerConfig` con
  `ServerConfig::builder_with_provider(ring::default_provider())` y
  `with_safe_default_protocol_versions()`. Cada `TlsAcceptor` lleva
  su propio provider via Arc. Cierra una race observada en CI
  macos-arm64 cuando varios tests E2E inicializaban TLS en paralelo:
  un test podia leer el provider global antes de que otro terminara
  de instalarlo. Los tests E2E ya no necesitan llamar
  `install_default()`; se eliminaron las llamadas en
  `tests/shield_full_pipeline.rs` y `tests/shield_tls.rs`.

### Fase 0 - Fundaciones y gobernanza

Anadido:

- Documentos maestros instalados en `docs/master/`:
  `ANTI-GRAVITAL-Blueprint-v4.0.pdf`,
  `ANTI-GRAVITAL-Arquitectura-Tecnica.md`,
  `ANTI-GRAVITAL-Hoja-de-Ruta.md` y `VERSION.md` con hashes SHA-256.
- Constitucion tecnica del repositorio en `CLAUDE.md`.
- README bilingue espanol e ingles.
- Documentos de gobernanza: `CONTRIBUTING.md`, `GOVERNANCE.md`,
  `SECURITY.md`, `CODE_OF_CONDUCT.md`.
- Estructura de documentacion: `docs/architecture/`, `docs/roadmap/`,
  `docs/modules/`, `docs/dsl/`, `docs/benchmarks/`, `docs/security/`,
  `docs/governance/`, `docs/examples/`, `docs/rfc/`, `docs/adr/`,
  `docs/diagrams/`, `docs/graph/`, `docs/es/`, `docs/en/`.
- Descomposicion verbatim de los maestros en archivos navegables por
  capitulo, fase y modulo.
- Workspace Cargo con 15 crates vacios: `ag-core`, `ag-dsl`,
  `ag-cli`, `ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`,
  `ag-storage`, `ag-observe`, `ag-ui`, `ag-cloud`, `ag-ai`,
  `ag-mobile`, `ag-migrate`, `ag-wasm-host`.
- Configuracion de toolchain: `rust-toolchain.toml`, `rustfmt.toml`,
  `clippy.toml`, `deny.toml`.
- Workflows de CI multiplataforma: `ci.yml`, `quality.yml`, `docs.yml`.
- Plantillas de issue, pull request y RFC en `.github/`.
- ADRs iniciales: `0001-monorepo-workspace.md`,
  `0002-bilingual-documentation.md`, `0003-bdfl-governance.md`,
  `0004-descomposicion-de-maestros.md`,
  `0005-contact-identities.md`.
- Tablero vivo del proyecto en `docs/roadmap/STATUS.md`.
- Lista de entregables externos pendientes en
  `docs/governance/external-deliverables.md`.

Cambiado:

- Identidades de contacto oficiales del proyecto. Los placeholders
  `security@gravital.io` y `hello@antigravital.dev` de los maestros se
  reemplazan por `anti@gravitalcloud.com` (correo raiz) y
  `angelnereira@gravitalcloud.com` (BDFL inicial) en
  `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` (15.3) y
  `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` (Fase 0). Hashes
  recomputados en `docs/master/VERSION.md` con entrada de historial.
  Derivados verbatim regenerados. Registrado en
  `docs/adr/0005-contact-identities.md`.

Sin codigo funcional. El primer hito tecnico (Shield MVP) se entrega
en Fase 1.

### Fase 1 - The Shield MVP (en curso)

Anadido:

- RFC-0001 que autoriza la paralelizacion de las puertas externas de
  Fase 0 con la implementacion de Fase 1 mientras el BDFL trabaja en
  solitario.
- RFC-0002 con el diseno detallado del Shield MVP: stack, modulos,
  features Cargo, configuracion TOML, sistema de errores y plan de
  implementacion en 11 PRs incrementales.
- Estado vivo de Fase 1 reflejado en `docs/roadmap/STATUS.md`.
- Bootstrap del crate `ag-core` con HTTP/1.1 y HTTP/2 funcionales via
  Axum + Tokio (sin TLS aun): modulos `error`, `config`, `runtime`,
  `shield` (capa de logging estructurado), `core` (placeholder).
- `AgError` y `AgResult` con mapeo automatico a respuestas HTTP via
  `IntoResponse`.
- `ShieldConfig` deserializable desde TOML con defaults seguros.
- Dependencias compartidas del workspace declaradas en
  `[workspace.dependencies]` (axum, tokio, tower, tower-http, tracing,
  serde, thiserror, hyper, http, bytes, toml, pin-project-lite).
- Tests: 12 unit tests por modulo, 2 tests E2E con servidor real, 1
  doctest. Todos en verde con `cargo fmt`, `cargo clippy -D warnings`
  y `cargo doc --no-deps` limpios.
- Capa de validacion de payload (`shield::validation`) detras de la
  feature `validation` activa por defecto. Trait `Validate`, agregado
  `ValidationErrors` con `FieldError` serializable y extractor
  `ValidatedJson<T>` que mapea fallos a `AgError::Validation` con
  detalle estructurado por campo (status 422). 4 unit tests
  adicionales y 3 tests E2E sobre `/projects`.
- Capa CORS (`shield::cors`) detras de la feature `cors` activa por
  defecto. Wraps `tower_http::cors::CorsLayer` con configuracion
  declarativa via `CorsConfig` en `ShieldConfig`. Defaults seguros:
  CORS deshabilitado salvo declaracion explicita. Errores de
  configuracion mapeados a `AgError::Cors` con codigo `cors_error`
  (status 403). 4 unit tests sobre construccion y 4 tests E2E sobre
  preflight, origenes listados y rechazados.
- Tower-http feature `cors` activada en el workspace.
- Capa CSRF (`shield::csrf`) detras de la feature `csrf` activa por
  defecto. Patron double-submit cookie apatrida: en peticiones que
  mutan estado (POST, PUT, PATCH, DELETE) se exige que el header y la
  cookie configurados lleven el mismo valor opaco. Configuracion via
  `CsrfConfig` con header por defecto `x-csrf-token` y cookie
  `ag_csrf`. CSRF deshabilitado por defecto. 7 unit tests sobre
  parsing de cookies y validacion y 6 tests E2E sobre flujo completo.
- Capa rate-limit (`shield::rate_limit`) detras de la feature
  `rate-limit` activa por defecto. Token bucket por IP con `governor`
  (dashmap storage). Cuando una IP excede `per_ip_rps`/`burst`, las
  peticiones rebotan con `AgError::RateLimit` (status 429, codigo
  `rate_limit_exceeded`). Sin `ConnectInfo` la capa pasa transparente
  (compatibilidad con tests sin transporte). `RateLimitConfig`
  deshabilitada por defecto, configuracion validada al construir
  Shield. Dependencia opcional `governor = 0.7`. 6 unit tests y 3
  tests E2E.
- Capa de autenticacion JWT Ed25519 (`shield::auth`) detras de la
  feature `auth-jwt` activa por defecto. Verifica el header
  `Authorization: Bearer <token>` contra una clave publica Ed25519
  cargada al arranque desde `AuthConfig.public_key_pem` o
  `public_key_path`. Valida firma, expiracion, issuer opcional y
  audience opcional. Leeway forzado a 0 para evitar tolerancia
  silenciosa a deriva de reloj. Inyecta `AuthContext` en las
  extensiones del request; expone el extractor `Claims<T>` que
  deserializa los claims al tipo de la aplicacion. Fallos mapeados a
  `AgError::Auth` (status 401). Dependencia opcional
  `jsonwebtoken = 9` con feature `use_pem`. 6 unit tests sobre
  parsing y carga de claves y 6 tests E2E sobre flujo completo,
  incluyendo expiracion, firma incorrecta y issuer no esperado.
- Dev-dependencies `ed25519-dalek = 2` y `rand_core = 0.6` para
  generar pares de claves Ed25519 en tests.
- Capa TLS 1.3 (`shield::tls`) detras de la feature `tls` activa por
  defecto. Construye un `tokio_rustls::TlsAcceptor` desde cert/key
  PEM declarados en `TlsConfig`. Provider criptografico: `ring`.
  Cuando la feature `tls` esta presente y `TlsConfig.enabled` es
  cierto, `Shield::serve(listener, router)` opera el accept loop
  envolviendo cada conexion con TLS; en otro caso delega a
  `axum::serve`. Errores de carga mapeados a `AgError::Tls`.
  Dependencias opcionales `rustls = 0.23`, `rustls-pki-types = 1.10`,
  `tokio-rustls = 0.26`. Dev-dependency `rcgen = 0.13` para generar
  certificados auto-firmados en tests. 4 unit tests sobre carga de
  PEM y 3 tests E2E incluyendo handshake HTTPS real con reqwest.

Cambiado:

- Migracion de `rustls-pemfile` (archivado, RUSTSEC-2025-0134) al
  trait `PemObject` de `rustls-pki-types`. La superficie de API
  publica no se altera.
- Tests E2E del pipeline Shield completo en
  `crates/ag-core/tests/shield_full_pipeline.rs`. Arrancan un servidor
  con TODAS las capas activas simultaneamente (TLS, auth-jwt, csrf,
  cors, rate-limit, validation, logging) sobre HTTPS real con cert
  auto-firmado y par Ed25519 generados en setup. 6 tests cubren:
  request valido pasa por toda la pipeline (GET y POST), JWT invalido
  bloqueado por auth (401), CSRF ausente bloqueado en POST (403),
  payload invalido bloqueado por validation (422), origen no listado
  no recibe header allow-origin.
- Ejemplo binario release-ready
  `crates/ag-core/examples/hello_world.rs` para correr con
  `cargo run --release -p ag-core --example hello_world` y medir con
  `oha`, `wrk` o equivalente.
- Plantilla `docs/benchmarks/measurement-template.md` para registrar
  oficialmente las metricas duras de cierre de Fase 1 (throughput,
  p99, memoria idle, arranque) cumpliendo la regla 17. RFC-0002 PR 10
  de 11.
- Rustdoc enriquecido de `ag-core` con introduccion crate-level
  ampliada (tabla de capas, features, ejemplos de uso con
  `Shield::serve` y `ShieldConfig::from_path`, lista de tipos
  publicos clave y enlaces cruzados a maestro y manual). Dos
  doctests compilan en CI.
- Capitulo 1 del manual de usuario: `docs/manual/01-shield-as-library.md`
  con once secciones cubriendo instalacion como dependencia git,
  servidor minimo, carga de configuracion desde TOML, orden de
  capas, activacion de TLS / JWT / CSRF / rate-limit, extractor
  `Claims<T>` y `ValidatedJson<T>`, errores y observabilidad,
  recomendaciones de despliegue y referencias cruzadas. Cierra el
  ultimo entregable en-repo de Fase 1. RFC-0002 PR 11 de 11.

Cambiado:

- `Shield::serve` inyecta `ConnectInfo<SocketAddr>` en cada request
  tanto en transporte plano como TLS. En el camino plano se usa
  `Router::into_make_service_with_connect_info::<SocketAddr>()`; en
  el camino TLS se captura `peer_addr` antes del handshake y se
  inyecta como extension del request. Antes de este fix, la capa
  rate-limit pasaba transparente sobre cualquier servicio arrancado
  via `Shield::serve` porque la IP del cliente no llegaba al
  middleware. La firma publica de `Shield::serve` no cambia.

- Benchmark Hello World del Shield con criterion en
  `crates/ag-core/benches/shield_hello_world.rs`. Tres grupos
  comparables a nivel Tower: `bare_axum_hello` (linea base),
  `shield_default_hello` (Shield con solo logging) y
  `shield_full_default_hello` (Shield con CORS, CSRF y rate-limit
  activas). Documentado en `crates/ag-core/benches/README.md` con la
  regla 17 (hardware, OS, version Rust, commit, configuracion,
  metodologia, ejecuciones, desviacion estandar). Dev-dependency
  `criterion = 0.5`. RFC-0002 PR 9 de 11.

- Configuracion TOML completa del Shield: `ShieldConfig::from_path`
  carga la configuracion desde un archivo, `to_toml_string` permite
  round-trip estable, y todas las structs llevan
  `#[serde(deny_unknown_fields)]` para rechazar typos con
  `AgError::Config` en vez de ignorarlos silenciosamente. Ejemplo
  documentado en `crates/ag-core/config.example.toml` con todas las
  secciones (`bind`, `runtime`, `cors`, `csrf`, `rate_limit`, `auth`,
  `tls`) y sus defaults explicados. 14 unit tests sobre parsing por
  seccion, rechazo de claves desconocidas, round-trip y carga desde
  disco. RFC-0002 PR 8 de 11.

- Flujo de pull requests con autofill automatizado: cada rama trae su
  descriptor pre-rellenado bajo `docs/pr-drafts/<rama-aplanada>.md`
  (las `/` de la rama se convierten en `-`). El nuevo workflow
  `.github/workflows/pr-autofill.yml` se dispara al abrir o reabrir
  la pull request, busca el descriptor y reemplaza el cuerpo del PR
  con su contenido completo. Si no encuentra descriptor, comenta el
  PR avisando. La plantilla `.github/PULL_REQUEST_TEMPLATE.md` queda
  como aviso visible solo en ese caso. Regla incorporada a
  `CLAUDE.md` y `CONTRIBUTING.md`. Descriptor de la rama
  `phase-0/foundations-and-governance` publicado en
  `docs/pr-drafts/phase-0-foundations-and-governance.md`.

Cambiado:

- API publica de `Shield`: `Shield::layer()` reemplazado por
  `Shield::apply(router)`. La nueva firma oculta la complejidad de
  tipos de la pipeline y permite agregar capas sin romper la
  superficie publica en cada PR.
- `Shield::try_new(config)` valida la configuracion en construccion
  (origenes, metodos y headers de CORS); `Shield::new(config)` mantiene
  semantica de panic para casos de prototipado.
- Workflow `quality.yml`: `cargo deny` ya pasa tras anadir
  `Unicode-3.0` a `deny.toml` (commit anterior).

[Unreleased]: https://github.com/anti-gravital/anti-gravital/compare/HEAD..HEAD
