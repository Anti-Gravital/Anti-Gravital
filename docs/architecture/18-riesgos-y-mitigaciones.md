# Capitulo 18. Analisis de riesgos y mitigaciones

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 18
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [17-gobernanza-open-source.md](./17-gobernanza-open-source.md)
> Siguiente: [19-glosario.md](./19-glosario.md)

## 18. Análisis de riesgos y mitigaciones

Esta sección documenta los riesgos reales del proyecto y las mitigaciones planeadas. Es deliberadamente honesta; un proyecto que no enumera sus riesgos no merece confianza.

### 18.1 Riesgo: complejidad del compilador del DSL

El compilador del DSL es un proyecto de varios años por sí solo. La mitigación es la implementación incremental por versiones del DSL descrita en la sección 7. La versión 0.1 cubre solo modelos básicos y es entregable en dos meses. Cada versión añade un subconjunto bien definido. La versión 1.0 estable del DSL es el hito de mayor riesgo del proyecto y se planifica para el final del cronograma.

### 18.2 Riesgo: curva de aprendizaje de Rust

Rust tiene una curva de aprendizaje real. La mitigación es triple. Primero, el DSL genera el 80% del scaffolding, de modo que los handlers que el desarrollador escribe son Rust simple: unos pocos `await`, acceso a estado compartido, retornar un `Result`. Segundo, la documentación incluye una guía "Rust para desarrolladores de Python/Node.js" con los conceptos mínimos necesarios. Tercero, el asistente AI integrado puede generar handlers que el desarrollador supervisa.

### 18.3 Riesgo: competencia con grandes players

Spring, .NET, Express y FastAPI tienen ecosistemas de décadas. Anti-Gravital no puede competir frontalmente con ellos en breadth. La mitigación es enfocarse en nichos donde los incumbentes tienen debilidades estructurales: aplicaciones de alta carga, servicios edge, backends para Flutter, backends para aplicaciones IA con streaming.

### 18.4 Riesgo: bus factor

El proyecto inicial tiene un bus factor preocupantemente bajo (un mantenedor). La mitigación es activa: documentación interna completa desde el día uno, incorporación de contribuidores externos desde la fase 1, y transición a comité técnico antes del 1.0.

### 18.5 Riesgo: cambios en el ecosistema Rust

El ecosistema Rust sigue evolucionando rápidamente. Axum, Tokio y sqlx pueden hacer cambios breaking en versiones futuras. La mitigación es pinneo conservador de versiones, tests de integración exhaustivos contra cada nueva versión de las dependencias core, y participación activa en sus comunidades para anticipar cambios.

### 18.6 Riesgo: fragmentación de la comunidad

Si la comunidad de Anti-Gravital fragmenta (por ejemplo, surgen forks competidores con features divergentes), el ecosistema se debilita. La mitigación es un proceso RFC abierto que da voz real a la comunidad, releases predecibles, y una hoja de ruta pública.

### 18.7 Riesgo: vulnerabilidades de seguridad post-lanzamiento

Aunque Rust elimina muchas categorías de vulnerabilidades, no elimina las lógicas (autorización rota, leaks de información, races a nivel de aplicación). La mitigación es la auditoría externa antes del 1.0, el programa de divulgación responsable, fuzzing continuo, y CI con análisis estático (clippy, cargo-audit, cargo-deny).

---

