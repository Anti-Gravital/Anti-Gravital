# Catálogo consistente de agentes

Fuente única: `.kaizen/org/prompts/<id>.md`. Generado por `scripts/kaizen-org catalog`; no editar a mano.

Registro: `.kaizen/org/registry.json` | Fecha: 2026-11-22 | Total: 43 agentes

## Resumen por departamento

| Departamento | Agentes | Activos | Veto |
| --- | ---: | ---: | --- |
| conocimiento | 4 | 4 | none |
| cumplimiento | 3 | 3 | compliance, none |
| gerencia | 3 | 3 | none, resource |
| operaciones | 2 | 2 | none |
| producto-negocio | 6 | 6 | none, scope |
| tecnologia | 25 | 25 | architecture, none, qa, security |

## Inventario

| ID | Nombre | Departamento | Estado | Proactividad | Autonomía | Veto | Versión | Herramientas | Disparadores | Conocimiento | KPI |
| --- | --- | --- | --- | --- | --- | --- | ---: | --- | --- | ---: | --- |
| `ai` | Experto en IA | tecnologia | active | high | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Evaluaciones reproducibles y coste medido] |
| `ai-rag-engineer` | Ingeniero de IA y RAG | tecnologia | active | high | scoped | none | 1 | [read, write-docs, research-web, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 2 | [evaluaciones con métricas y coste por consulta] |
| `analista-negocios` | Analista Oficial de Negocios | producto-negocio | active | high | scoped | scope | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Criterios de aceptacion completos y trazables] |
| `architect` | Arquitecto de Software | tecnologia | active | high | scoped | architecture | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Contratos documentados y sin deuda arquitectural] |
| `business-profiles-architect` | Arquitecto de Perfiles de Negocio | producto-negocio | active | high | scoped | scope | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [verticales sin condicionales y capacidades documentadas] |
| `capacidades-negocio` | Especialista en Capacidades Tecnológicas, Financieras y de Mercadeo | conocimiento | active | high | propose | none | 1 | [read, write-docs, research-web, run-checks, report] | [issue.label:capacidades, research.required:*, knowledge.updated:*, schedule:weekly] | 8 | [recomendaciones con supuestos, baseline y verificación, 0 cifras sin fuente, conocimiento revalidado trimestralmente] |
| `cumplimiento-panama` | Especialista en Cumplimiento y Normativa (Panamá + Estándares Internacionales) | cumplimiento | active | high | scoped | compliance | 1 | [read, write-docs, research-web, run-checks, report] | [issue.label:cumplimiento, research.required:*, knowledge.updated:*, schedule:weekly, regulatory.change:*] | 8 | [mapa normativo vigente con fuente oficial y fecha de corte, 0 afirmaciones legales sin cita, 100% de propuestas con control y dueño humano] |
| `data-expert` | Experto en Datos | tecnologia | active | medium | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Telemetria saneada y conforme a L-12] |
| `dev1-core` | Dev Core y Dominio | tecnologia | active | medium | scoped | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Pruebas de dominio en verde] |
| `dev2-api` | Dev API e Integraciones | tecnologia | active | medium | scoped | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Contratos publicados antes de codificar] |
| `dev3-ui` | Dev Frontend y Cliente | tecnologia | active | medium | scoped | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Flujos verificables con pruebas] |
| `dev4-data` | Dev Datos y Migraciones | tecnologia | active | medium | scoped | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Migraciones reversibles probadas] |
| `dev5-glue` | Dev Integracion y Glue | tecnologia | active | high | scoped | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Integraciones sin conflictos de propiedad] |
| `director-producto` | Director de Producto | gerencia | active | high | scoped | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Propuestas priorizadas con RICE y evidencia] |
| `dotnet-banking-integrator` | Integrador Bancario .NET | tecnologia | active | medium | scoped | none | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [contratos tipados y diagnóstico de incidentes] |
| `dx` | Experto en DX | tecnologia | active | high | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Tiempo de feedback medido] |
| `facilitador-equipo` | Facilitador de Equipo | gerencia | active | high | scoped | resource | 1 | [read, write-docs, run-checks, report] | [project.created, schedule:daily, knowledge.updated:*, handoff.pending:*] | 5 | [standups diarios sin tareas sin dueño, acuses pendientes < 1 por dia, proyectos avanzan sin saltarse puertas] |
| `finanzas-panamas` | Analista Financiero Panama | producto-negocio | active | medium | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Analisis reproducible con supuestos explicitos] |
| `finops` | FinOps y Release | tecnologia | active | medium | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Coste medido y K-7 cumplida] |
| `fiscal-integration-specialist` | Especialista en Integración Fiscal DGI/HKA | cumplimiento | active | high | scoped | compliance | 1 | [read, write-docs, research-web, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 2 | [documentos válidos ante DGI y sin secretos] |
| `flutter-m3-design-specialist` | Especialista en Diseño Flutter M3E | tecnologia | active | high | scoped | none | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [cero literales de diseño y pruebas WCAG AA] |
| `gcp-cloud-architect` | Arquitecto Cloud GCP | tecnologia | active | high | scoped | architecture | 1 | [read, write-docs, research-web, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [despliegues con IaC y presupuesto medido] |
| `gerente` | Gerente de Operaciones (COO) | gerencia | active | high | scoped | resource | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Decisiones administradas en plazo y sin escalados sin evidencia] |
| `identity-access-specialist` | Especialista en Identidad y Acceso | tecnologia | active | high | scoped | architecture | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [cero bypass de RLS y step-up en dinero] |
| `infra` | Experto en Infraestructura | tecnologia | active | medium | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Despliegues reversibles y sin secretos] |
| `innovation` | Analista de Innovacion | tecnologia | active | medium | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Propuestas con evidencia y coste] |
| `knowledge-curator` | Curador de Conocimiento | conocimiento | active | high | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Digest actualizado sin secretos ni datos personales] |
| `lowlevel` | Experto en Bajo Nivel | tecnologia | active | low | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Benchmarks con numeros reales] |
| `marketing` | Especialista en Marketing y Visualizacion | producto-negocio | active | high | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Insights con numeros reales y hipotesis verificable] |
| `monitor-gaceta` | Monitor de Gaceta Oficial y Normativa Panameña | cumplimiento | active | high | propose | none | 1 | [read, write-docs, research-web, report] | [schedule:daily, regulatory.change:*, research.required:*] | 5 | [cambios normativos detectados con URL oficial y fecha, 0 normas citadas sin Gaceta, expedientes de investigacion por cambio] |
| `platform-engineer` | Ingeniero de Plataforma y DevOps | tecnologia | active | high | scoped | none | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [despliegues reversibles y MTTR medido] |
| `product` | Analista de Producto y Alcance | tecnologia | active | medium | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Criterios verificables antes de implementar] |
| `product-owner` | Product Owner | producto-negocio | active | high | scoped | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Backlog sin items ambiguos y cierre con evidencia] |
| `qa` | Equipo de QA | tecnologia | active | high | scoped | qa | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Toda correccion con prueba que falla primero] |
| `rea-specialist` | Especialista en Ingeniería Inversa y Depuración Profunda (REA) | tecnologia | active | high | scoped | none | 1 | [read, write-docs, research-web, run-checks, report, rea-mcp] | [research.required:*, issue.label:depuracion, reverse_engineering.authorized:*, knowledge.updated:*] | 4 | [hallazgos con evidencia reproducible, 0 análisis sin autorización escrita, 0 secretos en reportes] |
| `researcher` | Investigador de Tecnologia | conocimiento | active | medium | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Experimentos con numeros reales y decision registrada] |
| `runbooks` | Responsable de Runbooks | operaciones | active | medium | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Runbooks probados y al dia] |
| `rust-systems-specialist` | Especialista en Rust y Sistemas | tecnologia | active | medium | propose | none | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [benchmarks reproducibles y cero regresiones] |
| `security` | Seguridad y Auditoria | tecnologia | active | high | propose | security | 1 | [read, audit, report] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [0 secretos y 0 supresiones sin issue] |
| `soporte` | Especialista de Soporte y Respuesta | operaciones | active | high | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Reproducciones documentadas y respuesta medida] |
| `systems` | Ingeniero de Sistemas | tecnologia | active | medium | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Diseno con bajo acoplamiento] |
| `verificador-evidencias` | Verificador de Evidencias | conocimiento | active | high | scoped | none | 1 | [read, write-docs, research-web, run-checks, report] | [decision.proposed, source.stale, schedule:weekly, regulatory.change:*] | 5 | [0 decisiones con source_ids no verificados, fuentes revalidadas cada 90 dias, conflictos escalados a humano] |
| `vertical-domain-specialist` | Especialista en Verticales Comerciales | producto-negocio | active | medium | propose | none | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [recorridos por vertical verificados con el negocio] |

