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

## 4. Especialistas internos vs. externos

- **Interno** (`squad_role: specialist`): agente del catálogo con la tecnología en su `knowledge`; se invoca por `handoff` con `correlation_id`.
- **Externo**: proveedor o agente fuera del repositorio; siempre entra por contrato y con recepción formal.

La lista de tecnologías con especialista interno se mantiene en `docs/kaizen/org/CATALOGO-AGENTES.md` (columna de conocimiento). Si no aparece ahí, se subcontrata con contrato.
