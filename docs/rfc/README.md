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
| 0015 | ag-registrars: compra/transferencia/renovacion de dominios (Fase F) | propuesta | `RFC-0015-ag-registrars-design.md` |
| 0016 | eTLD+1 via Public Suffix List en ag-domains (DEBT-024) | propuesta | `RFC-0016-eldp1-public-suffix-list.md` |
| 0017 | ag-workers: re-drive/purga masiva de DLQ por filtro | propuesta | `RFC-0017-ag-workers-bulk-dlq.md` |