## Fichas por departamento

### conocimiento

- **`capacidades-negocio`** — Especialista en Capacidades Tecnológicas, Financieras y de Mercadeo. Propósito: Ampliar y mantener el conocimiento especializado de la organización en tecnología, finanzas y mercadeo; investigar fuentes actuales y convertirlas en capacidades aplicables con métricas y planes de implementación.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`knowledge-curator`** — Curador de Conocimiento. Propósito: Alimenta, sanea e indexa documentacion para ampliar el conocimiento de los agentes.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`researcher`** — Investigador de Tecnologia. Propósito: Investiga hipotesis con experimentos reproducibles y propone ADR.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`verificador-evidencias`** — Verificador de Evidencias. Propósito: Ser la puerta de verdad: ninguna decisión se da por tomada sin fuentes verificadas, hashes/fecha de corte y registro. Bloquea automáticamente lo que no cumple (REG-EV-01..07).. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].

### cumplimiento

- **`cumplimiento-panama`** — Especialista en Cumplimiento y Normativa (Panamá + Estándares Internacionales). Propósito: Traducir la normativa panameña aplicable y los estándares internacionales en controles concretos, verificables y implementables; investigar su vigencia de forma proactiva y bloquear cambios que creen riesgo legal o de cumplimiento sin revisión humana.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`fiscal-integration-specialist`** — Especialista en Integración Fiscal DGI/HKA. Propósito: Implementa emisión electrónica offline con XMLDSig, CUFE/CAFE y SOAP HKA/DGI multi-tenant.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`monitor-gaceta`** — Monitor de Gaceta Oficial y Normativa Panameña. Propósito: Vigilar la Gaceta Oficial de Panamá y los órganos competentes para detectar leyes, decretos y resoluciones nuevas o modificadas, y avisar al equipo con evidencia verificable antes de que el cambio cause incumplimiento.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].

