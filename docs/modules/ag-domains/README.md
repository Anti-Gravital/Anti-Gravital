# ag-domains

> Capitulo de arquitectura: `docs/architecture/08-modulos-batteries-included.md`, seccion 8.9.
> Integracion con `ag-cloud`: `docs/architecture/10-despliegue-ag-cloud.md`, seccion 10.6.
> ADR de decision: `docs/adr/0007-ag-mail-ag-domains.md`.
> Fase de la Hoja de Ruta: `docs/roadmap/fase-04-5-ag-mail-y-ag-domains.md`.
> README del crate: `crates/ag-domains/README.md` (se creara en la implementacion de la Fase 4.5).
> Criticidad: Opcional infra.
> Fase de implementacion: Fase 4.5. Estado: Pendiente (rama `fase-4.5`).

## Dominio

Gestion declarativa de dominios y TLS para proyectos Anti-Gravital. Trait
`DnsProvider` con adapters (Cloudflare inicialmente). Modelo declarativo de
registros A/AAAA/CNAME/TXT/MX. Cliente ACME para Let's Encrypt. Generacion de
SPF/DKIM/DMARC para `ag-mail`. Verificacion de propagacion contra multiples
resolvers publicos.

## Alcance (Fase 4.5)

### Si

- Modelo declarativo de DNS aplicado via adapter de proveedor.
- ACME (DNS-01 / HTTP-01) con renovacion automatica.
- Generacion de registros SPF / DKIM / DMARC para `ag-mail`.
- Verificacion de propagacion contra multiples resolvers publicos.

### No

- Comprar / registrar dominios (se compran externamente, p. ej. Namecheap,
  Cloudflare Registrar).
- Reemplazar Terraform o Pulumi para infraestructura compleja multi-cloud.
- Administrar zonas DNS arbitrarias fuera del ambito declarado en `schema.ag`.

## Stack tecnico planificado

| Componente | Libreria | Notas |
|---|---|---|
| Cliente ACME | `instant-acme` | Let's Encrypt. DNS-01 preferido, HTTP-01 alternativo. |
| Resolver DNS | `hickory-resolver` | Verificacion de propagacion contra varios resolvers publicos. |
| Generacion claves | `rcgen` | CSRs y claves para certificados. |
| HTTP adapters | `reqwest` | Cloudflare API token. Diseno extensible (Route53, Namecheap). |
| Trait async | `async-trait` | `DnsProvider`. |
| Observabilidad | `ag-observe` | Metricas y trazas de operaciones DNS y ACME. |

Las dependencias `instant-acme` y `hickory-resolver` son **upstreams jovenes** en
un dominio donde los bugs se pagan caro (un cert que no renueva tumba el sitio).
Mitigacion: trait `DnsProvider` pequeno y versionado con tests de contrato,
pinning explicito en el workspace, vigilancia activa de la evolucion de los
crates. Politica de dependencias del CLAUDE.md regla 15.

## API publica planificada

```rust
#[async_trait::async_trait]
pub trait DnsProvider: Send + Sync {
    async fn list_records(&self, zone: &str) -> Result<Vec<DnsRecord>, AgDomainsError>;
    async fn upsert_record(&self, zone: &str, record: &DnsRecord) -> Result<(), AgDomainsError>;
    async fn delete_record(&self, zone: &str, id: &str) -> Result<(), AgDomainsError>;
    fn provider_name(&self) -> &'static str;
}
```

Pequeno, versionado, con **tests de contrato** que todo adapter debe pasar.
Adapter inicial: Cloudflare (autenticacion por API token). Disenado para anadir
Route53, Namecheap, DigitalOcean, etc. en iteraciones posteriores sin tocar la
superficie publica.

## Dependencias internas permitidas

- Puede depender de `ag-core`, `ag-observe` (metricas) y `ag-storage`
  (almacenamiento opcional de certificados).
- **NO puede depender de `ag-mail`.** La cooperacion para materializar
  SPF/DKIM/DMARC se hace en una sola direccion: `ag-mail` declara los requisitos
  via `MailSender::dns_requirements` y `ag-domains` los aplica.
- Es consumido por `ag-cloud` durante `ag deploy`, sin dependencia rigida en
  todos los targets (septima regla del capitulo 5, ADR-0007).

## DSL v0.7 — bloque `domain`

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

## Integracion con `ag-cloud` (Fase 5)

`ag-cloud` consume `ag-domains` durante `ag deploy` en seis pasos:

1. Validar control del dominio (registro TXT de verificacion).
2. `upsert_record` para apuntar el dominio al target del despliegue.
3. Emitir o renovar TLS via ACME contra Let's Encrypt.
4. Asociar el dominio al target (Caddy, Fly, Railway, k8s ingress).
5. Materializar SPF/DKIM/DMARC declarados por `ag-mail`.
6. Verificar propagacion antes de marcar el despliegue exitoso.

`ag-cloud` puede saltarse pasos individuales (p. ej. paso 3 si TLS lo gestiona
un balanceador externo). El detalle vive en
`docs/architecture/10-despliegue-ag-cloud.md`, seccion 10.6.

## Criterios de salida (puerta de Fase 4.5)

- [ ] `ag-domains` crea y verifica registros DNS en al menos un proveedor real.
- [ ] `ag-domains` emite y renueva certificados TLS via ACME en entorno de
      prueba (Let's Encrypt staging).
- [ ] `ag-domains` genera SPF/DKIM/DMARC requeridos por `ag-mail`.
- [ ] `ag domains check` y `ag domains sync` funcionan en CI reproducible.
- [ ] Cobertura de tests unitarios e integracion >= 75%.
- [ ] Cero dependencias circulares (job de CI verde).
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo audit` y
      `cargo deny check` verdes.

## Riesgos vigilados

- **Conversion accidental en registrador.** Mitigacion: ADR-0007 fija
  explicitamente el alcance; el dominio se compra externamente.
- **Bugs en upstreams jovenes** (`instant-acme`, `hickory-resolver`).
  Mitigacion: trait pequeno con tests de contrato; reemplazo por adapter
  alternativo si un upstream se vuelve inviable.
- **Convertir a Anti-Gravital en un panel de hosting.** Mitigacion: regla de
  interoperabilidad (capitulo 3.3). `ag-domains` es abstraccion + adapters, no
  reemplazo de Terraform ni de proveedores DNS.
