# CLAUDE.md - Reglas Maestras de Implementacion y Gobernanza Tecnica de Anti-Gravital

## 0. Principio absoluto del repositorio

Antes de escribir la primera linea de codigo funcional, Anti-Gravital debe existir primero como sistema arquitectonico documentado, verificable y entendible.

La documentacion es la fuente de verdad del proyecto.

El codigo implementa la documentacion.
La documentacion NO se adapta al codigo improvisado.

Claude Code tiene estrictamente prohibido:

- improvisar arquitectura,
- inventar componentes fuera del alcance definido,
- alterar el proposito del proyecto,
- anadir features no contempladas,
- reinterpretar la vision original sin RFC,
- crear complejidad innecesaria,
- convertir Anti-Gravital en un "framework que intenta resolverlo todo".

El objetivo es construir infraestructura real, sostenible y tecnicamente defendible.

---

## 1. Regla critica de inicializacion del repositorio

### OBLIGATORIO ANTES DE CUALQUIER IMPLEMENTACION

Antes de implementar cualquier modulo, crate, benchmark, feature o linea de logica funcional:

Claude Code DEBE instalar, organizar y mantener la documentacion maestra dentro del repositorio.

Sin documentacion estructurada:
NO se escribe codigo.

---

## 2. Documentos maestros obligatorios

Los siguientes documentos son obligatorios y deben existir exactamente con estos nombres:

- `ANTI-GRAVITAL-Blueprint-v4.0.pdf`
- `ANTI-GRAVITAL-Arquitectura-Tecnica.md`
- `ANTI-GRAVITAL-Hoja-de-Ruta.md`

Estos documentos son la fuente de verdad oficial del proyecto.

Jamas deben:

- renombrarse,
- fragmentarse arbitrariamente,
- duplicar conceptos inconsistentes,
- reemplazarse con versiones simplificadas,
- moverse fuera de la documentacion principal.

---

## 3. Estructura documental obligatoria del repositorio

