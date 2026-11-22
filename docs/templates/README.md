English | Espanol

---

# Template index

`ag new <name> --template <name>` scaffolds a project from a directory under
[`templates/`](../../templates/). The canonical decision table is the
[template gallery](GALLERY.md), which maps every wave-6 catalog name to its
use case, external requirements and example command.

- [Core](GALLERY.md#core): `rest`, `realtime`, `fullstack`, `layered`
- [Web](GALLERY.md#web): `static-site`, `marketing-site`, `blog`, `docs-site`, `dashboard`, `admin-panel`
- [API](GALLERY.md#api): `database-api`, `auth-service`, `multi-tenant-saas`, `webhook-gateway`, `file-upload-service`, `ecommerce-lite`
- [Realtime](GALLERY.md#realtime): `realtime-chat-v2`, `websocket-service`, `worker-service`, `scheduled-jobs`, `iot-ingest`, `edge-router`
- [Infrastructure](GALLERY.md#infrastructure): `mail-service`, `dns-manager`, `object-storage-api`, `monitoring-service`, `microservices-starter`, `cli-tool`
- [AI](GALLERY.md#ai): `ai-backend-template`, `vector-search`, `rag-pipeline`, `wasi-plugin`
- [Testing](GALLERY.md#testing): `testing-suite`

The current alpha `ag` binary accepts only `rest`, `realtime`, `fullstack` and
`layered`. The other names arrive with the wave-6 catalog registry and the
template directories. Each template's `template.toml` is authoritative
for `external_requirements`; the gallery marks inferred values with
`confirm`.

Back to the [documentation index](../INDEX.md) or the
[offline prototype guide](../examples/alpha-prototype.md).

---

# Indice de plantillas

`ag new <nombre> --template <nombre>` crea un proyecto a partir de un
directorio de [`templates/`](../../templates/). La tabla de decision canonica
es la [galeria de plantillas](GALLERY.md): para cada nombre del catalogo de la
ola 6 indica caso de uso, requisitos externos y comando de ejemplo.

- [Nucleo](GALLERY.md#core): `rest`, `realtime`, `fullstack`, `layered`
- [Web](GALLERY.md#web): `static-site`, `marketing-site`, `blog`, `docs-site`, `dashboard`, `admin-panel`
- [API](GALLERY.md#api): `database-api`, `auth-service`, `multi-tenant-saas`, `webhook-gateway`, `file-upload-service`, `ecommerce-lite`
- [Tiempo real](GALLERY.md#realtime): `realtime-chat-v2`, `websocket-service`, `worker-service`, `scheduled-jobs`, `iot-ingest`, `edge-router`
- [Infraestructura](GALLERY.md#infrastructure): `mail-service`, `dns-manager`, `object-storage-api`, `monitoring-service`, `microservices-starter`, `cli-tool`
- [IA](GALLERY.md#ai): `ai-backend-template`, `vector-search`, `rag-pipeline`, `wasi-plugin`
- [Testing](GALLERY.md#testing): `testing-suite`

El binario `ag` alpha actual acepta solo `rest`, `realtime`, `fullstack` y
`layered`. Los demas nombres llegan con el registro del catalogo y los
directorios de plantillas de la ola 6. El `template.toml` de cada plantilla es
la fuente autoritativa de `external_requirements`; la galeria marca con
`confirm` los valores inferidos.

Volver al [indice de documentacion](../INDEX.md) o a la
[guia de prototipo offline](../examples/alpha-prototype.md).
