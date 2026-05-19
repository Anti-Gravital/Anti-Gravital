# Fase 7 - ag-migrate importadores

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-06-ag-ai-knowledge-graph.md](./fase-06-ag-ai-knowledge-graph.md)
> Siguiente: [fase-08-ag-mobile.md](./fase-08-ag-mobile.md)

## Fase 7 — `ag-migrate` importadores

**Objetivo.** Construir los importadores de migración desde frameworks legacy. Es probablemente la fase con mayor impacto en adopción real.

### 7.1 Criterios de entrada

- [ ] Fase 6 completada.
- [ ] Investigación de muestras reales: al menos diez schemas/proyectos de cada framework objetivo recolectados como corpus de testing.

### 7.2 Entregables

- [ ] Crate `ag-migrate` con cinco importadores:
  - [ ] Importador OpenAPI 3.0 y 3.1.
  - [ ] Importador Prisma.
  - [ ] Importador Django.
  - [ ] Importador FastAPI.
  - [ ] Importador Sequelize.
  - [ ] Importador GraphQL SDL.
- [ ] Comando `ag migrate from <framework> <ruta>`.
- [ ] Guías oficiales de migración por framework con ejemplos completos.
- [ ] Estudio de caso documentado: migración real de una aplicación FastAPI mediana.

### 7.3 Criterios de salida (puerta antes de Fase 8)

- [ ] Cada importador tiene cobertura de tests ≥ 80% sobre el corpus de proyectos reales.
- [ ] La guía de migración FastAPI ha sido validada por al menos un equipo externo que migró su aplicación.
- [ ] Al menos 3 500 stars en el repositorio.

### 7.4 Riesgos de la fase

Los importadores cubren la traducción del contrato, no la lógica de negocio. El riesgo es generar expectativas exageradas. La mitigación es documentación honesta sobre lo que se importa y lo que no.

---
