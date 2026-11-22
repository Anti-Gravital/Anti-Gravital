# Catálogo consistente de agentes

Fuente única: `.kaizen/org/prompts/<id>.md`. Generado por `scripts/kaizen-org catalog`; no editar a mano.

Registro: `.kaizen/org/registry.json` | Fecha: 2026-11-22 | Total: 25 agentes

## Resumen por departamento

| Departamento | Agentes | Activos | Veto |
| --- | ---: | ---: | --- |
| conocimiento | 3 | 3 | none |
| cumplimiento | 3 | 3 | compliance, none |
| gerencia | 2 | 2 | resource |
| producto-negocio | 5 | 5 | none, scope |
| tecnologia | 12 | 12 | architecture, none |

## Inventario

| ID | Nombre | Departamento | Estado | Proactividad | Autonomía | Veto | Versión | Herramientas | Disparadores | Conocimiento | KPI |
| --- | --- | --- | --- | --- | --- | --- | ---: | --- | --- | ---: | --- |
| `ai-rag-engineer` | Ingeniero de IA y RAG | tecnologia | active | high | scoped | none | 1 | [read, write-docs, research-web, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 2 | [evaluaciones con métricas y coste por consulta] |
| `analista-negocios` | Analista Oficial de Negocios | producto-negocio | active | high | scoped | scope | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Criterios de aceptacion completos y trazables] |
| `business-profiles-architect` | Arquitecto de Perfiles de Negocio | producto-negocio | active | high | scoped | scope | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [verticales sin condicionales y capacidades documentadas] |
| `capacidades-negocio` | Especialista en Capacidades Tecnológicas, Financieras y de Mercadeo | conocimiento | active | high | propose | none | 1 | [read, write-docs, research-web, run-checks, report] | [issue.label:capacidades, research.required:*, knowledge.updated:*, schedule:weekly] | 8 | [recomendaciones con supuestos, baseline y verificación, 0 cifras sin fuente, conocimiento revalidado trimestralmente] |
| `container-k8s-specialist` | Container and Kubernetes Specialist | tecnologia | active | high | scoped | none | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [despliegues reversibles y MTTR medido] |
| `cumplimiento-panama` | Especialista en Cumplimiento y Normativa (Panamá + Estándares Internacionales) | cumplimiento | active | high | scoped | compliance | 1 | [read, write-docs, research-web, run-checks, report] | [issue.label:cumplimiento, research.required:*, knowledge.updated:*, schedule:weekly, regulatory.change:*] | 8 | [mapa normativo vigente con fuente oficial y fecha de corte, 0 afirmaciones legales sin cita, 100% de propuestas con control y dueño humano] |
| `dotnet-banking-integrator` | Integrador Bancario .NET | tecnologia | active | medium | scoped | none | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [contratos tipados y diagnóstico de incidentes] |
| `facilitador-equipo` | Facilitador de Equipo | gerencia | active | high | scoped | resource | 1 | [read, write-docs, run-checks, report] | [project.created, schedule:daily, knowledge.updated:*, handoff.pending:*] | 5 | [standups diarios sin tareas sin dueño, acuses pendientes < 1 por dia, proyectos avanzan sin saltarse puertas] |
| `finanzas-panamas` | Analista Financiero Panama | producto-negocio | active | medium | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Analisis reproducible con supuestos explicitos] |
| `fiscal-integration-specialist` | Especialista en Integración Fiscal DGI/HKA | cumplimiento | active | high | scoped | compliance | 1 | [read, write-docs, research-web, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 2 | [documentos válidos ante DGI y sin secretos] |
| `flutter-m3-design-specialist` | Especialista en Diseño Flutter M3E | tecnologia | active | high | scoped | none | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [cero literales de diseño y pruebas WCAG AA] |
| `gcp-cloud-architect` | Arquitecto Cloud GCP | tecnologia | active | high | scoped | architecture | 1 | [read, write-docs, research-web, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [despliegues con IaC y presupuesto medido] |
| `gerente` | Gerente de Operaciones (COO) | gerencia | active | high | scoped | resource | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Decisiones administradas en plazo y sin escalados sin evidencia] |
| `identity-access-specialist` | Especialista en Identidad y Acceso | tecnologia | active | high | scoped | architecture | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [cero bypass de RLS y step-up en dinero] |
| `knowledge-curator` | Curador de Conocimiento | conocimiento | active | high | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Digest actualizado sin secretos ni datos personales] |
| `marketing` | Especialista en Marketing y Visualizacion | producto-negocio | active | high | propose | none | 1 | [read, write-docs, run-checks] | [issue.label:swarm:ready, knowledge.updated:*, schedule:weekly] | 5 | [Insights con numeros reales y hipotesis verificable] |
| `monitor-gaceta` | Monitor de Gaceta Oficial y Normativa Panameña | cumplimiento | active | high | propose | none | 1 | [read, write-docs, research-web, report] | [schedule:daily, regulatory.change:*, research.required:*] | 5 | [cambios normativos detectados con URL oficial y fecha, 0 normas citadas sin Gaceta, expedientes de investigacion por cambio] |
| `oss-observability-specialist` | Open Source Observability Specialist | tecnologia | active | high | scoped | none | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [despliegues reversibles y MTTR medido] |
| `platform-engineer` | Ingeniero de Plataforma y DevOps | tecnologia | active | high | scoped | none | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [despliegues reversibles y MTTR medido] |
| `rea-specialist` | Especialista en Ingeniería Inversa y Depuración Profunda (REA) | tecnologia | active | high | scoped | none | 1 | [read, write-docs, research-web, run-checks, report, rea-mcp] | [research.required:*, issue.label:depuracion, reverse_engineering.authorized:*, knowledge.updated:*] | 4 | [hallazgos con evidencia reproducible, 0 análisis sin autorización escrita, 0 secretos en reportes] |
| `rust-lowlevel-specialist` | Rust Low-Level Specialist | tecnologia | active | medium | propose | none | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [benchmarks reproducibles y cero regresiones] |
| `rust-perf-benchmarker` | Rust Performance and Benchmark Specialist | tecnologia | active | medium | propose | none | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [benchmarks reproducibles y cero regresiones] |
| `rust-systems-specialist` | Especialista en Rust y Sistemas | tecnologia | active | medium | propose | none | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [benchmarks reproducibles y cero regresiones] |
| `verificador-evidencias` | Verificador de Evidencias | conocimiento | active | high | scoped | none | 1 | [read, write-docs, research-web, run-checks, report] | [decision.proposed, source.stale, schedule:weekly, regulatory.change:*] | 5 | [0 decisiones con source_ids no verificados, fuentes revalidadas cada 90 dias, conflictos escalados a humano] |
| `vertical-domain-specialist` | Especialista en Verticales Comerciales | producto-negocio | active | medium | propose | none | 1 | [read, write-docs, run-checks, report] | [research.required:*, knowledge.updated:*, schedule:weekly, issue.label:especialista] | 1 | [recorridos por vertical verificados con el negocio] |

