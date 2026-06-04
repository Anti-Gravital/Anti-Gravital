# Capitulo 10. Subsistema de despliegue (ag-cloud + ag-domains)

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 10
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [09-plugins-wasi.md](./09-plugins-wasi.md)
> Siguiente: [11-ai-knowledge-graph.md](./11-ai-knowledge-graph.md)

## 10. Subsistema de despliegue (`ag-cloud` + `ag-domains`)

Una de las correcciones estructurales más importantes derivadas del análisis crítico es que `ag-cloud` no es un competidor de Terraform ni de Kubernetes. Su rango objetivo es el mismo que cubren Railway, Fly.io, Render y Coolify: simplificar el despliegue de aplicaciones backend a entornos típicos sin obligar al equipo a operar infraestructura completa. Desde la Fase 4.5 (`ADR-0007`), `ag-cloud` coopera con `ag-domains` para resolver dominio, TLS y registros de correo dentro del propio flujo de `ag deploy`, sin reemplazar a los proveedores dominantes (Let's Encrypt, Cloudflare) y sin convertirse en un panel de hosting.

### 10.1 Filosofía de `ag-cloud`

El operador típico de un proyecto Anti-Gravital, especialmente en sus primeros años de vida, no necesita ni quiere operar un clúster Kubernetes. Necesita levantar su API en un VPS, conectarla a una base de datos, ponerla detrás de TLS, y olvidarse. `ag-cloud` resuelve este caso.

Para casos más complejos (despliegues multi-región, alta disponibilidad, gestión de secrets centralizada, políticas IAM, infraestructura compartida entre múltiples aplicaciones), `ag-cloud` no es la herramienta correcta y el proyecto debe declararlo abiertamente: usa Terraform, Pulumi o Helm.

### 10.2 El archivo `deploy.ag`

El subsistema de despliegue se controla con un archivo declarativo `deploy.ag` separado del schema del proyecto:

```yaml
app:
  name: payments-api
  domain: api.example.com

runtime:
  replicas: 3
  port: 8080
  health_check: /health
  resources:
    cpu: 1
    memory: 512MB

database:
  type: postgres
  version: "16"
  size: 20GB
  backup_schedule: "daily"

cache:
  type: redis
  version: "7"
  size: 1GB

storage:
  type: s3
  bucket: payments-api-uploads

secrets:
  source: vault
  path: secret/payments-api

observability:
  metrics: prometheus
  traces: tempo
  logs: loki

deployment:
  target: docker-compose      # opciones: docker-compose, fly, railway, k8s
  strategy: rolling
  max_surge: 1
  max_unavailable: 0
```

### 10.3 Targets de despliegue soportados

`ag-cloud` soporta cuatro targets de despliegue, cada uno con un nivel de abstracción distinto.

El target **docker-compose** genera un `docker-compose.yml` completo con servicios, redes, volúmenes, healthchecks, secrets cargados de archivos `.env` o de un secret manager, reverse proxy (Caddy por defecto) con TLS automático vía Let's Encrypt, y backup scripts para la base de datos. Es el target recomendado para self-hosting en un VPS único.

El target **fly** genera un `fly.toml` y ejecuta los comandos `flyctl` necesarios para desplegar a Fly.io. Es el target recomendado para edge computing global con bajo overhead operacional.

El target **railway** genera la configuración para Railway y triggerea el despliegue vía su API. Es el target recomendado para equipos que prefieren PaaS sin operación.

El target **k8s** genera manifests Kubernetes estándar (Deployment, Service, Ingress, ConfigMap, Secret, HorizontalPodAutoscaler) con valores razonables. Para configuraciones avanzadas, este target es un punto de partida que el equipo customiza, no una solución completa.

### 10.4 Pipeline de despliegue

El comando `ag deploy` ejecuta un pipeline estandarizado: validación del schema, compilación con `cargo build --release --target <target>`, construcción de la imagen Docker desde una base `scratch` o `distroless`, ejecución de tests de smoke, push de la imagen a un registro, aplicación de migraciones de base de datos en orden, despliegue rolling con healthchecks, y verificación post-despliegue.

### 10.5 Reverse proxy y TLS

Para despliegues docker-compose, `ag-cloud` configura Caddy como reverse proxy con TLS automático. Caddy obtiene y renueva certificados Let's Encrypt sin configuración explícita. Para entornos donde TLS lo gestiona un balanceador externo (Cloudflare, AWS ALB), Caddy se desactiva.

### 10.6 Integración con `ag-domains`

Introducida por `ADR-0007`. Cuando un proyecto declara dominios en su contrato
`.ag` (bloque `domain` del DSL v0.7), `ag deploy` resuelve un flujo de seis
pasos coordinado con `ag-domains`:

1. **Validar control del dominio.** Inserción de un registro TXT de verificación
   vía el `DnsProvider` configurado y confirmación de su presencia.
2. **Configurar DNS de aplicación.** `upsert_record` para apuntar el dominio al
   target del despliegue (CNAME al host de Fly/Railway, o registros A/AAAA en
   docker-compose).
3. **Emitir o renovar TLS.** Cliente ACME contra Let's Encrypt (DNS-01
   preferido). El certificado se almacena en filesystem o `ag-storage`.
4. **Asociar el dominio al target.** Configurar el reverse proxy (Caddy en
   docker-compose, fly cert en Fly, etc.) para servir el dominio con el
   certificado emitido.
5. **Materializar SPF/DKIM/DMARC** que `ag-mail` haya declarado en sus
   `MailSender::dns_requirements`.
6. **Verificar propagación** contra múltiples resolvers públicos antes de
   marcar el despliegue como exitoso.

`ag-cloud` **NO depende rígidamente** de `ag-domains` en todos los targets:
si el proyecto no declara dominios, el flujo se omite. Si el target es uno
donde el TLS lo gestiona un balanceador externo (Cloudflare en frente,
AWS ALB), `ag-cloud` puede saltarse el paso 3 sin afectar el resto del
pipeline. Esta flexibilidad es lo que mantiene a `ag-domains` como módulo
opcional, no como pieza obligatoria del runtime.

---

