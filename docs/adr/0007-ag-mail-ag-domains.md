## ADR-0007: Incorporación de `ag-mail` y `ag-domains` como Fase aditiva 4.5

**Estado:** Aprobado
**Fecha:** 2026-05-23
**Autor:** Angel Nereira (BDFL)
**Crates afectados:** nuevos `ag-mail`, `ag-domains`; consumidores `ag-auth`, `ag-cloud`; generador `ag-dsl`
**Documentos maestros tocados:** `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md`, `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`, `docs/master/ANTI-GRAVITAL-Blueprint-v4.0.pdf` (re-export v4.1)

---

## Contexto

La Hoja de Ruta cierra la Fase 4 con los cinco módulos estándar batteries-included
(`ag-auth`, `ag-cache`, `ag-realtime`, `ag-storage`, `ag-observe`) y abre la Fase 5
con `ag-cloud` y el hito v0.5 BETA. La narrativa de la Fase 5 es "despliega un
binario Anti-Gravital con un comando y obtén una API en producción".

Esa narrativa requiere dos capacidades operacionales que el ecosistema actual no
cubre y que NO son responsabilidad de `ag-cloud`:

1. **Comunicación transaccional outbound.** `ag-auth` ya genera flujos de
   verificación de correo, recuperación de contraseña y magic links, pero hoy
   esos flujos no tienen un emisor real dentro del ecosistema. El consumidor
   queda obligado a integrar Resend, SES, Postmark o un SMTP propio "por fuera",
   lo cual rompe la promesa schema-first del proyecto: el correo declarado en
   `schema.ag` no se valida contra ninguna implementación.

2. **Gestión declarativa de dominios y TLS.** Para que `ag deploy` entregue una
   URL `https://miapi.example.com` con certificado válido necesita configurar
   DNS, emitir/renovar certificados ACME, y publicar los registros SPF/DKIM/DMARC
   que la capa de correo requiere. Hoy esto vive como tarea humana fuera del
   contrato.

Construir estas dos capacidades **dentro** de la Fase 4 sobrecarga una fase ya
densa y atrasa el hito v0.5 BETA. Posponerlas a una Fase 5 ampliada confunde el
alcance de `ag-cloud`, que se define como "despliegue simplificado", no como
"plataforma de correo + DNS + despliegue".

La regla de interoperabilidad del Blueprint (§3.3) — *cuando exista una
herramienta dominante en un dominio adyacente, la estrategia es integrar, no
reemplazar* — impone una restricción adicional: las dos capacidades deben
modelarse como abstracciones con adapters, no como reemplazos nativos de Resend,
SES, Cloudflare, Route53 ni Let's Encrypt.

---

## Decisión

Se introduce una **Fase 4.5 aditiva** entre la Fase 4 (completa) y la Fase 5
(pendiente) con dos crates nuevos:

- **`ag-mail`** — clasificación **Estándar diferido**.
  Outbound transaccional. Sender SMTP nativo (`lettre` + `rustls`) **y** adapters
  de primera clase (Resend, SES, Postmark). Templates `askama` tipados validados
  en compile-time contra `schema.ag`. Cola asíncrona con reintentos y backoff
  exponencial. Métricas hacia `ag-observe`. Consumido por `ag-auth` para
  verificación, recuperación y magic links.

- **`ag-domains`** — clasificación **Opcional infra**.
  Trait `DnsProvider` con adapter inicial Cloudflare. Modelo declarativo de
  registros A/AAAA/CNAME/TXT/MX. Cliente ACME para Let's Encrypt
  (`instant-acme`). Generación de SPF/DKIM/DMARC requeridos por `ag-mail`.
  Verificación de propagación (`hickory-resolver`). Consumido por `ag-cloud` en
  el flujo `ag deploy`.

El DSL incorpora bloques `mail` y `domain` en la versión **v0.7**, con
validación de cierre (`from` debe referenciar un `domain` declarado, el template
debe existir, las variables del HTML deben coincidir con las `vars` tipadas).
Los hooks de plugin previstos originalmente para v0.7 se posponen a **v0.8 — Fin
Fase 9**, manteniendo la cadencia hasta la congelación v1.0 en Fase 10.

