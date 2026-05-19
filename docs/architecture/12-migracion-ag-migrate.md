# Capitulo 12. Framework de migracion (ag-migrate): importadores

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 12
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [11-ai-knowledge-graph.md](./11-ai-knowledge-graph.md)
> Siguiente: [13-mobile-ag-mobile.md](./13-mobile-ag-mobile.md)

## 12. Framework de migración (`ag-migrate`): importadores

La adopción real de cualquier framework backend exitoso ha pasado siempre por la posibilidad de migrar desde el incumbente. La industria odia las reescrituras. El módulo `ag-migrate` no es un afterthought; es un ciudadano de primera clase del proyecto.

### 12.1 Importadores soportados

`ag-migrate` ofrece importadores oficiales para los frameworks más adoptados del mercado.

El importador **OpenAPI** consume cualquier spec OpenAPI 3.0 o 3.1 y produce un `schema.ag` con modelos, endpoints, errores y validaciones. Es el importador más genérico y sirve para migrar desde cualquier servicio que documente una OpenAPI, independiente del lenguaje en que esté escrito.

El importador **Prisma** consume un archivo `schema.prisma` y traduce modelos, relaciones y migraciones a Anti-Gravital. Cubre la migración desde aplicaciones TypeScript que usan Prisma como ORM.

El importador **Django** lee modelos Django (definidos como clases Python) y produce los modelos Anti-Gravital equivalentes. Incluye traducción de relaciones, managers, signals y migraciones.

El importador **FastAPI** consume aplicaciones FastAPI examinando los routers y los modelos Pydantic. Produce endpoints y modelos Anti-Gravital. Es probablemente el caso de migración más natural por la similitud filosófica entre FastAPI y Anti-Gravital.

El importador **Sequelize** lee modelos de aplicaciones Node.js que usan Sequelize ORM. Cubre el caso Express + Sequelize, muy común en el mercado.

El importador **GraphQL** consume un schema GraphQL SDL y produce su equivalente en Anti-Gravital.

### 12.2 Limitaciones honestas

Los importadores cubren la traducción del contrato (modelos, endpoints, validaciones), no la lógica de negocio. La lógica de los handlers debe escribirse manualmente o con asistencia de un agente de IA. Esto se documenta claramente para evitar expectativas erróneas.

### 12.3 Guías oficiales de migración

Para cada framework soportado se publica una guía oficial en la documentación: estrategia recomendada (big bang vs strangler fig), patrones para coexistencia durante la transición (proxy reverso que reparte tráfico entre el sistema legacy y el nuevo), testing comparativo, y casos de estudio reales cuando estén disponibles.

---

