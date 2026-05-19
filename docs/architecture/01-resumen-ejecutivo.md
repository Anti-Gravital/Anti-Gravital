# Capitulo 1. Resumen ejecutivo

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 1
> Indice: [docs/architecture/README.md](./README.md)
> Siguiente: [02-manifiesto-y-posicionamiento.md](./02-manifiesto-y-posicionamiento.md)

## 1. Resumen ejecutivo

Anti-Gravital es un ecosistema de software libre para construir aplicaciones backend de alto rendimiento, escrito en Rust puro, con tres propiedades fundamentales que lo distinguen del resto del mercado de frameworks web actuales.

La primera es la ausencia total de runtime externo: el resultado de un proyecto Anti-Gravital es un binario estático autocontenido que se ejecuta sobre el sistema operativo sin intérprete ni máquina virtual de por medio. Esto elimina la JVM, CPython, Node.js y CLR del path de ejecución y, con ellos, las pausas de recolección de basura, los segundos de arranque en frío y los cientos de megabytes de memoria base que esos runtimes consumen antes de procesar la primera petición.

La segunda es el enfoque schema-first apoyado en un lenguaje de definición de dominio llamado Anti-DSL, archivos con extensión `.ag`. Un único archivo describe modelos, endpoints, políticas, validaciones, errores y relaciones; el compilador del DSL deriva de allí el código Rust, los clientes tipados para frontend y aplicaciones móviles, la documentación OpenAPI, y las migraciones de base de datos. El contrato es una sola fuente de verdad, y la deriva de esquema (schema drift) deja de ser una clase de problema posible por construcción.

La tercera es una arquitectura modular pensada como un ecosistema, no como un framework monolítico. El núcleo es deliberadamente pequeño y se compone con módulos publicados de forma independiente. Cada módulo (`ag-auth`, `ag-data`, `ag-realtime`, `ag-cache`, `ag-storage`, `ag-observe`, `ag-cloud`, `ag-ai`, `ag-mobile`, `ag-migrate`) tiene un dominio propio, versionado propio, y puede usarse de forma aislada en cualquier proyecto Rust. Esta separación hace al ecosistema sostenible a escala de comunidad y elimina el síndrome del "framework que intenta resolverlo todo".

El proyecto nace desde Panamá, con documentación bilingüe español/inglés desde el día cero. El primer foco de adopción es Latinoamérica; el horizonte es global. La licencia Apache 2.0 garantiza que no existirá nunca una versión Enterprise cerrada ni features reservadas para clientes pagos: la totalidad del ecosistema es y será código abierto.

Este documento describe en detalle cada componente, cada decisión arquitectónica, y los compromisos técnicos que sustentan el proyecto. El documento complementario *Hoja de Ruta y Puertas de Verificación* define la secuencia temporal de entregables y los criterios de bloqueo entre fases.

---