```
anti-gravital/
|
|-- README.md
|-- LICENSE
|-- CONTRIBUTING.md
|-- GOVERNANCE.md
|-- SECURITY.md
|-- CODE_OF_CONDUCT.md
|-- CLAUDE.md
|
|-- docs/
|   |-- master/
|   |   |-- ANTI-GRAVITAL-Blueprint-v4.0.pdf
|   |   |-- ANTI-GRAVITAL-Arquitectura-Tecnica.md
|   |   `-- ANTI-GRAVITAL-Hoja-de-Ruta.md
|   |
|   |-- architecture/
|   |-- roadmap/
|   |-- modules/
|   |-- dsl/
|   |-- benchmarks/
|   |-- security/
|   |-- governance/
|   |-- examples/
|   |-- es/
|   `-- en/
|
|-- crates/
|-- examples/
|-- templates/
|-- plugins/
|-- benchmarks/
`-- tools/
```

---

## 4. Regla de prioridad documental

La documentacion es un contrato arquitectonico.

Orden de prioridad:

1. `ANTI-GRAVITAL-Hoja-de-Ruta.md` define QUE se puede construir y CUANDO.
2. `ANTI-GRAVITAL-Arquitectura-Tecnica.md` define COMO se construye.
3. `ANTI-GRAVITAL-Blueprint-v4.0.pdf` define VISION, POSICIONAMIENTO y ALCANCE.

Si el codigo contradice la documentacion: el codigo esta mal.

---

## 5. Regla de bloqueo arquitectonico

NO se puede implementar nada que:

- no exista en la documentacion,
- contradiga el alcance,
- rompa la modularidad,
- altere las fases,
- viole las reglas entre crates,
- transforme Anti-Gravital en un proyecto distinto.

Si una idea nueva parece buena:

1. NO implementarla.
2. Crear RFC.
3. Documentarla.
4. Esperar aprobacion.
5. Solo entonces considerar implementacion.

---

## 6. Objetivo principal de la documentacion

La documentacion existe para impedir:

- desviaciones arquitectonicas,
- scope creep,
- features alucinadas,
- complejidad innecesaria,
- contradicciones internas,
- perdida de vision,
- deuda tecnica temprana,
- improvisacion guiada por hype,
- reescrituras constantes.

Cualquier feature no documentada es sospechosa hasta demostrarse lo contrario.

---

## 7. Regla de comprension previa

Antes de modificar cualquier archivo fuente se debe leer y comprender:

- arquitectura,
- limites,
- modulos,
- responsabilidades,
- roadmap,
- dependencias,
- vision,
- objetivos de rendimiento,
- restricciones,
- reglas de interoperabilidad.

NO se permite codificacion a ciegas.

---

## 8. Regla de implementacion incremental

Anti-Gravital se construye por fases bloqueantes.

Jamas debe:

- adelantarse de fase,
- implementar sistemas futuros prematuramente,
- dejar preparado codigo para features lejanas,
- anadir abstracciones especulativas.

Solo se implementa lo necesario para la fase actual.

---

## 9. Filosofia de ingenieria obligatoria

Se siguen:

- SOLID
- DRY
- KISS
- Clean Architecture
- Composition over inheritance
- Explicit over implicit
- Zero-cost abstractions
- Seguridad por construccion
- Performance-first engineering
- Observabilidad nativa
- Diseno modular
- Compatibilidad evolutiva
- Estabilidad semantica

---

## 10. Filosofia anti-hype

Anti-Gravital NO se construye para aparentar innovacion.

Se construye para:

- resolver problemas reales,
- reducir complejidad operacional,
- mejorar DX sin sacrificar rendimiento,
- aprovechar Rust correctamente,
- crear una arquitectura sostenible.

Se evita:

- marketing tecnico falso,
- benchmarks manipulados,
- claims exagerados,
- features cool sin proposito,
- complejidad por ego tecnico.

---

## 11. Regla de realismo tecnico

Toda decision debe considerar:

- mantenibilidad,
- costo operacional,
- memoria,
- cold start,
- concurrencia,
- seguridad,
- debugging,
- DX,
- CI/CD,
- observabilidad,
- compatibilidad multiplataforma,
- estabilidad futura.

---

## 12. Regla de interoperabilidad

Cuando exista una herramienta dominante:

Anti-Gravital se integra, NO la reemplaza.

NO se reinventan:

- Kubernetes
- Docker
- PostgreSQL
- Redis
- NATS
- MinIO
- Terraform
- Flutter
- React
- Next.js

Estrategia: integracion inteligente.

---

## 13. Regla de simplicidad operacional

Cada componente debe reducir complejidad operacional.

Se favorece:

- binarios estaticos,
- despliegues simples,
- configuraciones explicitas,
- observabilidad integrada,
- CI reproducible,
- builds deterministas,
- dependencia minima,
- runtime minimo.

---

## 14. Regla de crates

La separacion es estricta. El ecosistema tiene 17 crates clasificados en
cuatro niveles. La clasificacion "estandar diferido" fue introducida por
`ADR-0007` (Fase 4.5):

Nucleo:

- `ag-core`
- `ag-dsl`
- `ag-cli`
- `ag-wasm-host`

Estandar:

- `ag-auth`
- `ag-data`
- `ag-realtime`
- `ag-cache`
- `ag-storage`
- `ag-observe`

Estandar diferido (madurez de estandar, no instalado por defecto en
templates oficiales; se incorpora cuando el proyecto lo necesita):

- `ag-mail` — comunicacion transaccional outbound + adapters. Solo
  outbound en v1. NO es un MTA. Consumido por `ag-auth` para
  verificacion, recuperacion y magic links. `ag-mail` NO depende de
  `ag-auth` (sexta regla de dependencias).

Opcionales:

- `ag-ui`
- `ag-cloud`
- `ag-ai`
- `ag-mobile`
- `ag-migrate`

Opcionales de infraestructura:

- `ag-domains` — gestion DNS via trait `DnsProvider` + adapters
  (Cloudflare), certificados ACME (Let's Encrypt), generacion
  SPF/DKIM/DMARC. NO es registrador. NO reemplaza Terraform. Consumido
  opcionalmente por `ag-cloud` durante `ag deploy` (septima regla de
  dependencias).

---

## 15. Regla de dependencias

- `ag-core` NO depende de ningun crate Anti-Gravital.
- NO dependencias circulares.
- Versionado semantico independiente.
- Features Cargo bien aisladas.
- Dependencias minimas.

Cada nueva dependencia debe justificarse por:

- madurez,
- mantenimiento,
- seguridad,
- performance,
- estabilidad,
- necesidad real.

---

## 16. Regla de seguridad

Seguridad primero.

Se debe:

- minimizar superficie de ataque,
- evitar `unsafe`,
- documentar `unsafe` obligatorio,
- usar defaults seguros,
- evitar secretos hardcodeados,
- validar entrada tempranamente,
- aplicar least privilege,
- usar analisis estatico,
- ejecutar `cargo audit`,
- ejecutar `cargo deny`,
- ejecutar `clippy`.

---

## 17. Regla de benchmarks

Nunca inventar metricas.

Todo benchmark debe incluir:

- hardware,
- sistema operativo,
- version Rust,
- commit,
- configuracion,
- metodologia,
- numero de ejecuciones,
- desviacion estandar.

---

## 18. Regla de testing

Todo sistema critico requiere:

- unit tests,
- integration tests,
- fuzzing,
- regression tests,
- benchmarks,
- E2E tests.

Sin tests: NO esta terminado.

---

## 19. Regla de documentacion continua

Toda feature publica requiere:

- explicacion,
- ejemplos,
- limitaciones,
- estado de estabilidad,
- notas de compatibilidad.

La documentacion se actualiza junto al codigo. Nunca despues.

---

## 20. Regla del DSL

El DSL es la fuente de verdad del ecosistema.

Pipeline:

```
schema.ag
    |
    v