---

## Alcance y restricciones

### Lo que `ag-mail` SÍ hace en v1

- Envío outbound de correo transaccional.
- Sender SMTP nativo + adapters Resend / SES / Postmark como features de Cargo.
- Templates HTML y plaintext con validación build-time de variables.
- Cola asíncrona con reintentos, backoff y métricas.

### Lo que `ag-mail` NO hace en v1 (fuera de alcance, no diferible "luego")

- Servidor MTA completo.
- Recepción de correo (inbound, IMAP/POP).
- Buzones persistentes.
- Antispam, filtrado, reputación de IP.
- Gestión avanzada de bounces más allá de registro.

### Lo que `ag-domains` SÍ hace

- Modelo declarativo de DNS aplicado vía adapter de proveedor.
- ACME (DNS-01 / HTTP-01) con renovación automática.
- Generación de registros SPF / DKIM / DMARC para `ag-mail`.
- Verificación de propagación contra múltiples resolvers públicos.

### Lo que `ag-domains` NO hace

- Comprar / registrar dominios (se compran externamente, p. ej. Namecheap).
- Reemplazar Terraform o Pulumi para infraestructura compleja multi-cloud.
- Administrar zonas DNS arbitrarias fuera del ámbito declarado en `schema.ag`.

### Reglas de dependencia (verificadas en CI)

- `ag-mail` puede depender de `ag-core`, `ag-data` (cola persistente opcional),
  `ag-realtime` (fan-out opcional), `ag-observe` (métricas) y `ag-domains`
  (cooperación para SPF/DKIM/DMARC).
- `ag-mail` **NO** puede depender de `ag-auth`. Es `ag-auth` quien consume
  `ag-mail` definiendo un trait pequeño que invoca.
- `ag-domains` puede depender de `ag-core`, `ag-observe` y `ag-storage`
  (almacenamiento opcional de certificados). **NO** puede depender de
  `ag-mail`.
- `ag-cloud` consume `ag-domains` durante `ag deploy`, sin dependencia rígida en
  todos los targets (si el proyecto no declara dominios, el flujo se omite).
- El núcleo (`ag-core`, `ag-dsl`, `ag-cli`, `ag-wasm-host`) **NO** cambia.
- Cero ciclos verificables por el job de CI existente.

---

## Consecuencias

**Positivas:**

- El ecosistema cierra la promesa schema-first sobre comunicación y dominios:
  un correo mal formado o un dominio inconsistente es un error de compilación,
  no un bug de runtime.
- `ag-auth` deja de obligar al consumidor a integrar un proveedor de correo por
  fuera. La verificación, recuperación y magic links quedan dentro del binario.
- `ag-cloud` (Fase 5) puede entregar la narrativa "binario + dominio + TLS +
  correo" sin que su propio alcance crezca.
- La regla de interoperabilidad se mantiene literal: ambos crates son
  abstracción + adapters, igual que `ag-storage` integra S3 y `ag-cache`
  integra Redis.
- El núcleo no se contamina con preocupaciones de DNS ni de MTA.

**Negativas:**

- El conteo de crates del ecosistema pasa de **15 a 17**.
- El cronograma total estimado pasa de **24-28 a 25-30 meses**.
- Se introducen dos dependencias externas jóvenes (`instant-acme`,
  `hickory-resolver`) en dominios donde los bugs se pagan caro: un certificado
  que no renueva tumba el sitio. Mitigación: tests de contrato del trait
  `DnsProvider`, pinning explícito en el workspace y vigilancia activa de los
  upstreams.
- La validación build-time de templates implica parsing parcial de HTML para
  cruzar las `vars` declaradas con las del template. Es más costosa de
  implementar de lo que sugiere su brevedad en este documento.

**Neutrales:**

- El hito v0.5 BETA sigue al final de la Fase 5. La Fase 4.5 NO lo adelanta.
- Los bloques `mail` y `domain` del DSL pertenecen a v0.7, no v0.5/v0.6 ya
  entregados.

