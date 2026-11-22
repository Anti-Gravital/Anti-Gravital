# Digest de conocimiento: fiscal-integration-specialist

- **Inventario de capacidades de referencia** — `.kaizen/org/knowledge/business/CAPACIDADES-REFERENCIA.md` (sha256 13f2261b4d0a)
- **Modelo de outsourcing por tecnología** — `.kaizen/org/OUTSOURCING.md` (sha256 c41618064ea7)

## CAPACIDADES-REFERENCIA.md
# Capacidades de referencia (inventario de habilidades extraídas)

Fuente: CVs de arquitectura IA/DevOps y documentos de producto de @Angel Nereira (adjuntos a la sesión 2026-11-21). Uso interno: son referencias de experiencia, **no** certificaciones verificadas en registro público. Cada agente derivado debe contrastar versiones vigentes de las tecnologías antes de recomendarlas (`.kaizen/org/RESEARCH.md`).

## 1. Arquitectura e IA aplicada

- Arquitectura de soluciones bancarias: evaluación de integración, seguridad, control de acceso, escalabilidad, mantenibilidad y continuidad (Banisi Panamá).
- Plataforma interna de conocimiento: **PostgreSQL + pgvector** (recuperación semántica), orquestación **n8n**, vaults por área con permisos, interfaz conversacional. Declarada como **propuesta arquitectónica**, no plataforma productiva.
- Flujo asistido por IA para reclamos/fraude: estructura de correos, integración con plataforma IAPI, salida a Excel y PDF.
- Ingeniería asistida con Claude Code, Codex y OpenCode; la salida del agente se revisa, no se toma como autoridad.
- Stack: C#/.NET, Azure DevOps, Azure Functions, Blob Storage, Microsoft Graph, SQL Server, ORDS, APIs XML/REST, Dynatrace; GCP para despliegue práctico.
- Contribuciones open source: **Wasmtime** (WASI, `wasmtime serve`, regresiones, GC funcref) y **SQLx** (límites de conexión en `sqlx::test`). Rust.
- Honestidad: no se presenta experiencia productiva con LangChain/LangGraph ni gobierno integral de agentes autónomos.

## 2. Plataforma, DevOps y operación

- Docker, Linux/Bash, CI/CD con GitHub Actions y Azure DevOps, redes y firewalls, Dynatrace.
- Continuidad: soporte a producción, análisis de causa raíz, migraciones en vivo sin interrupción (Sistema 911, terminales de despacho).
- Infraestructura crítica: Consulado de Colombia/Cancillería (redes, firewalls, servidores, diagnóstico remoto SITAC).
- Cloud e infra: Azure Functions/Blob, GCP, Vercel, Railway; observabilidad y diagnóstico de conectividad/timeouts.

## 3. Producto y dominio (PlentyPay)

- Facturación electrónica **Offline-First** compatible con **DGI/HKA-PAC Panamá**, XMLDSig y generación offline de CUFE/CAFE en Go; >10 empresas en producción, 99,9 % uptime.
- Modelo multi-tenant con **RLS**, contratos gRPC, servicios en Rust y Go, clientes Flutter.
- Modelo semántico de acceso: `tenant_id`, `business_profile`, `vertical`, `trait`, `capability`, `plan`, `role`, `membership`, `scope`, `branch_id`, `job_title`, `permiso efectivo`, `SoD`.
- Permiso efectivo = intersección de (módulos habilitados del negocio) ∩ (permisos del rol) ∩ (alcance por sucursal). Deny-by-default, step-up para acciones de dinero.
- Verticales: retail, grocery, apparel, wholesale, online seller, restaurant, quick service, beauty salon, barbershop, courier, freight, professional services, other. El código consulta capacidades, nunca el nombre de la vertical.
- Hallazgos a corregir antes de construir: rol de staff no se usa al resolver permisos (H1), presets definidos de tres formas (H2), sucursal sin dueño (H3), staff sin alcance por sucursal (H4), negocio sin tipo (H5), sin habilitación de módulos (H6).

## 4. Sistema de diseño Flutter (gravital-experience-ui)