lexer
    |
    v
parser
    |
    v
AST
    |
    v
semantic analysis
    |
    v
diagnostics
    |
    v
codegen
```

El DSL debe generar:

- Rust,
- SQL,
- OpenAPI,
- TypeScript,
- Dart,
- migraciones,
- SDKs,
- knowledge graph.

---

## 21. Regla anti-complejidad

Se rechazan:

- abstracciones prematuras,
- metaprogramacion innecesaria,
- macros opacas,
- sistemas magicos,
- configuracion excesiva,
- acoplamiento implicito.

El sistema debe ser:

- auditable,
- legible,
- explicito,
- depurable,
- razonable para contribuidores externos.

---

## 22. Regla de gobernanza tecnica

Toda decision grande requiere RFC.

Incluye:

- cambios de arquitectura,
- cambios del DSL,
- cambios de seguridad,
- cambios de crates,
- cambios de performance targets,
- cambios de roadmap.

Sin RFC: NO implementar.

---

## 23. Regla de comportamiento del agente de codigo

Se comporta como:

- arquitecto principal,
- ingeniero de plataforma,
- mantenedor open source senior,
- reviewer critico,
- auditor tecnico.

NO como generador automatico de codigo improvisado.

---

## 24. Checklist obligatorio antes de finalizar tareas

Antes de marcar algo como terminado:

- [ ] Pertenece a la fase correcta.
- [ ] Respeta la documentacion.
- [ ] No rompe arquitectura.
- [ ] No anade complejidad innecesaria.
- [ ] No crea dependencias circulares.
- [ ] Compila.
- [ ] Pasa tests.
- [ ] Pasa fmt.
- [ ] Pasa clippy.
- [ ] Pasa audit.
- [ ] Tiene documentacion.
- [ ] Tiene benchmarks si aplica.
- [ ] Tiene observabilidad si aplica.
- [ ] Tiene manejo de errores correcto.
- [ ] Tiene trazabilidad.
- [ ] Mantiene coherencia con Anti-Gravital v4.0.

---

## 25. Frase rectora del proyecto

"Construir Anti-Gravital como infraestructura real, modular, verificable y sostenible; nunca como una demo inflada por hype tecnico."

---

## 26. Regla de sincronizacion Codigo <-> Documentacion

Existen dos productos:

1. El producto tecnico (codigo).
2. El producto arquitectonico (documentacion).

Ambos evolucionan juntos.

Esta prohibido:

- actualizar codigo sin actualizar documentacion afectada,
- actualizar documentacion simulando capacidades inexistentes,
- introducir comportamiento no documentado,
- dejar documentacion obsoleta.

Toda modificacion debe responder:

- Que documento afecta?
- Que modulo afecta?
- Que fase afecta?
- Que contrato cambia?

---

## 27. Flujo obligatorio para cualquier trabajo

### Etapa 1 - Comprension

Leer:

- Blueprint
- Arquitectura Tecnica
- Hoja de Ruta
- RFC relacionados

Salida requerida:

```
Comprension completada:
- Objetivo:
- Restricciones:
- Fase actual:
- Riesgos:
- Entregables:
```

NO escribir codigo.

### Etapa 2 - Validacion de alcance

Determinar:

```
Esto pertenece al alcance?           Si / No
Pertenece a esta fase?               Si / No
Requiere RFC?                        Si / No
Existe contradiccion documental?     Si / No
```

Si existe contradiccion: DETENER.

### Etapa 3 - Diseno

Generar:

```
Arquitectura propuesta
Dependencias
Riesgos
Impacto
Tests
Benchmark
Rollback
```

NO escribir codigo.

### Etapa 4 - Implementacion

Implementar minimo cambio funcional.

Prioridad: Correctitud > Seguridad > Simplicidad > Rendimiento. Nunca al reves.

### Etapa 5 - Verificacion

Ejecutar:

```
cargo fmt
cargo clippy
cargo test
cargo audit
cargo deny
cargo bench
```

Si falla: NO finalizar.

### Etapa 6 - Documentacion

Actualizar:

- docs/
- ejemplos
- README
- CHANGELOG
- roadmap

---

## 28. Regla de RFC

Se crea RFC antes de:

- introducir nuevos crates,
- alterar limites entre modulos,
- cambiar targets de rendimiento,
- modificar DSL,
- introducir plugins,
- cambiar CI,
- cambiar stack,
- introducir IA,
- anadir comandos CLI.

Formato:

```
RFC-XXXX

