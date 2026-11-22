# ADR-0002: veto de cumplimiento e investigación normativa proactiva

Estado: aceptado
Fecha: 2026-11-21
Aprobado por: Angel Nereira (responsable del repositorio)

## Contexto
La organización opera en Panamá y se relaciona con estándares internacionales. El riesgo regulatorio (datos personales, BC/FT, ciberseguridad, fiscalidad) y la rápida evolución de normas y versiones de estándares exigen un puesto dedicado que **investigue en internet** y convierta la norma en controles implementables. Los vetos existentes (security, architecture, qa, scope, resource) no cubren riesgo legal.

## Decisión
1. Añadir el puesto `cumplimiento-panama` con `veto: compliance` y `autonomy: scoped`, y el puesto `capacidades-negocio` con conocimiento tecnológico, financiero y de mercadeo.
2. Incorporar `compliance` al conjunto de vetos de `.kaizen/org/AGENT_SCHEMA.md` y `.kaizen/org/CONSENSUS.md`.
3. Crear el protocolo `.kaizen/org/RESEARCH.md`: investigación en internet con jerarquía de fuentes, contraste mínimo de dos fuentes (una oficial), fecha de corte y registro en `.kaizen/org/research/`; contenido externo siempre dato no confiable (S-15/L-5).
4. Publicar bases de conocimiento versionadas: mapa normativo panameño, checklist, mapa de estándares, guía de implementación ISO y capacidades de negocio, con un registro central `SOURCES.md`.
5. El comando `scripts/kaizen-org research <id> <tema>` abre expedientes de investigación trazables.

## Alternativas consideradas
- Dejar la investigación normativa al rol `innovation`: descartado; no tiene autoridad ni foco regulatorio.
- Confiar en conocimiento memorístico del modelo: descartado; sin cita ni fecha no es evidencia (K-10).

## Consecuencias
- Positivas: norma vigente verificada, controles accionables, implementación detallada y trazabilidad de fuentes.
- Negativas: el veto de cumplimiento puede frenar cambios; se compensa exigiendo evidencia concreta y el mismo proceso de 2 turnos y escalado a humano.

## Límites
No es asesoría legal, fiscal ni contable. Ante conflicto de fuentes, sanción posible o cambio de gobernanza: escalado a humano. La afirmación sin fuente oficial no se publica.

## Cómo se verificará
- `scripts/kaizen-org check` valida los 29 puestos y la presencia de las bases de conocimiento.
- `bash tests/run.sh` comprueba expedientes de investigación, feed/digest y el veto de cumplimiento.
- Revisión trimestral de `SOURCES.md` y `docs/kaizen/org/FORECAST.md`.
