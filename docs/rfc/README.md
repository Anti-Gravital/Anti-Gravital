# Proceso RFC de Anti-Gravital

> Reglas: 22, 28 y 35 de `CLAUDE.md`.

Una RFC (Request For Comments) es el mecanismo formal para proponer
decisiones tecnicas importantes. Toda decision que afecte la
arquitectura, el DSL, los objetivos de rendimiento, la lista de
crates, el CI, la seguridad, los plugins, la integracion de IA o los
comandos de la CLI requiere RFC aprobada antes de comenzar la
implementacion.

## Cuando abrir una RFC

Abra una RFC cuando proponga:

- Introducir un nuevo crate al workspace.
- Alterar los limites o las reglas de dependencia entre crates.
- Cambiar targets de rendimiento publicados.
- Modificar la gramatica del DSL.
- Introducir un nuevo plugin oficial o cambiar el modelo de plugins.
- Cambiar la pipeline de CI o las herramientas de calidad.
- Cambiar la stack de dependencias clave (runtime, parser, codegen).
- Anadir capacidades de IA al producto.
- Anadir un comando nuevo a la CLI.
- Modificar un documento maestro de `docs/master/`.

No necesita RFC para correcciones puntuales, mejoras de documentacion
sin cambio de alcance, o tareas de mantenimiento que sigan decisiones
ya aprobadas.

## Estructura de un archivo RFC

Cree un archivo `RFC-XXXX-titulo-corto.md` donde `XXXX` es el numero
secuencial de cuatro digitos. Use la plantilla `template.md` como base.

Las secciones obligatorias son: Titulo, Motivacion, Problema,
Alternativas, Diseno, Riesgos, Impacto y Rollback. Vease la regla 28.

## Flujo

1. Cree la RFC como archivo bajo `docs/rfc/` en una pull request.
2. Abra el periodo de comentarios. Minimo siete dias calendario.
3. Discusion en el PR. Los cambios de redaccion se integran como
   commits adicionales.
4. Decision: aceptada, rechazada o diferida. Quien decide depende de
   la fase:
   - Fase 0 a 3: BDFL.
   - Fase 4 en adelante: comite tecnico.
5. Si es aceptada, se mergea la RFC al `main` y se etiqueta como
   `accepted`. La implementacion vive en PRs posteriores que
   referencian la RFC por numero.
6. Si es rechazada, se etiqueta como `rejected` y se merge para
   preservar el historial de la decision.
7. Si es diferida, se etiqueta como `deferred` con una nota del motivo
   y la condicion para retomarla.

## Vigencia

Una RFC aceptada permanece vigente mientras su decision este reflejada
en el codigo y la documentacion. Si la realidad diverge, se abre una
RFC sucesora que la enmienda o reemplaza, y la anterior se etiqueta
como `superseded`.

## Lista de RFC