Titulo:
Motivacion:
Problema:
Alternativas:
Diseno:
Riesgos:
Impacto:
Rollback:
```

Ubicacion: `docs/rfc/`

---

## 29. Regla de deuda tecnica

Toda deuda tecnica debe ser explicita.

Prohibido:

- TODO sin contexto,
- hacks silenciosos,
- codigo temporal permanente.

Formato del marcador en el codigo:

```rust
// TECH-DEBT (issue #123):
// motivo:
// impacto:
// eliminacion esperada:
```

### Seguimiento en GitHub Issues (no en archivos del repositorio)

La deuda tecnica, los bugs, los fallos y la documentacion de problemas se
registran como **GitHub Issues**, NO como archivos Markdown dentro del
repositorio. El objetivo es no llenar el repo de archivos que documentan
errores/deuda y mantener ese seguimiento organizado y buscable en Issues.

Reglas:

- Toda deuda nueva se abre como Issue con la etiqueta `tech-debt` (y `bug`,
  `security`, etc. segun corresponda), con motivo, impacto, plan de
  eliminacion y prioridad/fecha objetivo en el cuerpo del Issue.
- Los marcadores `// TECH-DEBT` en el codigo referencian el numero/URL del
  Issue; no describen la deuda por extenso ni la duplican en archivos.
- Esta PROHIBIDO crear nuevos archivos cuyo proposito sea documentar deuda,
  fallos, auditorias de problemas o "pendientes" (p. ej. `*-DEBT.md`,
  `BACKLOG.md`, `TODO.md`, notas de problemas). Esa informacion va a Issues.
- `docs/DEBT.md` queda CONGELADO como registro historico: no se anaden
  entradas nuevas; las existentes se migran a Issues al tocarlas y se
  reemplazan por su referencia. No se borra de golpe para no perder
  trazabilidad.
- La documentacion tecnica que SI permanece en el repo es la que describe el
  sistema tal como es (arquitectura, modulos, RFC/ADR aprobados, referencia,
  guias), no la que enumera lo que falta o esta roto.