### gerencia

- **`director-producto`** — Director de Producto. Propósito: Prioriza valor y alinea negocio con tecnologia junto al analista de negocios.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`facilitador-equipo`** — Facilitador de Equipo. Propósito: Hacer que la comunidad funcione como un equipo: convoca, asigna, vigila acuses y leases, ordena turnos y escala cuando hace falta. No resuelve el fondo de los temas; resuelve la coordinación.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`gerente`** — Gerente de Operaciones (COO). Propósito: Administra la organizacion de agentes, asigna trabajo, marca plazos y escala a humanos.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].

### operaciones

- **`runbooks`** — Responsable de Runbooks. Propósito: Mantiene runbooks ejecutables y reversibles con rollback verificado.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`soporte`** — Especialista de Soporte y Respuesta. Propósito: Clasifica incidencias, reproduce con datos sinteticos y propone correcciones.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].

### producto-negocio

- **`analista-negocios`** — Analista Oficial de Negocios. Propósito: Convierte objetivos de negocio en criterios verificables, procesos y reglas de decision.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`business-profiles-architect`** — Arquitecto de Perfiles de Negocio. Propósito: Modela verticales, rasgos, capacidades y módulos sin código por tipo de negocio.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`finanzas-panamas`** — Analista Financiero Panama. Propósito: Analiza coste, presupuesto y contexto financiero panameno con fuentes citadas.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`marketing`** — Especialista en Marketing y Visualizacion. Propósito: Analiza senales de marketing y visualizacion de campanas y producto.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`product-owner`** — Product Owner. Propósito: Gestiona el backlog y los criterios de salida; mantiene el tablero veraz.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`vertical-domain-specialist`** — Especialista en Verticales Comerciales. Propósito: Traduce necesidades de restaurante, salón, barbería, mensajería y carga a capacidades del producto.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].

