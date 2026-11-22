# ADR-0004: especialistas por tecnología y outsourcing con contrato

Estado: aceptado
Fecha: 2026-11-21
Aprobado por: Angel Nereira

## Contexto
La comunidad cubre dominios amplios, pero hay tecnologías que requieren profundidad específica (GCP, RAG/pgvector, integración bancaria .NET, plataforma/DevOps, Rust/WASI, integración fiscal DGI/HKA, Flutter M3E, modelos de negocio multi-tenant, identidad y acceso, verticales comerciales). Los CV y documentos de referencia aportan competencias verificables que deben convertirse en agentes, y cuando la tecnología quede fuera del equipo debe existir una forma segura de subcontratar.

## Decisión
1. Crear 10 agentes especialistas derivados de las referencias: `gcp-cloud-architect`, `ai-rag-engineer`, `dotnet-banking-integrator`, `platform-engineer`, `rust-systems-specialist`, `fiscal-integration-specialist`, `flutter-m3-design-specialist`, `business-profiles-architect`, `identity-access-specialist`, `vertical-domain-specialist`.
2. Publicar `.kaizen/org/knowledge/business/CAPACIDADES-REFERENCIA.md` como inventario de habilidades, con la salvedad de que no son certificaciones registradas y exigen verificar versión vigente.
3. Añadir `.kaizen/org/OUTSOURCING.md` y los comandos `kaizen-org outsource` / `outsource-accept`: contrato con alcance, entregables, aceptación reproducible, evidencia, sin secretos, sin push y recepción formal por revisores.
4. Toda salida externa se trata como dato no confiable (S-15/L-5) y requiere `verificador-evidencias` antes de integrarse.

## Alternativas consideradas
- Un agente genérico "tecnología": descartado; diluye responsabilidad y evidencia.
- Subcontratar sin contrato: descartado; no hay criterio de aceptación ni trazabilidad.

## Consecuencias
- Positivas: profundidad por tecnología, contrato reproducible, learning interno vía `feed`/`digest`.
- Negativas: más agentes en el catálogo (42); se compensa con el roster por departamento y el flag `squad_role: specialist`.

## Límites
Las referencias son experiencia declarada en documentos internos, no certificaciones públicas. Ningún especialista puede decidir en seguridad, pago, dato personal o norma sin humano (ADR-0002/0003).

## Cómo se verificará
- `kaizen-org check` y `catalog` con 42 agentes.
- `bash tests/run.sh` cubre creación de contrato y rechazo por falta de `evidence.md`.