Excepcion: una RFC/ADR puede enumerar fases o limitaciones como parte de una
decision de diseno; eso es contrato arquitectonico, no un registro de deuda.

---

## 30. Regla de eliminacion de codigo

Preferencia:

Eliminar > Simplificar > Refactorizar > Agregar

Si una funcionalidad no se usa, rompe diseno o anade complejidad: evaluar eliminarla.

Cada linea nueva aumenta mantenimiento.

---

## 31. Regla de APIs publicas

Toda API publica debe cumplir:

- Estabilidad: evitar cambios innecesarios.
- Descubribilidad: debe ser intuitiva.
- Compatibilidad: no romper usuarios.
- Explicitud: nada implicito.
- Versionado: SemVer obligatorio.

---

## 32. Regla de ergonomia para desarrolladores

Todo lo generado por Anti-Gravital debe sentirse rapido, intuitivo, predecible y consistente.

Si requiere leer 100 paginas para crear una API: fallo el diseno.

Objetivo:

```
ag new my-api
ag dev
```

y comenzar rapido.

---

## 33. Regla del Knowledge Graph

Se mantiene conocimiento estructurado en `docs/graph/`.

Debe contener:

- modulos,
- relaciones,
- crates,
- comandos,
- DSL,
- eventos,
- ejemplos,
- decisiones.

Formato preferido: JSON, Markdown, OpenAPI, Mermaid.

Nunca conocimiento oculto en conversaciones.

---

## 34. Regla de diagramas

Toda arquitectura importante requiere:

- diagrama logico,
- diagrama fisico,
- flujo de requests,
- dependencias,
- despliegue.

Ubicacion: `docs/diagrams/`

Formatos: Mermaid, SVG, PNG exportado. Nunca diagramas unicamente embebidos en PDFs.

---

## 35. Regla de decisiones arquitectonicas (ADR)

Toda decision importante se persiste en `docs/adr/`.

Formato:

```
ADR-XXX

Contexto
Decision
Consecuencias
Alternativas
Estado
```

---

## 36. Regla de reproducibilidad

Todo resultado debe poder repetirse.

Incluye:

- builds,
- benchmarks,
- despliegues,
- generacion DSL,
- CI.

Si otro desarrollador no puede reproducirlo: se considera fallo.

---

## 37. Regla Open Source

Todo se disena para colaboradores externos.

Se evita:

- conocimiento tribal,
- procesos ocultos,
- pasos manuales secretos.

Cada contribuidor debe poder:

```
git clone
cargo build
cargo test
```

sin depender del mantenedor.

---

## 38. Regla de mantenimiento a largo plazo

Se optimiza para 24-60 meses, no para 2 semanas.

Preguntas obligatorias:

- Como se actualiza?
- Como se depura?
- Como se migra?
- Como se desactiva?
- Como se reemplaza?

---

## 39. Regla de salud del proyecto

Nunca optimizar por estrellas.

Prioridad: Calidad > Adopcion > Comunidad > Popularidad.

No perseguir hype.

---

## 40. Regla final

Si existe duda entre hacer mas o hacerlo mejor: siempre elegir hacerlo mejor.

---

## Reglas adicionales del repositorio (no negociables)

### Limite de 256 caracteres

Todo mensaje de commit y todo titulo de pull request del proyecto debe
tener un maximo de 256 caracteres. Las descripciones largas, listas de
cambios y notas adicionales pertenecen al cuerpo de la PR o al
CHANGELOG, no al asunto del commit o al titulo de la PR. Cuando un
mensaje exceda el limite, se recorta o se compacta sin perder claridad.

### Ausencia de evidencia de herramientas IA

Ninguna rama, ningun commit, ninguna pull request, ningun comentario en
el codigo, ningun archivo de documentacion y ningun fragmento de texto
publicado en este repositorio puede contener evidencia de que se uso
una herramienta IA. Esto incluye:

- nombres de ramas con prefijo `claude/`, `gpt/`, `ai/`, etc.,
- pies de commit con atribucion de IA,
- URLs de sesiones de chat con asistentes,
- "Co-Authored-By" de modelos o agentes,
- comentarios en codigo del tipo "generado por IA",
- firmas de herramientas IA en documentos.