## Fichas por departamento

### conocimiento

- **`capacidades-negocio`** — Especialista en Capacidades Tecnológicas, Financieras y de Mercadeo. Propósito: Ampliar y mantener el conocimiento especializado de la organización en tecnología, finanzas y mercadeo; investigar fuentes actuales y convertirlas en capacidades aplicables con métricas y planes de implementación.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`knowledge-curator`** — Curador de Conocimiento. Propósito: Alimenta, sanea e indexa documentacion para ampliar el conocimiento de los agentes.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`verificador-evidencias`** — Verificador de Evidencias. Propósito: Ser la puerta de verdad: ninguna decisión se da por tomada sin fuentes verificadas, hashes/fecha de corte y registro. Bloquea automáticamente lo que no cumple (REG-EV-01..07).. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].

### cumplimiento

- **`cumplimiento-panama`** — Especialista en Cumplimiento y Normativa (Panamá + Estándares Internacionales). Propósito: Traducir la normativa panameña aplicable y los estándares internacionales en controles concretos, verificables y implementables; investigar su vigencia de forma proactiva y bloquear cambios que creen riesgo legal o de cumplimiento sin revisión humana.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`fiscal-integration-specialist`** — Especialista en Integración Fiscal DGI/HKA. Propósito: Implementa emisión electrónica offline con XMLDSig, CUFE/CAFE y SOAP HKA/DGI multi-tenant.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`monitor-gaceta`** — Monitor de Gaceta Oficial y Normativa Panameña. Propósito: Vigilar la Gaceta Oficial de Panamá y los órganos competentes para detectar leyes, decretos y resoluciones nuevas o modificadas, y avisar al equipo con evidencia verificable antes de que el cambio cause incumplimiento.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].

