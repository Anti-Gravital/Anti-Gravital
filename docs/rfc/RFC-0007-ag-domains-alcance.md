# RFC-0007: ag-domains — alcance, stack tecnico y plan de implementacion

- Estado: aceptada
- Autor: Angel Nereira (BDFL), Gravital Labs
- Fecha de borrador: 2026-05-24
- Fase objetivo: Fase 4.5
- Modulos o crates afectados: `ag-domains` (nuevo), `ag-cloud` (consumidor en Fase 5), `ag-mail` (cooperacion SPF/DKIM/DMARC), `ag-dsl` (v0.7)
- RFC predecesora: ninguna
- ADR gobernante: `docs/adr/0007-ag-mail-ag-domains.md` (Aprobado 2026-05-23)
- Periodo de comentarios: omitido por decision del BDFL (este RFC formaliza el detalle tecnico de un ADR ya aprobado por la regla 28 del CLAUDE.md, que exige RFC antes de introducir nuevos crates).

## 1. Motivacion

`ADR-0007` decidio introducir `ag-domains` como modulo opcional de
infraestructura para resolver dominios y TLS dentro del flujo de
`ag deploy`. Este RFC fija el detalle tecnico de implementacion: librerias
ACME y DNS resolver, layout del crate, estrategia de tests con red real
(Cloudflare API token, Let's Encrypt staging), y el contrato de tests del
trait `DnsProvider` que todo adapter debe pasar.

## 2. Problema

El ADR define **que** capacidad cubre `ag-domains` (DNS via adapters, ACME,
SPF/DKIM/DMARC, verificacion de propagacion) y **que NO cubre** (registrador,
reemplazo de Terraform). El **como** queda abierto:

- Que cliente ACME se usa (`instant-acme`, `acme-rs`, propio).
- Que DNS resolver se usa para verificacion (`hickory-resolver`, `trust-dns`).
- Como se generan claves para CSR (`rcgen`, propio).
- Como se almacenan certificados (filesystem, `ag-storage`).
- Como se contractualizan los adapters (`#[async_trait]` MailSender style,
  o composicion sin trait).
- Como se testean adapters sin credenciales reales en CI publico.

Sin acuerdo en estos detalles, la PR de implementacion se convierte en una
sequencia de re-trabajos. Este RFC los fija antes de tocar codigo.

## 3. Alternativas consideradas

### A. Implementar ACME a mano

Ventaja: cero dependencias externas; control total.

Inconveniente: ACME es un protocolo complejo (RFC 8555 + extensions);
implementarlo correctamente toma meses. Los upstreams (`instant-acme`)
ya resuelven los casos de uso comunes.

### B. Soportar HTTP-01 unicamente y diferir DNS-01

Ventaja: implementacion mas simple, no requiere `DnsProvider`.

Inconveniente: HTTP-01 obliga a tener el servidor escuchando en :80 antes
del despliegue, lo que rompe el flujo "compra dominio + `ag deploy`". El
ADR fija DNS-01 como preferido y HTTP-01 como alternativo precisamente
para evitar ese requisito.

### C. Hacer `ag-domains` un binario CLI separado, no un crate librerie

Ventaja: aislamiento maximo.

Inconveniente: contradice el ADR (clasificacion "Opcional infra" implica
crate librerie consumido por `ag-cloud`).

### D (elegida). Crate librerie con trait `DnsProvider` + cliente ACME + resolver de propagacion

Estructura modular. Cliente ACME via `instant-acme`. Resolver via
`hickory-resolver`. Generacion de claves via `rcgen` (ya en workspace).
Adapter Cloudflare inicial. Trait con tests de contrato que todo adapter
futuro debe pasar.

## 4. Diseno propuesto

### 4.1 Layout del crate

```
crates/ag-domains/
|- Cargo.toml
|- README.md
|- src/
|  |- lib.rs                 # AgDomains (struct orquestador), re-exports
|  |- error.rs               # AgDomainsError con From para reqwest, instant_acme, hickory
|  |- record.rs              # DnsRecord, RecordType (A/AAAA/CNAME/TXT/MX), DnsRecordSpec
|  |- provider/
|  |  |- mod.rs              # trait DnsProvider + contract_tests helper
|  |  |- cloudflare.rs       # CloudflareDnsProvider                     [feature "cloudflare"]
|  |- acme/
|  |  |- mod.rs              # AcmeClient (wrapper instant-acme)         [feature "acme"]
|  |  |- challenge.rs        # DNS-01 + HTTP-01 challenge handling
|  |  |- renewal.rs          # task Tokio de renovacion automatica
|  |- mail_records.rs        # generate_mail_records (SPF/DKIM/DMARC)
|  |- propagation.rs         # verificacion via hickory-resolver
|  |- metrics.rs             # contadores hacia ag-observe
|- tests/
   |- contract.rs            # tests de contrato del trait DnsProvider
   |- integration.rs         # tests con red real, todos #[ignore]
```

### 4.2 API publica minima

```rust
#[async_trait::async_trait]
pub trait DnsProvider: Send + Sync {
    async fn list_records(&self, zone: &str) -> Result<Vec<DnsRecord>, AgDomainsError>;
    async fn upsert_record(&self, zone: &str, record: &DnsRecord) -> Result<(), AgDomainsError>;
    async fn delete_record(&self, zone: &str, id: &str) -> Result<(), AgDomainsError>;
    fn provider_name(&self) -> &'static str;
}

pub struct DnsRecord {
    pub id: Option<String>,
    pub record_type: RecordType,
    pub name: String,
    pub value: String,
    pub ttl: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordType { A, AAAA, CNAME, TXT, MX }

pub struct AgDomains<P: DnsProvider> {
    provider: P,
    // ...
}
```

Trait pequeno, versionado. Los **tests de contrato** son una funcion
publica `contract_tests::<P>(provider: P)` que cualquier adapter futuro
puede invocar en sus propios tests para verificar conformidad. Esto
amortiza el costo del trait y previene drift entre adapters.

### 4.3 Dependencias nuevas (con feature gates)

| Crate | Version | Feature | Justificacion |
|---|---|---|---|
| `instant-acme` | 0.7 | `acme` (default) | Cliente ACME maduro, async, RFC 8555 + 8737. |
| `hickory-resolver` | 0.24 | `propagation` (default) | DNS resolver async puro Rust. |
| `rcgen` | (workspace) | `acme` | CSR y claves para ACME. Ya en workspace. |
| `reqwest` | (workspace) | `cloudflare` | HTTP client para Cloudflare API. Ya en workspace. |
| `serde`, `serde_json` | (workspace) | always | Serializacion de records y respuestas. |
| `tokio` | (workspace) | always | Runtime y tasks de renovacion. |

Dev-dependencies:

| Crate | Version | Uso |
|---|---|---|
| `wiremock` | 0.6 | Mocks HTTP para tests de adapters sin red real. |

### 4.4 Estrategia de tests

Tres niveles:

1. **Unit tests** en cada modulo (`#[cfg(test)] mod tests`):
   - Serializacion de `DnsRecord` y `RecordType`.
   - Parsing de respuestas Cloudflare (con strings hardcoded).
   - Logica del renewal scheduler con `tokio::time::pause`.
   - Comportamiento del trait sin red.

2. **Tests de contrato** en `tests/contract.rs`:
   - Plantilla generica `contract_tests::<P: DnsProvider>(provider: P)` que
     ejecuta create/list/delete contra el provider. Se usa con un
     `MockDnsProvider` en CI sin credenciales.

3. **Tests de integracion** en `tests/integration.rs`, todos `#[ignore]`:
   - `cloudflare_real_record` requiere `CLOUDFLARE_API_TOKEN`,
     `CLOUDFLARE_ZONE_ID`, `AG_DOMAINS_TEST_DOMAIN`.
   - `acme_staging_issue` requiere los anteriores mas
     `LETSENCRYPT_STAGING_EMAIL`. Usa solo el endpoint de **staging** de
     Let's Encrypt, nunca production en CI.
   - Cada test documenta env vars en doc-comment del `#[ignore]`.

CI publico corre niveles 1 y 2. CI privado (workflow protegido con
`environment: production`) puede correr nivel 3 en push a `main`.

### 4.5 Cambios en CI o tooling

- Nuevo job `domains-contract` en `ci.yml` que corre `cargo test -p
  ag-domains --features "cloudflare,acme"` sin secrets (niveles 1 y 2).
- Job opcional `domains-integration` corre con `-- --ignored` cuando hay
  secrets disponibles (Fase 4.5 punto de integracion CI).
- `cargo deny` y `cargo audit` validan que `instant-acme` y
  `hickory-resolver` no introduzcan vulnerabilidades.

### 4.6 Cambios en DSL (v0.7)

Bloque `domain` en `ag-dsl` v0.7:

```ag
domain plenty.market {
    provider "cloudflare"
    tls { mode auto  acme true }
    dns {
        CNAME "api"     -> "ag-cloud-target"
        TXT   "_dmarc"  -> "v=DMARC1; p=quarantine"
    }
    mail { spf auto  dkim auto  dmarc quarantine }
}
```

El generador Rust de `ag-dsl` materializa el bloque como configuracion para
el orquestador `AgDomains`. La validacion semantica confirma que el
`provider` referenciado tiene adapter compilado (feature activa).

### 4.7 Cambios en documentacion maestra

Ninguno adicional: `ADR-0007` ya actualizo Arquitectura-Tecnica §8.9 y
§10.6. Este RFC complementa el ADR con el detalle de implementacion.

## 5. Plan de implementacion

PRs en orden:

1. **PR Etapa 2-1**: workspace + skeleton conjunto de `ag-mail` y `ag-domains`.
2. **PR Etapa 2-2**: trait `DnsProvider` + `DnsRecord` + `RecordType` +
   `AgDomainsError` + tests unitarios. Sin adapter.
3. **PR Etapa 2-3**: `CloudflareDnsProvider` + tests con `wiremock` +
   tests `#[ignore]` con credenciales reales.
4. **PR Etapa 2-4**: cliente ACME (`instant-acme`) + renewal + propagation
   (`hickory-resolver`).
5. **PR Etapa 2-7**: cooperacion `generate_mail_records` con `ag-mail`.
6. **PR Etapa 2-8**: DSL v0.7 (bloques `mail`, `domain`).
7. **PR Etapa 2-9**: CLI (`ag domains check`, `ag domains sync`).

## 6. Riesgos

| Riesgo | Probabilidad | Impacto | Mitigacion |
|---|---|---|---|
| `instant-acme` upstream joven | Media | Alto (cert no renueva) | Vigilancia activa, pinning explicito, abstraccion via `AcmeClient`. |
| Rate limits de Let's Encrypt en CI | Alta | Bajo (test falla) | Usar staging endpoint en CI; production solo en operacion real. |
| API de Cloudflare cambia | Baja | Medio (regression) | Tests de contrato fuerzan a actualizar el adapter; mock con responses canonicas. |
| Convertirse en panel de hosting | Baja | Alto (alcance) | ADR fija frontera; rechazar PRs que añadan funciones de gestion arbitraria de zonas. |
| Hickory resolver inconsistencia en CI runners | Baja | Medio (test flaky) | Configurar resolvers explicitos en tests; reintentar verificacion N veces antes de fallar. |

## 7. Impacto

- **Alcance del producto:** cumple el ADR, no expande alcance.
- **Cronograma:** dentro de 1-2 meses de la Fase 4.5.
- **Complejidad operacional:** ninguna obligatoria (modulo opcional).
- **APIs publicas:** crate nuevo. `ag-cloud` (Fase 5) lo consumira via API
  del crate, no via `Cargo.toml` rigido.
- **Documentacion:** este RFC + actualizaciones al README de `ag-domains`.

## 8. Rollback

`ag-domains` es opcional de infraestructura. Si la implementacion descubre
que `instant-acme` o `hickory-resolver` no son viables, el trait
`DnsProvider` y la abstraccion `AcmeClient` permiten swap. Si el crate
entero se vuelve inviable, eliminarlo no afecta al resto del workspace:
`ag-cloud` puede operar sin el (los proyectos que necesiten dominio
delegan a Terraform).

## 9. Decision

- Decisor: Angel Nereira (BDFL)
- Fecha de decision: 2026-05-24
- Resultado: aceptada
- Justificacion: formaliza tecnicamente el alcance fijado por `ADR-0007`
  sin contradecir ninguna restriccion. Cumple la regla 28 del CLAUDE.md.

## 10. Referencias

- `docs/adr/0007-ag-mail-ag-domains.md` — decision gobernante.
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` § 8.9 y § 10.6.
- `docs/roadmap/fase-04-5-ag-mail-y-ag-domains.md` — criterios.
- `docs/modules/ag-domains/README.md` — ficha del modulo.
- `CLAUDE.md` reglas 14 (crates), 15 (dependencias), 16 (seguridad),
  22 (RFC), 28 (RFC para nuevos crates).
- RFC-0006 — alcance complementario de `ag-mail`.
- RFC 8555 (ACME) — protocolo gobernante del cliente ACME.