El trabajo se atribuye unicamente a personas humanas y a la
organizacion responsable.

### Descriptor pre-rellenado y autofill por PR

Toda rama que vaya a producir una pull request acompana sus commits
con un descriptor pre-rellenado bajo
`docs/pr-drafts/<rama-aplanada>.md` (las `/` del nombre de la rama se
convierten en `-`). El descriptor contiene el resumen final (titulo
del PR), fase afectada, tipo de cambio, documentos relacionados,
plan de prueba, criterios de salida que avanza y checklist final,
todo con valores concretos en lugar de los placeholders de la
plantilla.

El workflow `.github/workflows/pr-autofill.yml` se dispara al abrir
o reabrir la pull request, busca el descriptor por nombre de rama
aplanada y reemplaza el cuerpo del PR con su contenido completo. Si
no encuentra descriptor, comenta el PR avisando y marca el job como
warning.

La plantilla en `.github/PULL_REQUEST_TEMPLATE.md` es solo un aviso
que aparece cuando el autofill no encuentra descriptor.

Si un agente o colaborador commitea sin crear o actualizar el
descriptor correspondiente, la PR no se acepta.

### Politica de idioma (ADR-0008)

El ingles es el idioma canonico del codigo y de la documentacion tecnica.
Regla fijada por `ADR-0008` (supersede el "espanol predeterminado" de
`ADR-0002`) al cierre de la Fase 4.5.

Codigo:

- Todos los comentarios de codigo (`//`, `///`, `//!`) se escriben en
  ingles. Sin excepciones para codigo nuevo.
- Los identificadores van en ingles (ya era la practica).

Documentacion:

- Ingles canonico para la documentacion tecnica profunda: `docs/architecture/`,
  `docs/modules/`, `docs/dsl/`, `docs/rfc/`, `docs/adr/`, `docs/benchmarks/`,
  `docs/security/`, `docs/governance/`.
- Documentos vitrina bilingues (EN+ES en el mismo archivo, seccion inglesa
  primero y canonica, seccion espanola despues, separadas por regla
  horizontal, con ancla `English | Espanol` al inicio):
  - `README.md` (raiz),
  - los tres maestros de `docs/master/`,
  - los capitulos de `docs/manual/`.
- Si la seccion inglesa y la espanola divergen, gana la inglesa; la espanola
  se marca como pendiente de actualizacion.

Esta regla NO se aplica retroactivamente como bloqueo: la documentacion
tecnica preexistente en espanol se migra a ingles de forma gradual al
tocarse. El codigo se convirtio integramente en la fase de cierre documental
4.0-4.5.

### Prohibicion absoluta de emojis

No se usan emojis en ningun lugar del proyecto: ni en documentacion, ni
en codigo, ni en texto de interfaces de usuario, ni en comentarios, ni
en mensajes de commit, ni en titulos de PR, ni en issues, ni en
plantillas. Los iconos se manejan como SVG o glifos tipograficos cuando
sea estrictamente necesario.

### Politica de marcas comerciales de terceros (ADR-0011)

No se usan nombres de marcas comerciales de terceros para nombrar componentes
del proyecto: ni en codigo, ni en identificadores, ni en features de Cargo, ni
en comentarios, ni en documentacion. Un nombre de marca comercial de tercero
solo se admite con una unica condicion:

- como etiqueta explicita de un adaptador que integra especificamente a ese
  tercero (para que el contribuidor sepa donde "enchufar" ese servicio),
  confinado al modulo de ese adaptador y detras de su feature de Cargo, y
  solo cuando NO existe una via nativa generica que cubra la misma necesidad.

Esta prohibido usar una marca comercial como nombre de un componente propio o
generico, como parte de la superficie publica del producto, o por comodidad de
no inventar un nombre propio.

Excepciones:

- Tecnologias open-source en las que nos apoyamos o que usamos para construir
  Anti-Gravital no son marcas comerciales prohibidas y se mantienen (p. ej.
  `lettre`, `mail-send`, `mail-auth`, `hickory-resolver`, `rustls`, `tokio`,
  `axum`, `sqlx`, `moka`).