### tecnologia

- **`ai`** — Experto en IA. Propósito: Disena prompts, guardrails y evaluacion de agentes.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`ai-rag-engineer`** — Ingeniero de IA y RAG. Propósito: Diseña recuperación semántica con PostgreSQL y pgvector, orquestación n8n y evaluaciones reproducibles.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`architect`** — Arquitecto de Software. Propósito: Define limites, contratos y ADR; arbitra disputas tecnicas.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`data-expert`** — Experto en Datos. Propósito: Define schemas, calidad de datos y privacidad.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`dev1-core`** — Dev Core y Dominio. Propósito: Implementa invariantes y modelos de negocio.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`dev2-api`** — Dev API e Integraciones. Propósito: Expone contratos, eventos y conectores.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`dev3-ui`** — Dev Frontend y Cliente. Propósito: Construye vistas, estados y accesibilidad.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`dev4-data`** — Dev Datos y Migraciones. Propósito: Disena esquemas y migraciones reversibles.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`dev5-glue`** — Dev Integracion y Glue. Propósito: Conecta piezas y mantiene diffs acotados.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`dotnet-banking-integrator`** — Integrador Bancario .NET. Propósito: Integra C#/.NET, SQL Server, ORDS y APIs XML/REST en entorno bancario con continuidad operativa.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`dx`** — Experto en DX. Propósito: Mejora CLI, feedback y ergonomia de desarrollo.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`finops`** — FinOps y Release. Propósito: Controla coste, despliegue/rollback y documentacion.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`flutter-m3-design-specialist`** — Especialista en Diseño Flutter M3E. Propósito: Aplica gravital-experience-ui con tokens, accesibilidad y offline-first; rechaza colores y radios literales.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`gcp-cloud-architect`** — Arquitecto Cloud GCP. Propósito: Diseña y revisa despliegues en Google Cloud con criterios de seguridad, coste y continuidad.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`identity-access-specialist`** — Especialista en Identidad y Acceso. Propósito: Resuelve el permiso efectivo como intersección de módulos habilitados, rol y alcance por sucursal.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`infra`** — Experto en Infraestructura. Propósito: Revisa contenedores, CI/CD y reversibilidad.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`innovation`** — Analista de Innovacion. Propósito: Evalua opciones emergentes sin dependencias riesgosas.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`lowlevel`** — Experto en Bajo Nivel. Propósito: Optimiza memoria, latencia y syscalls.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`platform-engineer`** — Ingeniero de Plataforma y DevOps. Propósito: Mantiene CI/CD, contenedores, Linux y observabilidad Dynatrace; ejecuta migraciones controladas.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`product`** — Analista de Producto y Alcance. Propósito: Valida criterios y alcance minimo del issue.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`qa`** — Equipo de QA. Propósito: Escribe pruebas de reproduccion, regresion e integracion.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`rea-specialist`** — Especialista en Ingeniería Inversa y Depuración Profunda (REA). Propósito: Profundizar en el análisis de sistemas —binarios, Electron/.NET, Android, firmware, tráfico y comportamiento en runtime— usando REA cuando el humano lo autoriza, para convertir hallazgos difíciles en evidencia verificable y cambios seguros.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`rust-systems-specialist`** — Especialista en Rust y Sistemas. Propósito: Trabaja en runtime WebAssembly, WASI, no_std, C FFI y audio de baja latencia con benchmarks.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`security`** — Seguridad y Auditoria. Propósito: Audita secretos, supresiones y privilegios; bloquea riesgos.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`systems`** — Ingeniero de Sistemas. Propósito: Resuelve concurrencia, resiliencia y flujos transversales.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].

## Reglas del catálogo

1. El catálogo se **genera**, nunca se edita a mano.
2. `id` único; campos obligatorios validados por `kaizen-org check`.
3. Copiar un agente (`kaizen-org copy`) crea una ficha nueva; el original no se modifica.
4. Cambiar autonomía o veto requiere ADR humano, no edición del catálogo.
5. `enable|disable` y `save` se reflejan aquí tras `catalog`.
