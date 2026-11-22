# Indice maestro de la documentacion de Anti-Gravital

Este archivo es la entrada principal a la documentacion del proyecto.
Su funcion es enumerar todos los documentos navegables y senalar a un
agente (humano o automatizado) que falta al proyecto en cada momento.

## Fuente de verdad

Los maestros son la unica fuente de verdad. Viven en `docs/master/` y
no se editan fuera de un procedimiento RFC. Todos los demas archivos
bajo `docs/` son derivados verbatim de los maestros.

| Documento maestro | Archivo |
| --- | --- |
| Vision, posicionamiento y alcance (canonico) | `docs/master/ANTI-GRAVITAL-Blueprint-v4.1.md` |
| Vision, posicionamiento y alcance (PDF historico, desfasado) | `docs/master/ANTI-GRAVITAL-Blueprint-v4.0.pdf` |
| Arquitectura tecnica e implementacion | `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` |
| Hoja de ruta y puertas de verificacion | `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` |
| Registro de integridad de los maestros | `docs/master/VERSION.md` |

El Blueprint canonico es `ANTI-GRAVITAL-Blueprint-v4.1.md`. El PDF v4.0 se
conserva como artefacto historico de presentacion y esta desfasado: cuando
difiera del markdown, gobierna `v4.1.md` (`docs/master/VERSION.md`, politica
#4).

## Como leer la documentacion

1. Empiece por la version vigente en `docs/master/VERSION.md`.
2. Lea el preambulo de la Hoja de Ruta:
   `docs/roadmap/preambulo.md`.
3. Identifique la fase activa en `docs/roadmap/STATUS.md`. El estado del hito
   alpha (ola 5) y las filas abiertas de la puerta pre-Fase 5 estan registrados
   en la seccion `Hito alpha` de ese mismo archivo.
4. Lea el capitulo de arquitectura correspondiente a la fase activa
   bajo `docs/architecture/`.
5. Lea el README del modulo o crate especifico bajo
   `docs/modules/<crate>/`.
6. Lea RFC y ADR vigentes en `docs/rfc/` y `docs/adr/`.

## Mapa de derivados

### docs/architecture/

Descomposicion verbatim del maestro de Arquitectura Tecnica en 20
archivos numerados, uno por capitulo. Indice en
`docs/architecture/README.md`.

### docs/roadmap/

Descomposicion verbatim de la Hoja de Ruta en archivos por fase y por
seccion (preambulo, resumen, reglas de oro, calendario, mas alla de
1.0). Estado vivo del tablero en `docs/roadmap/STATUS.md`.

### docs/modules/

Un subdirectorio por cada crate del workspace, con README propio que
referencia el capitulo de arquitectura correspondiente y la fase de
implementacion.

### docs/dsl/

Subsecciones del capitulo 7 de Arquitectura Tecnica, dedicadas al
Anti-DSL. Incluye la referencia implementada v0.1–v0.4
(`referencia-v01-v04.md`) y la hoja de ruta del servidor LSP
(`lsp-roadmap.md`).

### docs/fuzz/

Documentacion del harness de fuzzing del compilador DSL (cargo-fuzz).
Incluye instrucciones para el gate manual de 24h requerido antes de
cerrar Fase 3 e historial de crashes encontrados y corregidos.

### docs/benchmarks/, docs/security/, docs/governance/

Vistas verbatim de los capitulos 16, 15 y 17 del maestro de
Arquitectura, respectivamente.

### docs/audits/

Registros de auditoria y revisiones independientes, no derivados verbatim de los
maestros: recogen evidencia de verificacion y hallazgos sin modificar el
contrato arquitectonico. Incluye `infrastructure-review-112-149-141.md`
(revision de infraestructura de los issues #112, #149 y #141).

### docs/releases/

Notas de las versiones etiquetadas, con alcance, limitaciones conocidas,
evidencia de verificacion y ruta de actualizacion. Referencia:
`docs/releases/0.1.0-alpha.1.md` (primer alpha, solo para pruebas).

### docs/rfc/ y docs/adr/

Procesos formales del proyecto: RFC para decisiones tecnicas
importantes y ADR para decisiones arquitectonicas estables. Plantillas
en `docs/rfc/template.md` y `docs/adr/template.md`.

### docs/audits/

Revisiones tecnicas y de gobernanza con evidencia sobre el estado real
del repositorio. Incluye `mcp-readiness-review.md`, el analisis de
brechas de RFC-0024 y ADR-0017 antes de implementar la interfaz de
maquina por capacidades (solo diseno, sin codigo). Referencia:
`docs/audits/mcp-readiness-review.md`.
La reconciliacion del tablero de issues de la ola 4 (matriz de los 13
issues abiertos, etiquetas, comentarios de estado y condiciones de cierre)
esta en `docs/audits/issue-board-reconciliation.md`.

### docs/diagrams/, docs/graph/, docs/examples/

Espacios reservados para diagramas, knowledge graph y catalogo de
ejemplos. Se pueblan a partir de la fase indicada en cada README.
El catalogo de ejemplos incluye la guia de prototipo alpha
`docs/examples/alpha-prototype.md`: de cero a una API local y un SDK
generado sin servicios externos, con matriz de infraestructura requerida
y evidencia de verificacion.

### docs/templates/

Indice y galeria de las plantillas de proyecto consumidas por `ag new`.
`docs/templates/GALLERY.md` es la tabla de decision de caso de uso a plantilla,
requisitos externos y comando de ejemplo para los 33 nombres del catalogo de la
ola 6. `docs/templates/README.md` es su indice bilingue, enlazado tambien desde
el README raiz y desde `docs/manual/02-primera-api.md`. La disponibilidad real
de cada nombre depende del cableado del catalogo en la CLI.

### docs/es/ y docs/en/

Indices por idioma. Funcionan como atajo bilingue al resto de `docs/`.

### deploy/k8s/

Empaquetado opcional de Kubernetes para las familias de plantillas (web, api,
realtime, infrastructure y ai). No es un requisito: el camino por defecto
sigue siendo `ag new`, `cargo run` y `ag dev` (regla 12 de CLAUDE.md y
ADR-0009). Contiene una raiz de kustomize, una base comun endurecida, un
overlay por familia, componentes opcionales de HPA e Ingress desactivados
por defecto, `README.md` con comandos de apply/delete, `SECRETS.md` con el
patron de inyeccion de secretos sin valores enlinea y `EVIDENCE.md` con la
verificacion offline real (sin despliegue en cluster).

## Politica de derivados

Si un derivado contradice un maestro, gana el maestro. Para corregir
un derivado, se regenera a partir del maestro con
`bash tools/split-masters.sh`. Para corregir un maestro, se abre RFC.

## Que falta al proyecto

Para saber que esta pendiente en cada momento, consulte
`docs/roadmap/STATUS.md`. Ese archivo agrega el estado de cada casilla
de cada fase y se actualiza con cada pull request que avanza la hoja
de ruta.
Registros de verificacion tecnica y revisiones de bajo nivel. Incluye
las puertas `PRE_FASE5_RELEASE_GATE.md` y `pre-fase5-*.md`, y la revision
`low-level-review-178-180.md` (stack de correo e RFC de chumsky).
Incluye tambien `wave4-review.md`, la revision de seguridad y de bajo
nivel de las siete ramas de la wave 4 (tui, machinejson, layered,
envsecrets, guardrails, hooks y qa), con hallazgos, severidad y
recomendaciones. Referencia: `docs/audits/wave4-review.md`.
La revision de seguridad del camino de release alpha (ola 5) registra el
estado de los controles de instalacion, prototypes y workflows, con dos
defectos (#191 y #192) y cuatro observaciones. Referencia:
`docs/audits/alpha-release-security-review.md`.
Convenciones de diseno de las plantillas de proyecto y sus artefactos de
despliegue. Referencia tecnica en ingles (canonico segun ADR-0008):
`docs/templates/TEMPLATE_DESIGN.md` define el esqueleto estandar, nombres y
variables de entorno, contrato de `/health` y `/ready`, precedencia de
configuracion, secretos (RFC-0020), limite plantilla/adaptador, composicion,
ruta de actualizacion, matriz Docker/Kubernetes y checklist para nuevas
plantillas. La tabla de decision bilingue del catalogo esta en
`docs/templates/README.md` y `docs/templates/GALLERY.md` (ola 6).
### deploy/docker/
Empaquetado opcional en contenedores para las familias de plantillas
(`web`, `api`, `realtime`, `infrastructure` y `ai`). No es obligatorio:
`cargo run` y `ag dev` siguen siendo la ruta de desarrollo por defecto.
Incluye un `Dockerfile.template` multi-etapa con builder en `rust:1.99.1`,
runtime `scratch`, usuario no root, `EXPOSE` explicito y `HEALTHCHECK`, un
compose por familia con variables de entorno y volumenes con nombre, una guia
de adaptacion y evidencia de verificacion con SKIP explicito del build real
cuando no hay demonio disponible. Referencia: `deploy/docker/README.md` y
`deploy/docker/VERIFICATION.md`.
