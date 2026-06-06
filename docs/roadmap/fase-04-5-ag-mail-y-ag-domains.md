# Fase 4.5 - ag-mail + ag-domains: comunicacion y dominios

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-04-modulos-estandar.md](./fase-04-modulos-estandar.md)
> Siguiente: [fase-05-ag-cloud.md](./fase-05-ag-cloud.md)
> Origen de la decision: [docs/adr/0007-ag-mail-ag-domains.md](../adr/0007-ag-mail-ag-domains.md)

## Fase 4.5 — `ag-mail` + `ag-domains`: comunicación y dominios

**Objetivo.** Añadir capacidades operativas de comunicación transaccional, DNS,
TLS y dominios sin sobrecargar la Fase 4 ni retrasar los módulos estándar.
Prepara el terreno para que `ag-cloud` (Fase 5) despliegue aplicaciones con
dominio, certificado y correo transaccional usando una experiencia integrada.
La introducción de esta fase está oficializada en `ADR-0007`.

**Duración:** 1–2 meses.

### 4.5.1 Criterios de entrada

- [ ] Fase 4 completada con todos sus criterios de salida marcados.
- [ ] `ag-auth` expone hooks/eventos para verificación de correo, recuperación
  de contraseña y magic links.
- [ ] `ag-observe` registra métricas y trazas de jobs asíncronos.
- [ ] RFC aprobado para el alcance inicial de `ag-mail`.
- [ ] RFC aprobado para el alcance inicial de `ag-domains`.

### 4.5.2 Entregables

- [ ] Crate `ag-mail` (estándar diferido): sender SMTP outbound nativo
  (`lettre` + `rustls`) más trait `MailSender`; para proveedores externos se usa el relay SMTP nativo.
- [ ] Templates HTML/plaintext con `askama` tipados, validados en compile-time
  contra `schema.ag`.
- [ ] Declaración de correos en `schema.ag` (bloque `mail`).
- [ ] Integración `ag-auth` → `ag-mail` para verificación, recuperación y
  magic links, vía trait pequeño definido en `ag-auth`.
- [ ] Cola asíncrona con reintentos y backoff exponencial; backend en memoria
  por defecto, persistente vía `ag-data` opcional.
- [ ] Métricas hacia `ag-observe`: `ag_mail_sent_total`, `ag_mail_failed_total`,
  `ag_mail_retry_total`, histograma de latencia.
- [ ] Crate `ag-domains` (opcional infra): trait `DnsProvider` con adapter
  Cloudflare; modelo declarativo A/AAAA/CNAME/TXT/MX.
- [ ] Soporte ACME (Let's Encrypt) vía `instant-acme`: emisión y renovación
  automática, challenge DNS-01 preferido, HTTP-01 alternativo.
- [ ] Generación de SPF/DKIM/DMARC requeridos por `ag-mail` (cooperación
  `ag-mail` ↔ `ag-domains` sin ciclo de dependencia).
- [ ] Verificación de propagación contra múltiples resolvers públicos
  (`hickory-resolver`).
- [ ] DSL v0.7: bloques `mail`, `domain`, `dns`, `tls`; el compilador valida
  que el `from` referencia un `domain` declarado, que el template existe y
  que las variables del HTML coinciden con las `vars` tipadas.
- [ ] Actualización del LSP `ag-lsp` para los bloques nuevos.
- [ ] Comandos CLI: `ag domains check`, `ag domains sync`, `ag mail test`.
- [ ] Example `auth-mail-demo` en `examples/`: registro + verificación por
  correo + magic link.
- [ ] Documentación: "Configurar dominio, TLS y correo transaccional con
  Anti-Gravital".

### 4.5.3 Criterios de salida (puerta antes de Fase 5)

- [ ] `ag-mail` envía correo transaccional HTML y plaintext desde un proyecto
  Anti-Gravital vía sender nativo **y** vía al menos un adapter.
- [ ] `ag-auth` usa `ag-mail` para verificación de correo y recuperación de
  contraseña en el example `auth-mail-demo`.
- [ ] `ag-domains` crea y verifica registros DNS en al menos un proveedor
  real.
- [ ] `ag-domains` emite y renueva certificados TLS vía ACME en entorno de
  prueba (Let's Encrypt staging).
- [ ] `ag-domains` genera SPF/DKIM/DMARC requeridos por `ag-mail`.
- [ ] `ag domains check`, `ag domains sync` y `ag mail test` funcionan en CI
  reproducible.
- [ ] Cobertura de tests unitarios e integración ≥ 75 % en ambos crates.
- [ ] Cero dependencias circulares con `ag-core`, `ag-dsl`, `ag-auth` o
  `ag-cloud` (job de CI verde).
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo audit` y
  `cargo deny check` verdes.

### 4.5.4 Riesgos de la fase

El riesgo principal es **confundir `ag-mail` con un MTA completo**. La
mitigación es la restricción explícita del alcance v1 a outbound + adapters;
inbound, IMAP/POP, buzones persistentes y antispam quedan documentados como
fuera de alcance, no como "diferidos a v2".

El segundo riesgo es la **dependencia de upstreams jóvenes** (`instant-acme`,
`hickory-resolver`) en dominios donde los bugs se pagan caro: un certificado
que no renueva tumba el sitio. La mitigación es un trait `DnsProvider`
pequeño y versionado con tests de contrato, pinning explícito en el
workspace, y vigilancia activa de la evolución de los crates.

El tercer riesgo es **convertir Anti-Gravital en un panel de hosting** por
acumulación de capacidades. La mitigación es la regla de interoperabilidad
del proyecto: ambos crates son abstracciones con adapters, no reemplazos de
proveedores. La frontera está fijada en `ADR-0007` y no se mueve sin un nuevo
ADR.

### 4.5.5 Nota futura — Fase 4.6 MTA nativo (`ADR-0010`)

Esa frontera de `ADR-0007` ya se movió, mediante el nuevo ADR que ella misma
exigía. `ADR-0010` (2026-06-03) supersede la restricción v1 "NO es un MTA /
inbound nunca" y expande `ag-mail` a un MTA outbound nativo, por fases y
opt-in tras features de Cargo, conservando el patrón Native | Adapter y el
baseline implementado de la Fase 4.5. Plan técnico: `RFC-0009`. La Fase 4.5
sigue completa para su alcance original. La Fase 4.6-A (núcleo del MTA:
resolución MX, ESMTP+STARTTLS, firma DKIM Ed25519, clasificación de bounces)
está implementada tras la feature opt-in `mta`; 4.6-B..D siguen pendientes.
Origen de la decisión:
[docs/adr/0010-ag-mail-native-mta-pivot.md](../adr/0010-ag-mail-native-mta-pivot.md).

---
