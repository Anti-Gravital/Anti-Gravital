# Capitulo 12. Framework de migracion (ag-migrate): importadores

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 12
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [11-ai-knowledge-graph.md](./11-ai-knowledge-graph.md)
> Siguiente: [13-mobile-ag-mobile.md](./13-mobile-ag-mobile.md)

## 12. Migration framework (`ag-migrate`): importers

The real adoption of any successful backend framework has always passed through the possibility of migrating from the incumbent. The industry hates rewrites. The `ag-migrate` module is not an afterthought; it is a first-class citizen of the project.

### 12.1 Supported importers

`ag-migrate` offers official importers for the most adopted frameworks on the market.

The **OpenAPI** importer consumes any OpenAPI 3.0 or 3.1 spec and produces a `schema.ag` with models, endpoints, errors, and validations. It is the most generic importer and serves to migrate from any service that documents an OpenAPI, regardless of the language it is written in.

The **Prisma** importer consumes a `schema.prisma` file and translates models, relationships, and migrations to Anti-Gravital. It covers migration from TypeScript applications that use Prisma as an ORM.

The **Django** importer reads Django models (defined as Python classes) and produces the equivalent Anti-Gravital models. It includes translation of relationships, managers, signals, and migrations.

The **FastAPI** importer consumes FastAPI applications by examining the routers and the Pydantic models. It produces Anti-Gravital endpoints and models. It is probably the most natural migration case due to the philosophical similarity between FastAPI and Anti-Gravital.

The **Sequelize** importer reads models from Node.js applications that use the Sequelize ORM. It covers the Express + Sequelize case, very common in the market.

The **GraphQL** importer consumes a GraphQL SDL schema and produces its equivalent in Anti-Gravital.

### 12.2 Honest limitations

The importers cover the translation of the contract (models, endpoints, validations), not the business logic. The logic of the handlers must be written manually or with the assistance of an AI agent. This is documented clearly to avoid erroneous expectations.

### 12.3 Official migration guides

For each supported framework, an official guide is published in the documentation: recommended strategy (big bang vs strangler fig), patterns for coexistence during the transition (reverse proxy that splits traffic between the legacy system and the new one), comparative testing, and real case studies when available.

---