- Proveedores de buzon/destino citados como requisito de interoperabilidad o
  entregabilidad (aquellos cuyas reglas SPF/DKIM/DMARC o de envio masivo hay
  que cumplir) no son componentes nuestros y se mantienen donde sea
  tecnicamente necesario.

Caso aplicado: `ag-mail` no expone adaptadores con nombre de marca; para usar
un proveedor externo se apunta el `SmtpSender` nativo a su endpoint SMTP, y la
via sin terceros es el `MtaSender` nativo. Los adaptadores `CloudflareProvider`
(`ag-domains`) y `S3Store` (`ag-storage`) se conservan como etiquetas legitimas
de adaptador. El job `prohibited content scan` en `.github/workflows/docs.yml`
verifica la ausencia de las marcas de correo retiradas.

### Sincronizacion obligatoria del README

El `README.md` es la ventana publica del proyecto. Cualquier cambio que
altere el estado observable del proyecto debe actualizar el README en el
mismo commit o PR. No se acepta un README desfasado de la realidad.

Cambia que obligan a actualizar el README de forma inmediata:

- Avance de fase o cambio del estado de una fase (de "proxima" a "en
  curso", de "en curso" a "completada", etc.).
- Nuevo comando de la CLI operativo (`ag generate`, `ag schema lint`, etc.).
- Nueva version del DSL implementada (v0.1, v0.2, v0.3...).
- Nuevo crate publicado o promovido a estado funcional.
- Nuevo ejemplo disponible en `examples/`.
- Nuevos benchmarks medidos en hardware real.
- Cambio en los requisitos de instalacion (version minima de Rust,
  nueva dependencia de sistema, etc.).
- Cualquier cambio que haga que la seccion "Estado del proyecto" o
  la tabla "Calendario" queden incorrectos.

Que NO requiere actualizar el README:

- Correcciones de bugs internos sin efecto en la API publica.
- Mejoras de tests, cobertura o CI.
- Cambios en documentacion tecnica interna (`docs/architecture/`, `docs/adr/`, etc.).
- Actualizaciones de dependencias sin cambio de comportamiento visible.

El README se actualiza en el mismo commit que el cambio que lo provoca.
Nunca en un commit separado posterior. Nunca "lo actualizo despues".

Un README que dice "Fase 3 proxima" cuando el codigo ya implementa el
compilador DSL es una mentira tecnica y viola esta regla.

### Reglas de estado real y autosuficiencia (ADR-0009)

1. Prohibido marcar un modulo, crate o API como "vacio", "skeleton", "no
   implementado" o equivalente cuando ya contiene codigo funcional y/o pruebas.
   El estado declarado en README y en la cabecera `//!` debe corresponder a la
   realidad del codigo. La deuda se enumera en `docs/DEBT.md`, no se disfraza de
   "skeleton".
2. Toda integracion con un servicio externo (Redis, NATS, S3, Cloudflare, SMTP de
   terceros, etc.) debe pasar por un adaptador detras de una feature de Cargo, y el
   crate debe conservar un modo de operacion nativo por defecto. La dependencia
   externa nunca es un requisito para usar el modulo.
3. La documentacion tecnica afectada se actualiza en la MISMA PR que el codigo y se
   revisa junto a el. Una PR que cambia comportamiento observable sin actualizar la
   documentacion correspondiente no se acepta.
4. Los scripts de instalacion (`install.sh`, `install.ps1`) deben ser auditables y
   verificar integridad/firma antes de ejecutar acciones privilegiadas. No se
   distribuye un instalador que descargue y ejecute sin verificacion.
5. Toda dependencia externa de infraestructura (bases de datos, colas, caches
   distribuidas) debe ser reemplazable por una implementacion nativa o auto-alojable,
   salvo justificacion explicita en una RFC.

### Cierre

Esta es la constitucion tecnica del repositorio. Primero documentacion,
despues diseno, despues implementacion. El codigo queda subordinado al
contrato documental.
