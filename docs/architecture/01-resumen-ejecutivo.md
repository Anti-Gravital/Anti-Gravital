# Capitulo 1. Resumen ejecutivo

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 1
> Indice: [docs/architecture/README.md](./README.md)
> Siguiente: [02-manifiesto-y-posicionamiento.md](./02-manifiesto-y-posicionamiento.md)

## 1. Executive summary

Anti-Gravital is a free software ecosystem for building high-performance backend applications, written in pure Rust, with three fundamental properties that distinguish it from the rest of the current web framework market.

The first is the total absence of an external runtime: the result of an Anti-Gravital project is a self-contained static binary that runs on the operating system without an interpreter or virtual machine in between. This eliminates the JVM, CPython, Node.js, and the CLR from the execution path and, with them, the garbage collection pauses, the cold-start seconds, and the hundreds of megabytes of base memory that those runtimes consume before processing the first request.

The second is the schema-first approach supported by a domain definition language called Anti-DSL, files with the `.ag` extension. A single file describes models, endpoints, policies, validations, errors, and relationships; the DSL compiler derives from there the Rust code, the typed clients for frontend and mobile applications, the OpenAPI documentation, and the database migrations. The contract is a single source of truth, and schema drift ceases to be a possible class of problem by construction.

The third is a modular architecture conceived as an ecosystem, not as a monolithic framework. The core is deliberately small and is composed with independently published modules. Each module (`ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`, `ag-storage`, `ag-observe`, `ag-cloud`, `ag-ai`, `ag-mobile`, `ag-migrate`) has its own domain, its own versioning, and can be used in isolation in any Rust project. This separation makes the ecosystem sustainable at community scale and eliminates the "framework that tries to solve everything" syndrome.

The project is born in Panama, with bilingual Spanish/English documentation from day zero. The first adoption focus is Latin America; the horizon is global. The Apache 2.0 license guarantees that there will never be a closed Enterprise version or features reserved for paying customers: the entirety of the ecosystem is and will be open source.

This document describes in detail each component, each architectural decision, and the technical commitments that underpin the project. The complementary document *Roadmap and Verification Gates* defines the temporal sequence of deliverables and the blocking criteria between phases.

---