---

## Alternativas consideradas

**A. Meter ambas capacidades dentro de la Fase 4.**
Sobrecarga una fase ya densa (cinco módulos estándar) y retrasa el hito v0.5
BETA. Descartada.

**B. Posponer ambas capacidades a la Fase 5 dentro de `ag-cloud`.**
Confunde el alcance de `ag-cloud`, convirtiéndolo en una plataforma de
correo + DNS + despliegue. Rompe la separación de responsabilidades por crate.
Descartada.

**C. Implementar `ag-mail` como wrapper exclusivo de un proveedor (p. ej.
Resend).**
Más rápido, pero rompe la regla de interoperabilidad: convierte al ecosistema
en cliente cautivo de un proveedor. Descartada en favor del patrón
abstracción + adapters.

**D. Implementar `ag-domains` solo como cliente Cloudflare.**
Mismo argumento que (C). El trait `DnsProvider` mantiene la opción de adapters
futuros (Route53, Namecheap, etc.) sin tocar la superficie pública.

**E (elegida). Fase 4.5 aditiva con dos crates clasificados.**
Mantiene la disciplina de fases bloqueantes, no modifica el alcance de las
fases ya completadas, respeta la regla de interoperabilidad, y prepara el
terreno para la Fase 5 sin inflarla.

---

## Impacto documental

Este ADR exige aplicar cambios coordinados a (orden de aplicación):

1. `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` — nueva Fase 4.5 entre 4 y 5;
   resumen de fases; duración total 25-30 meses; tabla del DSL realineada.
2. `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` — mapa de crates (17);
   reglas de dependencia con la regla anti-ciclo `ag-auth → ag-mail`;
   integración `ag-cloud → ag-domains`; subsecciones de módulos.
3. `docs/master/ANTI-GRAVITAL-Blueprint-v4.0.pdf` — re-export como v4.1 con
   patches a portada, §5.1, §5.2, §7.3, §8.8/8.9, §10, §19.1, §19.6.5 y
   Apéndice A.
4. Derivados (verbatim del maestro): `docs/roadmap/STATUS.md`,
   `docs/roadmap/fase-04-5-ag-mail-y-ag-domains.md`,
   `docs/roadmap/preambulo.md`, `docs/roadmap/calendar.md`,
   `docs/roadmap/README.md`, `docs/architecture/05-ecosistema-modulos.md`,
   `docs/architecture/08-modulos-batteries-included.md`,
   `docs/architecture/10-despliegue-ag-cloud.md`,
   `docs/architecture/19-glosario.md`, `docs/dsl/versionado.md`,
   `docs/modules/README.md`, `docs/modules/ag-mail/`,
   `docs/modules/ag-domains/`.
5. `README.md` (bilingüe ES/EN) — fila 4.5 en Calendario, secciones "Qué es" /
   "What it is" y "Qué no es" / "What it is not".
6. `CHANGELOG.md` — bump documental v4.0 → v4.1.
7. `CLAUDE.md` — contexto de los dos crates en §14.

La etapa de implementación técnica se aborda en una rama posterior `fase-4.5`
y queda bloqueada hasta que este conjunto documental esté mergeado.

---

## Referencias

- `CLAUDE.md` reglas 0, 1, 4, 5, 14, 15, 22, 26, 27 — gobernanza documental,
  crates, dependencias, sincronización código ↔ documentación.
- ADR-0001 — monorepo workspace (15 crates iniciales, superseded en parte por
  este ADR respecto al conteo: 15 → 17).
- ADR-0004 — descomposición verbatim de los maestros (esta ADR respeta esa
  regla: el contenido canónico vive en los maestros, los derivados se
  regeneran).
- ADR-0006 — `ag-auth` WebAuthn sin `webauthn-rs` (precedente de decisión
  sobre dependencias y licencias; aplica vigilancia equivalente a
  `instant-acme` y `hickory-resolver`).
- Blueprint §3.3 — regla de interoperabilidad.
