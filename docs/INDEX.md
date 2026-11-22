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
cerrar Fase 3 e historial de crashes encontrados y corregidos. El
procedimiento canonico de verificacion y el esquema de evidencia JSONL
estan en `24h-gate-verification.md` y `windows-jsonl-schema.md`. El runbook
operativo (`24h-gate-runbook.md`) describe el preflight, los siete comandos con
`FUZZ_TARGETS`, los artefactos y los pasos humanos de cierre; su verificacion
estructural esta en `docs/audits/fuzz-24h-workflow-structural-verification.md`.
RFC-0036 autoriza el workflow programado `.github/workflows/fuzz-24h.yml`, que
solo corre en un runner auto-alojado con la etiqueta `fuzz-dedicated` y no
cierra la issue #156. La preparacion del host, el aislamiento de red, el
dimensionamiento de disco y los comandos de verificacion estan en
`docs/fuzz/self-hosted-runner.md`.
### docs/benchmarks/, docs/security/, docs/governance/
Vistas verbatim de los capitulos 16, 15 y 17 del maestro de
Arquitectura, respectivamente.
El handoff de una sesion para la puerta de benchmarks (preflight, `--dry-run`,
artefactos, edicion de la fila de puerta y del README EN/ES) esta en
`docs/benchmarks/REFERENCE_RUNBOOK.md`; la definicion del host de referencia y
el procedimiento canonico estan en `docs/benchmarks/reference-host.md`.
### docs/audits/
Registros de auditoria y revisiones independientes, no derivados verbatim de los
maestros: recogen evidencia de verificacion y hallazgos sin modificar el
contrato arquitectonico. Incluye `infrastructure-review-112-149-141.md`
(revision de infraestructura de los issues #112, #149 y #141).
La revision consolidada de la QA de terminal de la ola 10
(`terminal-user-review.md`) prioriza los hallazgos reales de los recorridos
de instalacion, TUI, scaffold, DNS/deploy y lockfiles, separa defectos de
friccion y dado a cada recomendacion un comando de verificacion.
### docs/releases/
Notas de las versiones etiquetadas, con alcance, limitaciones conocidas,
evidencia de verificacion y ruta de actualizacion. Referencia:
`docs/releases/0.1.0-alpha.2.md` (segundo alpha: plantillas, deploy local y
ciclo de dominios; solo para pruebas).
### docs/rfc/ y docs/adr/
Procesos formales del proyecto: RFC para decisiones tecnicas
importantes y ADR para decisiones arquitectonicas estables. Plantillas
en `docs/rfc/template.md` y `docs/adr/template.md`.
Revisiones tecnicas y de gobernanza con evidencia sobre el estado real
del repositorio. Incluye `mcp-readiness-review.md`, el analisis de
brechas de RFC-0024 y ADR-0017 antes de implementar la interfaz de
maquina por capacidades (solo diseno, sin codigo). Referencia:
`docs/audits/mcp-readiness-review.md`.
La reconciliacion del tablero de issues de la ola 4 (matriz de los 13
issues abiertos, etiquetas, comentarios de estado y condiciones de cierre)
esta en `docs/audits/issue-board-reconciliation.md`.
El kit de evidencia de staging de ACME (prerrequisitos, comandos, campos y
plantilla vacia; el mock local NO cierra la puerta) esta en
`docs/audits/acme-staging-handoff.md`.
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
### docs/dns/
Guia bilingue del circulo dominio-despliegue (ola 8): como una plantilla
generada se vuelve alcanzable mediante un dominio propio. Cubre requisitos,
los tres entornos, los comandos implementados frente a los planificados,
publico vs privado, resolucion de problemas y seguridad. La seccion inglesa
es canonica segun ADR-0008. Referencia: `docs/dns/README.md`.
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
### deploy/link/ y examples/domain-link/ (ola 8)
Enlace del ultimo paso del andamiaje: artefactos de despliegue a una URL
alcanzable. `deploy/link/render_ingress.py` lee el manifiesto de dominio
canonico (`docs/dns/ENVIRONMENT_RULES.md`, reglas ER-01..ER-11) y genera un
overlay de Kustomize con un parche JSON6902 que fija host/TLS del Ingress;
`link.sh` es el comando unico que renderiza y muestra la URL. Expone solo
los servicios con `visibility` explicita vpn-only o public: un servicio
private no genera Ingress (ER-08). `deploy/link/compose/local-host.yml` es un
override opcional para acceder con un host personalizado en local sin imagen
proxy externa. `examples/domain-link/` contiene un servicio HTTP sin
dependencias, un manifiesto con dev/staging/prod y `mock-hosts.sh`, que
verifica los tres entornos sin red ni demonio mediante `curl --resolve`.
La evidencia real esta en `docs/audits/domain-link-validation.md`.
No se contacta a ningun proveedor DNS ni cluster: Kubernetes y Docker siguen
siendo objetivos opcionales (regla 12 y ADR-0009).

### Guia de stacks locales (ola 7)
`docs/templates/STACKS.md` relaciona caso de uso, stack, familia de plantilla,
dependencia externa y puertos. Los archivos Compose estan en `deploy/stacks/`,
con un `README.md` por stack y una guia comun. Cada stack usa `.env.example`
sin secretos, volumenes con nombre y healthchecks. La evidencia de verificacion
(YAML y `docker compose config`, sin servicios iniciados) esta en
`deploy/stacks/VERIFICATION.md`.
La auditoria de seguridad de los artefactos de despliegue (ola 7) cubre
Dockerfiles, compose, manifiestos de Kubernetes, stacks y `ag deploy` con
hallazgos por familia, severidad y recomendacion de merge. Referencia:
`docs/audits/deployment-security-review.md`.
La auditoria de seguridad del circuito de dominios (ola 8) revisa el
aprovisionamiento DNS/dominio (`ag-domains`, `ag-edge`) y los disenos de las
ramas de la ola 8, con checklist de controles, hallazgos por componente y
recomendacion de go/no-go. Referencia:
`docs/audits/domain-loop-security-review.md`.