### gerencia

- **`facilitador-equipo`** — Facilitador de Equipo. Propósito: Hacer que la comunidad funcione como un equipo: convoca, asigna, vigila acuses y leases, ordena turnos y escala cuando hace falta. No resuelve el fondo de los temas; resuelve la coordinación.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`gerente`** — Gerente de Operaciones (COO). Propósito: Administra la organizacion de agentes, asigna trabajo, marca plazos y escala a humanos.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].

### producto-negocio

- **`analista-negocios`** — Analista Oficial de Negocios. Propósito: Convierte objetivos de negocio en criterios verificables, procesos y reglas de decision.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`business-profiles-architect`** — Arquitecto de Perfiles de Negocio. Propósito: Modela verticales, rasgos, capacidades y módulos sin código por tipo de negocio.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`finanzas-panamas`** — Analista Financiero Panama. Propósito: Analiza coste, presupuesto y contexto financiero panameno con fuentes citadas.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`marketing`** — Especialista en Marketing y Visualizacion. Propósito: Analiza senales de marketing y visualizacion de campanas y producto.. Reglas: [K-1..K-12, L-1..L-17, S-1..S-16].
- **`vertical-domain-specialist`** — Especialista en Verticales Comerciales. Propósito: Traduce necesidades de restaurante, salón, barbería, mensajería y carga a capacidades del producto.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].

### tecnologia

- **`ai-rag-engineer`** — Ingeniero de IA y RAG. Propósito: Diseña recuperación semántica con PostgreSQL y pgvector, orquestación n8n y evaluaciones reproducibles.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`container-k8s-specialist`** — Container and Kubernetes Specialist. Propósito: Mantiene CI/CD, contenedores, Linux y observabilidad Dynatrace; ejecuta migraciones controladas.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`dotnet-banking-integrator`** — Integrador Bancario .NET. Propósito: Integra C#/.NET, SQL Server, ORDS y APIs XML/REST en entorno bancario con continuidad operativa.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`flutter-m3-design-specialist`** — Especialista en Diseño Flutter M3E. Propósito: Aplica gravital-experience-ui con tokens, accesibilidad y offline-first; rechaza colores y radios literales.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`gcp-cloud-architect`** — Arquitecto Cloud GCP. Propósito: Diseña y revisa despliegues en Google Cloud con criterios de seguridad, coste y continuidad.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`identity-access-specialist`** — Especialista en Identidad y Acceso. Propósito: Resuelve el permiso efectivo como intersección de módulos habilitados, rol y alcance por sucursal.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`oss-observability-specialist`** — Open Source Observability Specialist. Propósito: Mantiene CI/CD, contenedores, Linux y observabilidad Dynatrace; ejecuta migraciones controladas.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`platform-engineer`** — Ingeniero de Plataforma y DevOps. Propósito: Mantiene CI/CD, contenedores, Linux y observabilidad Dynatrace; ejecuta migraciones controladas.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`rea-specialist`** — Especialista en Ingeniería Inversa y Depuración Profunda (REA). Propósito: Profundizar en el análisis de sistemas —binarios, Electron/.NET, Android, firmware, tráfico y comportamiento en runtime— usando REA cuando el humano lo autoriza, para convertir hallazgos difíciles en evidencia verificable y cambios seguros.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`rust-lowlevel-specialist`** — Rust Low-Level Specialist. Propósito: Trabaja en runtime WebAssembly, WASI, no_std, C FFI y audio de baja latencia con benchmarks.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`rust-perf-benchmarker`** — Rust Performance and Benchmark Specialist. Propósito: Trabaja en runtime WebAssembly, WASI, no_std, C FFI y audio de baja latencia con benchmarks.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].
- **`rust-systems-specialist`** — Especialista en Rust y Sistemas. Propósito: Trabaja en runtime WebAssembly, WASI, no_std, C FFI y audio de baja latencia con benchmarks.. Reglas: [K-1..K-14, L-1..L-17, S-1..S-16].

## Reglas del catálogo

1. El catálogo se **genera**, nunca se edita a mano.
2. `id` único; campos obligatorios validados por `kaizen-org check`.
3. Copiar un agente (`kaizen-org copy`) crea una ficha nueva; el original no se modifica.
4. Cambiar autonomía o veto requiere ADR humano, no edición del catálogo.
5. `enable|disable` y `save` se reflejan aquí tras `catalog`.
