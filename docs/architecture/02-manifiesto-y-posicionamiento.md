# Capitulo 2. Manifiesto y posicionamiento

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 2
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [01-resumen-ejecutivo.md](./01-resumen-ejecutivo.md)
> Siguiente: [03-alcance-y-limites.md](./03-alcance-y-limites.md)

## 2. Manifesto and positioning

For the last twenty years, the software industry has accepted a trade-off that is no longer necessary: choosing between performance and productivity. The world's most widely adopted frameworks thrived by solving only one of the two extremes. Spring Boot and .NET imposed enterprise structure at the price of heavy virtual machines and multi-second startups. Django and FastAPI made it possible for small teams to build APIs in hours, at the price of the GIL and an interpreter that places an invisible ceiling on performance. Node.js brought isomorphic development at the price of a single-threaded event loop and an npm ecosystem chronically vulnerable to supply chain attacks.

None of these frameworks is bad. They all solve real problems. But they were all designed in an era prior to three converging phenomena that change the calculation: the production maturity of the Rust ecosystem, the arrival of AI agents capable of writing quality code at superhuman speeds, and the industry's disenchantment with the operational complexity of multi-language stacks.

Anti-Gravital is built on the premise that systems performance and developer productivity are not opposing forces, but design problems. A correctly designed framework can offer both simultaneously without hidden trade-offs.

The name describes the thesis. Current frameworks have *gravity*: they tie you to interpreters, virtual machines, external runtimes, and abstraction layers that charge in latency, memory, and operational complexity. Anti-Gravital breaks with that gravity from the foundations: no JVM, no GC, no interpreter, no external runtime. Only native machine code, memory safety guaranteed at compile time, and massive concurrency without garbage collection cost.

**Explicit positioning.** Anti-Gravital does not position itself against any language or any framework. It positions itself as the modern unified backend and runtime layer for applications that need three things simultaneously: systems performance, high-level framework productivity, and operational deployment simplicity. The target audience is not the team that already has a Spring stack running in production and has no pain — it is the team that is starting a new project, or that has reached the structural limits of Python/Node.js, or that needs to reduce the memory footprint of its service fleet.

The adoption strategy is built on interoperability and gradual migration, not on the aggressive replacement of existing stacks. The official importers (OpenAPI, Prisma, Sequelize, Django ORM, FastAPI/Pydantic models) are first-class citizens, not an afterthought.

---

