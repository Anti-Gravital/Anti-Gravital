# Capitulo 4. Analisis del estado del arte

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 4
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [03-alcance-y-limites.md](./03-alcance-y-limites.md)
> Siguiente: [05-ecosistema-modulos.md](./05-ecosistema-modulos.md)

## 4. Analysis of the state of the art

This section documents the competitive context in technical terms, without adversarial rhetoric. Each framework analyzed solves a real set of problems; the analysis identifies the structural limitations that Anti-Gravital intends to address.

### 4.1 Spring Boot and the JVM ecosystem

Spring Boot dominates enterprise Java and Kotlin development with two decades of mature ecosystem. Its structural weaknesses derive from the JVM: a base memory consumption of 256-512 MB before serving the first request, startup times of 6-8 seconds incompatible with serverless computing, and configuration verbosity. GraalVM Native Image partially mitigates startup and memory, but introduces its own limitations (limited reflection, incomplete library compatibility, long compilation times). The fundamental trade-off — a managed runtime with GC — remains.

### 4.2 ASP.NET Core and .NET

Technically one of the fastest managed frameworks on the market, with modern and expressive C#. The CLR with GC keeps measurable pauses at p99 under sustained load. The technical direction of the ecosystem is unilaterally Microsoft's. Memory safety is not guaranteed by the compiler; race condition bugs and null reference exceptions are possible. Adoption outside the Microsoft ecosystem remains limited for cultural rather than technical reasons.

### 4.3 Django and FastAPI

Django maintains the best prototyping experience in the Python world, with a rich ecosystem for administration, authentication, and templates. FastAPI raised the DX standard for Python APIs with Pydantic types, automatic OpenAPI generation, and native async support. Both share the structural ceiling of CPython: the Global Interpreter Lock prevents real CPU-bound concurrency within a process, which forces scaling with multiple processes (Gunicorn, Uvicorn workers) multiplying memory consumption. Django's async support remains partial; many libraries in the ecosystem remain synchronous.

### 4.4 Node.js, Express, and NestJS

Node.js brought JavaScript to the server and the npm ecosystem is the broadest in the industry. V8's single-threaded event loop is optimal for concurrent I/O but degrades with any CPU-bound work. The npm supply chain is chronically vulnerable: the average transitive dependency of a modern Node.js project exceeds 200 libraries, and incidents of compromised packages are recurrent. TypeScript adds type safety in development, but at runtime it remains JavaScript.

### 4.5 Next.js and the JS fullstack frameworks

Next.js represents the frontend/backend convergence in JavaScript. Server Components and Server Actions reduce the boilerplate of internal APIs. The structural weaknesses are inherited from Node.js: serverless cold starts, de facto coupling with Vercel, inadequacy for persistent WebSockets, shared state, and long-running processing. Next.js is an excellent presentation layer; it is not a robust backend.

### 4.6 Axum, Actix-Web, Rocket (Rust)

The current Rust frameworks are technically excellent in performance (consistently in the TechEmpower top 10) but offer what the community calls a *low-level* experience: the developer builds authentication, the data layer, observability, client generation, and the migration system from scratch. Anti-Gravital is built on Axum, Tokio, and Tower as internal dependencies — it does not compete with them, but packages them into a complete framework experience with DSL, CLI, and opinionated modules.

### 4.7 Conclusion of the analysis

There is a real market space: a dominant enterprise-grade Rust framework does not yet exist. Spring Boot pays the historical cost of the JVM. Node.js has structural event loop limits. Python has concurrency problems. Go sacrifices the type system. Rust has runtime and performance, but lacks a complete framework experience. Anti-Gravital intends to fill that gap.

---

