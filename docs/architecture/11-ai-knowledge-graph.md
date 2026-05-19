# Capitulo 11. Integracion con IA (ag-ai) y el Knowledge Graph

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 11
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [10-despliegue-ag-cloud.md](./10-despliegue-ag-cloud.md)
> Siguiente: [12-migracion-ag-migrate.md](./12-migracion-ag-migrate.md)

## 11. Integración con Inteligencia Artificial (`ag-ai`) y el Knowledge Graph

El módulo `ag-ai` es probablemente el diferenciador más significativo del proyecto en el contexto 2026, donde los agentes de IA son colaboradores cotidianos en el desarrollo de software. El módulo tiene dos componentes complementarios.

### 11.1 El Anti-DSL como contrato para agentes

El primer componente no es código: es la decisión arquitectónica de que el `schema.ag` sirva como contrato perfecto para agentes de IA. Un agente que recibe un endpoint declarado en `.ag` tiene exactamente lo que necesita para generar un handler correcto: tipos precisos, errores definidos, políticas de acceso, validaciones, eventos a emitir. Y, crítica diferencia con cualquier otro framework, el compilador Rust verifica después que el código generado por el agente sea type-safe antes de que llegue a producción.

Esto convierte el flujo de desarrollo en una colaboración estructurada. El ingeniero diseña el schema. El agente implementa los handlers. El compilador actúa como segundo revisor automático que rechaza cualquier desincronización. El operador supervisa y aprueba.

### 11.2 El Knowledge Graph

El segundo componente es el grafo de conocimiento del proyecto. `ag-ai` mantiene un grafo dirigido que indexa todas las entidades del proyecto y sus relaciones: modelos, endpoints, eventos, políticas, dependencias entre handlers, llamadas a bases de datos, llamadas a servicios externos, configuraciones, plugins instalados.

El grafo se reconstruye automáticamente en cada `ag generate` y se serializa a `.ag/knowledge-graph.json`. Desde este insumo se producen automáticamente: documentación arquitectónica en Markdown, diagramas C4 (Context, Container, Component) en formato Mermaid, registros de decisión arquitectónica (ADRs) sugeridos, listas de dependencias críticas, mapas de impacto para cambios propuestos, y un dashboard interactivo en el dev server.

### 11.3 Capacidades AI asistidas

El módulo expone tres capacidades adicionales accesibles desde la CLI:

`ag ai suggest-schema` analiza un dominio descrito en lenguaje natural y propone un primer borrador de `schema.ag`. El ingeniero refina desde allí.

`ag ai review-migration` analiza una migración SQL propuesta y reporta riesgos: locks, downtime, pérdida de datos, queries lentas durante la transición. Sugiere alternativas con migración en dos pasos cuando es necesario.

`ag ai analyze-architecture` produce un reporte sobre el grafo de conocimiento del proyecto: identificación de hotspots (modelos con demasiadas dependencias), endpoints sin tests, eventos emitidos pero sin consumidores, dead code, antipatrones.

### 11.4 Conexión con proveedores de modelos

El módulo no embebe ningún modelo de lenguaje. Se conecta a proveedores externos vía API: Anthropic Claude, OpenAI, modelos locales servidos por Ollama o vLLM. La conexión es configurable y el operador puede elegir o auto-hospedar. Para entornos sensibles, se soporta el modo offline donde las funciones AI están desactivadas pero el resto del framework funciona normalmente.

---

