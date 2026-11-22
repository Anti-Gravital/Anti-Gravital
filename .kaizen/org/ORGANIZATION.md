# Organización de agentes

La comunidad no es solo ingeniería: es una organización con dirección, negocio, tecnología, operaciones y conocimiento. Cada puesto es un archivo en `.kaizen/org/prompts/<id>.md`, reproducible y especializable (ver `AGENT_SCHEMA.md`). El inventario vivo se consulta con `scripts/kaizen-org list`.

## Departamentos y puestos iniciales

| Departamento | Puestos | Autoridad |
| --- | --- | --- |
| **gerencia** | `gerente` (COO), `director-producto` | `gerente`: veto `resource`, administra procesos y escala; `director-producto`: prioriza valor |
| **producto-negocio** | `analista-negocios` (Analista Oficial de Negocios), `marketing` (visualización/marketing insights), `finanzas-panamas`, `product-owner` | `analista-negocios`: veto `scope`; el resto propone con evidencia |
| **cumplimiento** | `cumplimiento-panama` (normativa panameña + estándares internacionales, investigación en internet e implementación ISO) | veto `compliance` (ADR-0002) |
| **tecnologia** | los 17 roles de `.kaizen/swarm/ROSTER.md`: `architect`, `product`, `innovation`, `dev1-core`…`dev5-glue`, `systems`, `lowlevel`, `ai`, `data-expert`, `infra`, `qa`, `security`, `dx`, `finops` | `architect` veto `architecture`; `security` veto `security`; `qa` veto `qa` |
| **operaciones** | `soporte`, `runbooks` | proponen correcciones y documentan reversibilidad |
| **conocimiento** | `knowledge-curator`, `researcher`, `capacidades-negocio` (tecnología, finanzas y mercadeo aplicados) | alimentan y sanean la base de conocimiento; investigan en internet; no deciden por otros |

**Total: 42 agentes activos.** Los 10 puestos `squad_role: specialist` (`gcp-cloud-architect`, `ai-rag-engineer`, `dotnet-banking-integrator`, `platform-engineer`, `rust-systems-specialist`, `fiscal-integration-specialist`, `flutter-m3-design-specialist`, `business-profiles-architect`, `identity-access-specialist`, `vertical-domain-specialist`) amplían el catálogo con tecnología específica según `CAPACIDADES-REFERENCIA.md` (ADR-0004). Cuando una tecnología queda fuera de esos contratos se aplica `.kaizen/org/OUTSOURCING.md`. Los puestos con `research-web` en `tools` (`cumplimiento-panama`, `capacidades-negocio`, `researcher`) investigan en internet según `.kaizen/org/RESEARCH.md`: fuentes oficiales primero, contraste mínimo de dos fuentes, fecha de corte y expediente en `.kaizen/org/research/`. Sus resultados alimentan `.kaizen/org/knowledge/` y se regeneran con `kaizen-org digest`.

## Principios de la organización

1. **Puestos, no personalidades**: cada rol tiene propósito, entradas, salidas, KPI y límites. Se puede copiar y refinar.
2. **Proactividad con evidencia**: los disparadores (`triggers`) generan propuestas en `decisions/inbox/`, nunca acciones sin control (L-3).
3. **Consenso registrado**: consulta → discusión (2 turnos) → mayoría cualificada sin veto → decisión con fecha de revisión (`CONSENSUS.md`).
4. **Memoria en el repo**: inventario, decisiones, knowledge y diario son archivos versionados (L-2).
5. **Ratchet también para agentes**: `save` sube versión; una definición no empeora sin ADR (K-5).

## Especialización por copia

Ejemplos previstos: copiar `marketing` a `marketing-visual` (métricas de visualización y campaña), copiar `finanzas-panamas` a `finanzas-panamas-iva` (focus en IVA y normativa local), copiar `lowlevel` a `lowlevel-wasm` (bindings y zero-copy en WASI). El original queda intacto; la copia hereda reglas y puede añadir knowledge específico.
