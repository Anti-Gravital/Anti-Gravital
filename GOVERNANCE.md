# GOVERNANCE - Modelo de gobernanza de Anti-Gravital

Este documento describe el modelo de gobernanza del proyecto
Anti-Gravital y su plan de transicion. Es complemento de la seccion 17
del maestro `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md`, cuya
copia verbatim vive en `docs/architecture/17-gobernanza-open-source.md`.

## Estado actual: BDFL

El proyecto opera bajo un modelo BDFL (Benevolent Dictator For Life)
acotado en el tiempo. El mantenedor inicial es Angel Nereira, en
nombre de Gravital Labs, division open source de Nereira Technology
and Business Solutions, Republica de Panama.

Responsabilidades del BDFL:

- Custodia de los documentos maestros.
- Aprobacion final de RFC durante Fase 0 a Fase 3.
- Designacion del comite tecnico cuando llegue Fase 4.
- Veto temporal en casos de seguridad o de derivacion de scope.
- Liberacion de versiones hasta la 0.5 inclusive.

El BDFL no puede:

- Modificar la licencia.
- Cerrar el codigo.
- Aceptar features que contradigan los maestros sin abrir RFC publica.
- Ignorar a un comite tecnico una vez formado.

## Comite tecnico

A partir de Fase 4 se constituye un comite tecnico con al menos cinco
miembros activos, no todos pertenecientes a Gravital Labs. El comite
gobierna a partir de Fase 5.

Responsabilidades del comite:

- Aprobacion de RFC.
- Aprobacion de roadmap.
- Resolucion de disputas tecnicas.
- Promocion a versiones mayores.
- Promocion de mantenedores de crates individuales.

Quorum: dos tercios para decisiones ordinarias, unanimidad para
promocion a 1.0 segun la regla de salida de Fase 10.

## Transicion BDFL a comite

| Hito                          | Cambio de gobernanza                                  |
| --- | --- |
| Fase 0 a 3                    | BDFL puro.                                            |
| Fase 4                        | Se forma el comite tecnico (mas miembros que BDFL).   |
| Fase 5 a 9                    | Comite tecnico vota; BDFL conserva veto de seguridad. |
| Fase 10                       | Comite tecnico es soberano; BDFL pasa a miembro mas.  |

## Proceso RFC

Toda decision tecnica importante requiere RFC. El proceso vive en
`docs/rfc/README.md` y la plantilla en `docs/rfc/template.md`. Resumen:

1. Borrador con numero `RFC-XXXX-titulo-corto.md` en una pull request.
2. Periodo minimo de comentarios de 7 dias calendario.
3. Decision: aceptado, rechazado, diferido.
4. Si aceptado, se ejecuta la implementacion en pull requests
   subsiguientes que referencian la RFC.

## Decisiones arquitectonicas (ADR)

Las decisiones arquitectonicas se persisten como ADR en `docs/adr/`.
Un ADR registra contexto, decision, consecuencias y alternativas
consideradas. No reemplaza la RFC; la complementa una vez la decision
es estable.

## Mantenedores de crates

Cada crate del workspace puede tener mantenedores propios. La lista se
mantiene en `.github/CODEOWNERS`. El BDFL designa los mantenedores
iniciales; el comite tecnico designa los subsiguientes.

## Releases

Cadencia objetivo:

- Pre-0.5 (Fase 0 a 4): releases por hito.
- 0.5 a 1.0 (Fase 5 a 10): releases mensuales o por hito.
- Post-1.0: releases minor trimestrales, releases patch segun
  necesidad de seguridad.

Las reglas duras de promocion estan en
`docs/roadmap/fase-10-endurecimiento-y-1.0.md`.

## Marca y dominios

La marca Anti-Gravital y los dominios oficiales se gestionan desde
Gravital Labs. El comite tecnico no tiene autoridad sobre la marca,
pero si sobre el codigo. Esta separacion existe para proteger ambos
intereses.

## Conducta

Toda interaccion en el proyecto se rige por `CODE_OF_CONDUCT.md`. El
incumplimiento es causa de expulsion del proyecto, decidida por el
BDFL en Fase 0 a 3 y por el comite a partir de Fase 4.
