# Roster canónico de la comunidad

> El documento de diseño decía "16 roles", pero la tabla enumeraba 17. Se fija **17** como roster canónico: los 5 devs + los roles de dirección, especialización y operación que completan el ciclo de ingeniería y negocio. Esta corrección queda registrada en ADR-0001.

| ID | Rol | Dominio | Round(s) | Autoridad | Herramienta |
| --- | --- | --- | --- | --- | --- |
| `architect` | Arquitecto de Software | Dirección y Producto | R1, R3, finalize | **Veto** (arquitectura/contratos) + árbitro | `KSW_ROLE_AGENT_ARCHITECT` |
| `product` | Product & Scope Analyst | Dirección y Producto | R1, finalize | Árbitro de alcance | default |
| `innovation` | Analista de Innovación y Nuevas Tech | Dirección y Producto | R1 | Propuesta, sin veto | default |
| `dev1-core` | Dev 1 — Core / Dominio | Desarrollo | R2 | Implementa invariantes | `KSW_ROLE_AGENT_DEV` |
| `dev2-api` | Dev 2 — API / Servicios / Integraciones | Desarrollo | R2 | Implementa contratos | `KSW_ROLE_AGENT_DEV` |
| `dev3-ui` | Dev 3 — Frontend / UI / Cliente | Desarrollo | R2 | Implementa cliente/UX | `KSW_ROLE_AGENT_DEV` |
| `dev4-data` | Dev 4 — Datos / Persistencia / Migraciones | Desarrollo | R2 | Implementa esquema/migraciones reversibles | `KSW_ROLE_AGENT_DEV` |
| `dev5-glue` | Dev 5 — Integración / Refactor / Glue | Desarrollo | R2, finalize | Integrador: resuelve colisiones, diffs acotados | `KSW_ROLE_AGENT_DEV` |
| `systems` | Ingeniero de Sistemas / Especialista | Especialistas Técnicos | R1 | Flujos transversales, concurrencia | default |
| `lowlevel` | Experto en Software de Bajo Nivel | Especialistas Técnicos | R1 | Memoria, latencia, syscalls | default |
| `ai` | Experto en IA | Especialistas Técnicos | R1 | Prompts, evaluación, guardrails, coste | default |
| `data-expert` | Experto en Datos | Especialistas Técnicos | R1 | Schemas, pipelines, privacidad | default |
| `infra` | Experto en Infraestructura | Especialistas Técnicos | R1 | Contenedores, CI/CD, reversibilidad | default |
| `qa` | Equipo de QA | Calidad, DX y Gobernanza | R2, R3 | **Veto funcional** (tests/regresión) | `KSW_ROLE_AGENT_QA` |
| `security` | Seguridad y Auditoría | Calidad, DX y Gobernanza | R1, R3 | **Veto** (secretos, S-10, privilegios) | `KSW_ROLE_AGENT_SECURITY` |
| `dx` | DX Experiences 2026–2027 Expert | Calidad, DX y Gobernanza | R1, R3 | Ergonomía, CLI, feedback, agents/skills | default |
| `finops` | FinOps / Release & Docs | Calidad, DX y Gobernanza | R3 | Coste, despliegue/rollback, K-7 | default |

## Regla de parallelismo

`KSW_MAX_PARALLEL=6`. Si el número de roles en una ronda excede el tope, el orquestador los reparte en oleadas respetando dependencias: ningún rol de R2 arranca sin que `contracts/` contenga las firmas que su prompt declara como entrada, y ninguna firma se considera válida sin `ack` de su destinatario.
