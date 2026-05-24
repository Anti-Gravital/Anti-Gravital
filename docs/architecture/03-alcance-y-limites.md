# Capitulo 3. Que es Anti-Gravital y que no es (alcance y limites)

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 3
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [02-manifiesto-y-posicionamiento.md](./02-manifiesto-y-posicionamiento.md)
> Siguiente: [04-estado-del-arte.md](./04-estado-del-arte.md)

## 3. Qué es Anti-Gravital y qué no es (alcance y límites)

La definición clara del alcance es probablemente la decisión arquitectónica más importante de este proyecto. Un framework que intenta ser todo termina siendo nada. Esta sección establece los límites explícitos del proyecto.

### 3.1 Qué es Anti-Gravital

Anti-Gravital es:

- Un **runtime backend Rust** de alto rendimiento para servicios HTTP, WebSocket y SSE.
- Un **lenguaje de definición de dominio** (Anti-DSL, archivos `.ag`) y su compilador.
- Una **CLI unificada** (`ag`) para creación, generación, desarrollo, build, despliegue y administración.
- Un **conjunto de módulos opcionales** publicados como crates Rust independientes (auth, data, realtime, cache, storage, observe, mail —estándar diferido—).
- Una **capa de gestión de dominios y TLS** (`ag-domains`, opcional infra) que integra DNS vía adapters, ACME para certificados, y SPF/DKIM/DMARC para correo transaccional.
- Un **sistema de plugins WASI** para extensibilidad multilenguaje aislada.
- Una **capa de orquestación de despliegue** simplificada al estilo Railway/Fly.io para casos comunes (no un reemplazo de Kubernetes).
- Un **generador de SDKs tipados** para TypeScript, Dart y otros lenguajes cliente.
- Un **conjunto de importadores de migración** desde frameworks legacy.
- Un **knowledge graph** auto-generado que mantiene la documentación arquitectónica sincronizada con el código.

### 3.2 Qué NO es Anti-Gravital

Esta lista es igualmente importante. Anti-Gravital **no** intenta ni intentará:

- **No reemplaza Kubernetes.** Para cargas que justifican Kubernetes, Anti-Gravital se despliega *sobre* Kubernetes como cualquier otro binario contenedorizado. `ag-cloud` cubre el rango Docker Compose hasta Fly.io. Cuando un equipo necesita orquestación a escala de cientos de nodos, usa Kubernetes y se acabó.
- **No reemplaza Flutter ni React Native.** Anti-Gravital no es un framework de UI multiplataforma. Es el backend nativo ideal *para* aplicaciones Flutter y React Native, con generación automática de SDKs cliente tipados, autenticación nativa, realtime, offline sync y streaming.
- **No reemplaza React, Vue, Svelte ni Next.js.** El módulo `ag-ui` ofrece SSR + HTMX para casos donde un stack JS completo es excesivo, pero no compite con frameworks frontend establecidos. Para aplicaciones SPA o SSR ricas, el patrón recomendado es Anti-Gravital como backend + Next.js (o equivalente) como frontend, comunicándose vía cliente TypeScript generado.
- **No reemplaza Docker.** Genera Dockerfiles. Se ejecuta en contenedores. No reinventa el formato OCI.
- **No reemplaza PostgreSQL, Redis, MinIO ni NATS.** Se integra con ellos como dependencias externas estándar.
- **No reemplaza Terraform ni Pulumi.** `ag-cloud` orquesta despliegues simples; para infraestructura compleja multi-cloud con políticas, IaC declarativa y módulos compartidos, Terraform sigue siendo la herramienta correcta. `ag-domains` (Fase 4.5) tampoco reemplaza Terraform: orquesta DNS y TLS para los dominios declarados en el `schema.ag` del proyecto, no gestiona zonas DNS arbitrarias ni infraestructura compartida.
- **No es un servidor de correo completo.** `ag-mail` (Fase 4.5) envía correo transaccional outbound (verificación, recuperación, magic links, alertas) vía SMTP nativo o adapters de proveedor (Resend, SES, Postmark). NO es un MTA, NO recibe correo (sin IMAP/POP), NO ofrece buzones, NO implementa antispam ni gestión de reputación de IP. Para inbound o un servidor de correo completo, usar Postfix, Stalwart u otro proyecto especializado.
- **No es un registrador de dominios.** `ag-domains` (Fase 4.5) consume el dominio que el operador ya compró (Namecheap, Cloudflare Registrar, etc.) y lo configura mediante un adapter (Cloudflare inicialmente). NO registra dominios, NO actúa como mercado de dominios.
- **No es un motor de juegos, ni un framework de cómputo científico, ni una alternativa a Unreal Engine, Unity, NumPy, PyTorch o TensorFlow.** Estos dominios tienen herramientas especializadas que Anti-Gravital no intenta replicar.

### 3.3 La regla de interoperabilidad

Cuando exista una herramienta dominante en un dominio adyacente, la estrategia es integrar, no reemplazar. Esta regla evita que el proyecto crezca en direcciones inmanejables y mantiene el alcance defendible.

---

