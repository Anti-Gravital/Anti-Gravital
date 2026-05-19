# Capitulo 4. Analisis del estado del arte

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 4
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [03-alcance-y-limites.md](./03-alcance-y-limites.md)
> Siguiente: [05-ecosistema-modulos.md](./05-ecosistema-modulos.md)

## 4. Análisis del estado del arte

Esta sección documenta el contexto competitivo en términos técnicos, sin retórica adversarial. Cada framework analizado resuelve un conjunto real de problemas; el análisis identifica las limitaciones estructurales que Anti-Gravital pretende abordar.

### 4.1 Spring Boot y el ecosistema JVM

Spring Boot domina el desarrollo empresarial Java y Kotlin con dos décadas de ecosistema maduro. Sus debilidades estructurales derivan de la JVM: un consumo de memoria base de 256–512 MB antes de servir el primer request, tiempos de arranque de 6–8 segundos incompatibles con cómputo serverless, y verbosidad de configuración. GraalVM Native Image mitiga parcialmente el arranque y la memoria, pero introduce sus propias limitaciones (reflexión limitada, compatibilidad incompleta de librerías, tiempos de compilación largos). El compromiso fundamental — un runtime gestionado con GC — permanece.

### 4.2 ASP.NET Core y .NET

Técnicamente uno de los frameworks gestionados más rápidos del mercado, con C# moderno y expresivo. CLR con GC mantiene pausas medibles en p99 bajo carga sostenida. La dirección técnica del ecosistema es unilateral de Microsoft. La seguridad de memoria no está garantizada por el compilador; los bugs de race conditions y null reference exceptions son posibles. La adopción fuera del ecosistema Microsoft sigue siendo limitada por razones culturales más que técnicas.

### 4.3 Django y FastAPI

Django mantiene la mejor experiencia de prototipado del mundo Python, con un ecosistema rico para administración, autenticación y plantillas. FastAPI elevó el estándar de DX en APIs Python con tipos Pydantic, generación automática de OpenAPI y soporte async nativo. Ambos comparten el techo estructural de CPython: el Global Interpreter Lock impide concurrencia real CPU-bound dentro de un proceso, lo que obliga a escalar con múltiples procesos (Gunicorn, Uvicorn workers) multiplicando el consumo de memoria. El soporte async de Django sigue siendo parcial; muchas librerías del ecosistema permanecen sincrónicas.

### 4.4 Node.js, Express y NestJS

Node.js trajo JavaScript al servidor y el ecosistema npm es el más amplio de la industria. El event loop monohilo de V8 es óptimo para I/O concurrente pero se degrada con cualquier trabajo CPU-bound. La cadena de suministro npm es crónicamente vulnerable: la dependencia transitiva media de un proyecto Node.js moderno excede las 200 librerías, y los incidentes de paquetes comprometidos son recurrentes. TypeScript añade seguridad de tipos en desarrollo, pero en runtime sigue siendo JavaScript.

### 4.5 Next.js y los frameworks fullstack JS

Next.js representa la convergencia frontend/backend en JavaScript. Server Components y Server Actions reducen el boilerplate de APIs internas. Las debilidades estructurales son herencia de Node.js: cold starts en serverless, acoplamiento de facto con Vercel, inadecuación para WebSockets persistentes, estado compartido y procesamiento de larga duración. Next.js es una excelente capa de presentación; no es un backend robusto.

### 4.6 Axum, Actix-Web, Rocket (Rust)

Los frameworks Rust actuales son técnicamente excelentes en rendimiento (top 10 de TechEmpower de forma consistente) pero ofrecen lo que la comunidad llama una experiencia *low-level*: el desarrollador construye desde cero la autenticación, la capa de datos, la observabilidad, la generación de clientes y el sistema de migraciones. Anti-Gravital se construye sobre Axum, Tokio y Tower como dependencias internas — no compite con ellos, sino que los empaqueta en una experiencia de framework completo con DSL, CLI y módulos opinados.

### 4.7 Conclusión del análisis

Existe un espacio de mercado real: un framework Rust enterprise-grade dominante todavía no existe. Spring Boot paga el costo histórico de la JVM. Node.js tiene límites estructurales de event loop. Python tiene problemas de concurrencia. Go sacrifica el sistema de tipos. Rust tiene runtime y rendimiento, pero le falta una experiencia de framework completa. Anti-Gravital pretende llenar ese hueco.

---