| Numero | Titulo | Estado | Archivo |
| --- | --- | --- | --- |
| 0001 | Paralelizar puertas externas de Fase 0 con implementacion de Fase 1 | aceptado | `RFC-0001-paralelizar-fase-0-externa-y-fase-1.md` |
| 0002 | Diseno del Shield MVP (Fase 1) | aceptado | `RFC-0002-diseno-shield-mvp.md` |
| 0003 | Librerias base del compilador ag-dsl (Fase 3) | aceptado | `RFC-0003-librerias-compilador-ag-dsl.md` |
| 0005 | ag-cache L2 nativo RESP2 (Fase 4) | propuesto | `RFC-0005-ag-cache-native-l2.md` |
| 0006 | ag-mail alcance, stack y plan de implementacion (Fase 4.5) | superseded por RFC-0009 (alcance ag-mail) | `RFC-0006-ag-mail-alcance.md` |
| 0007 | ag-domains alcance, stack y plan de implementacion (Fase 4.5) | aceptada | `RFC-0007-ag-domains-alcance.md` |
| 0008 | Politica de idioma (ingles canonico) | aceptada | `RFC-0008-politica-de-idioma.md` |
| 0009 | ag-mail native outbound MTA: alcance, stack y plan por fases (Fase 4.6) | aceptada | `RFC-0009-ag-mail-native-mta.md` |
| 0010 | ag-mail superficie de envio sin marcas comerciales | aceptada | `RFC-0010-ag-mail-superficie-sin-marcas.md` |
| 0011 | ag-domains control plane: attachment y serving (extiende RFC-0007) | aceptada | `RFC-0011-ag-domains-control-plane.md` |
| 0012 | ag-workers: motor de ejecucion en segundo plano (Fase 4.6-D) | aceptada | `RFC-0012-ag-workers.md` |
| 0013 | Confinamiento de filesystem por capabilities en ag-storage | aprobada | `RFC-0013-capability-filesystem-confinement.md` |
| 0014 | Validacion @regex ejecutable en Rust (generador DSL) | aceptada | `RFC-0014-regex-runtime-validation.md` |
| 0015 | ag-registrars: compra/transferencia/renovacion de dominios (Fase F) | aceptada | `RFC-0015-ag-registrars-design.md` |
| 0016 | eTLD+1 via Public Suffix List en ag-domains (DEBT-024) | aceptada | `RFC-0016-eldp1-public-suffix-list.md` |
| 0017 | ag-workers: re-drive/purga masiva de DLQ por filtro | aceptada | `RFC-0017-ag-workers-bulk-dlq.md` |
| 0018 | Experiencia interactiva de terminal (TUI) para la CLI `ag` | aceptada | `RFC-0018-interactive-terminal-experience.md` |
| 0019 | Arquitectura por capas de los proyectos generados y alcance de interoperabilidad | aceptada | `RFC-0019-project-layered-architecture.md` |
| 0020 | Variables de entorno declaradas y gestion de secretos | aceptada | `RFC-0020-environment-and-secrets.md` |
| 0021 | Instalacion en servidor con un comando, consola remota e interfaz de maquina | aceptada | `RFC-0021-server-install-remote-console-and-machine-interface.md` |
| 0022 | Guardarrailes de mantenibilidad: reglas verificables contra la degradacion | aceptada | `RFC-0022-maintainability-guardrails.md` |
| 0023 | Funciones serverless | aceptada | `RFC-0023-serverless-functions.md` |
| 0024 | Interfaz de maquina MCP compartida con agentes por capacidades | aceptada | `RFC-0024-mcp-shared-machine-interface.md` |
| 0025 | ORM tipado, RLS y multi-tenancy generados desde el DSL para ag-data (Fase 5+) | aceptada como diseno; implementacion diferida hasta Fase 5 y cierre de la puerta (ADR-0020, 2026-11-22) | `RFC-0025-ag-data-orm-rls.md` |
| 0027 | chumsky 0.9 a 0.13/0.14 para el parser de Anti-DSL (enmienda de RFC-0003) | aceptada; recomienda diferir, mantener 0.9.3 (ADR-0021, 2026-11-22) | `RFC-0027-chumsky-parser-upgrade.md` |
| 0028 | Verificacion en CI del feature opt-in `otlp` de ag-observe | aceptada | `RFC-0028-otlp-feature-ci-verification.md` |
| 0026 | Sustitucion de chrono: evaluacion y plan de migracion bloqueado | propuesta, bloqueada por la decision OAuth2 de RFC-0029 | `RFC-0026-chrono-replacement.md` |
| 0029 | Sustituir oauth2 5.0.0 por un cliente nativo de Authorization Code + PKCE en ag-auth | propuesta | `RFC-0029-oauth2-chrono-unblock.md` |
| 0030 | Signed release manifest and trust anchor (diseno; no cierra #141) | propuesta; no aceptada | `RFC-0030-signed-release-manifest-and-trust-anchor.md` |
| 0031 | Actualizacion del runner macOS en CI (issue #186) | aceptada | `RFC-0031-macos-runner-refresh.md` |
| 0032 | Alpha release process: version `0.1.0-alpha.1`, cuatro targets, checksums sin firma (no cierra #141) | aceptada (BDFL, periodo de comentarios dispensado) | `RFC-0032-alpha-release-process.md` |
| 0033 | Domain-to-deploy loop: contrato de dominio, entorno y visibilidad para hacer reachable un proyecto generado (ola 8, solo diseno) | propuesta | `RFC-0034-domain-to-deploy-loop.md` |
| 0033 | Renderizado local de artefactos de despliegue con `ag deploy plan\|apply\|status` | propuesta; alcance local autorizado en wave 7, NO autoriza el deploy cloud de Fase 5 | `RFC-0034-local-deploy-artifact-rendering.md` |
| 0035 | Opt-in CI check for independent DKIM verification (#178, no cierra la puerta de buzon real) | aceptada (periodo de comentarios dispensado) | `RFC-0035-independent-dkim-ci.md` |
| 0036 | Gate de fuzzing de 24 horas programado en runner propio (issue #156) | aceptada (BDFL, periodo de comentarios dispensado); no cierra la issue #156 | `RFC-0036-scheduled-fuzz-24h-gate.md` |
| 0037 | Diagnostic error contract | aceptada (BDFL, 2026-11-22) | `RFC-0037-diagnostic-error-contract.md` |
| 0038 | Observability scopes and component health in ag-observe | aceptada (BDFL, 2026-11-22) | `RFC-0038-observability-scopes-and-health.md` |
| 0039 | Load, saturation and installation measurement protocol | aceptada (BDFL, 2026-11-22) | `RFC-0039-load-saturation-and-install-measurement.md` |