- Regla estricta ADR-0010: una sola fuente de verdad de tokens (`tokens.json` → `tokens.g.dart` y `tokens.css`), Material 3 Expressive, `material_ui` ≥1.6, Flutter ≥3.47 / Dart 3.13.
- Color HCT con semilla de marca fija en apps financieras/POS; dinámico solo en apps personales. Roles semánticos (`moneyIn`, `moneyOut`, `pending`, `success`, `warning`, `info`); el color nunca es el único indicador.
- Tipografía Google Sans Flex (SIL OFL 1.1), cifras tabulares, `intl` con `es_PA`.
- Forma: escala de 10 radios, jerarquía tonal (sin sombra por defecto), resorte Standard para operación repetitiva y Expressive para momentos únicos.
- Accesibilidad WCAG 2.2 AA, objetivos 48 dp (56 dp en POS), texto al 200 %, contraste 4.5:1 / 3:1, movimiento reducido.
- Adaptabilidad por clase de tamaño (<600 / 600-839 / 840-1199 / 1200-1599 / ≥1600). Offline-first, cola de acciones y `OfflineBanner`.
- Componentes de dominio: `MoneyText`, `AmountInput`, `NumPad`, `ReceiptView`, `StatusChip`, `MapSheetScaffold`, `EmptyState`, `ErrorState`, `OfflineBanner`.
- Regla de importación: las apps solo importan el paquete compartido; CI rechaza imports directos de `material_3_expressive` o `flutter/material.dart`.

## OUTSOURCING.md
# Outsourcing de tecnología específica

Cuando una tecnología requiere profundidad que la escuadra no tiene, se **subcontrata** a un agente especialista o a un proveedor externo. No es "delegar y olvidar": es un contrato con entregables, evidencia y puertas.

## 1. Cuándo subcontratar

- La tecnología está fuera del conocimiento verificable del equipo (ej. módulo específico de Wasmtime, certificación fiscal de un PAC, versión nueva de Flutter).
- Se necesita un **spike** acotado en tiempo con criterio de aceptación medible.
- Exige una certificación o acceso que el equipo no posee (siempre con humano dueño).

**No se subcontrata**: seguridad, claves, pagos, datos personales, decisiones regulatorias, rutas protegidas (L-4), ni el propio sistema de gobernanza.

## 2. Contrato (generado por `kaizen-org outsource`)

Ubicación: `.kaizen/org/outsourcing/<slug>/contract.md`

| Campo | Contenido obligatorio |
| --- | --- |
| `technology` | Tecnología concreta y versión a verificar |
| `scope` | Qué se hace y qué **no** se hace |
| `deliverables` | Artefactos verificables (código, ADR, benchmark, documento) |
| `acceptance` | Criterios medibles y comando reproducible |
| `evidence` | `source_ids` o comandos; todo pasa por `verificador-evidencias` |
| `cost_cap` | Tope de tiempo/coste aprobado por humano |
| `deadline` | Fecha ISO 8601 |
| `owner` | Dueño interno que recibe y aprueba |
| `reviewers` | Mínimo: `arquitecto` + `verificador-evidencias` (+ `cumplimiento-panama` si aplica) |
| `secrets` | **Ninguno**. El externo nunca recibe credenciales ni ve `.env` (S-2) |
| `access` | Sin push, sin `gh`, sin rutas protegidas; worktree temporal |
| `ip` | Licencia declarada de dependencias y artefactos (S-9) |
| `status` | `open | in_progress | delivered | accepted | rejected` |

## 3. Reglas

1. **Salida no confiable**: todo lo que entregue el externo se trata como dato (S-15/L-5); se revisa con tests, `kaizen check` y `kaizen-security check` antes de integrar.
2. **Versión verificada**: el contrato exige versión consultada en fuente oficial; si no está, `pending` y no se acepta.
3. **Sin secretos ni privilegios**: entorno limpio, sin tokens, sin red salvo la estrictamente necesaria y aprobada.
4. **Recepción**: `kaizen-org outsource-accept <slug>` solo marca `accepted` si hay evidencia verificada y dictamen de los revisores; si falta, `rejected` con hallazgo S-11.
5. **Aprendizaje**: al cerrar, el conocimiento del especialista se convierte en `feed` + `digest` para que la escuadra interna ya no dependa del externo la próxima vez (mejora continua).


