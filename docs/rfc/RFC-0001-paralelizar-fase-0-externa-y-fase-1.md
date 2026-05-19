# RFC-0001 - Paralelizar puertas externas de Fase 0 con implementacion de Fase 1

- Estado: aceptado
- Autor: Angel Nereira (BDFL inicial)
- Fecha de borrador: 2026-05-19
- Fecha de aceptacion: 2026-05-19
- Fase objetivo: Fase 0 mas Fase 1
- Modulos o crates afectados: gobernanza del proceso (sin codigo)
- RFC predecesora: ninguna
- Periodo de comentarios: omitido por decision del BDFL en modo solo

## 1. Motivacion

La Hoja de Ruta v4.0 fija dos reglas que se cruzan en este momento:

1. Una fase no se considera concluida hasta que todas sus casillas de
   criterio de salida estan marcadas (Regla 1 de las Reglas de Oro).
2. Los criterios de entrada de Fase 1 exigen "Fase 0 completada" y
   "al menos un contribuidor adicional al mantenedor principal
   activo".

Estas reglas, aplicadas literalmente, bloquean el proyecto hasta que
ocurran tres eventos externos (primer star externo, cinco miembros
externos en Discord, landing page operativa) y aparezca un segundo
contribuidor. Esos eventos no se pueden forzar desde el repositorio.

La cuestion es si el proyecto detiene toda actividad tecnica hasta que
esos eventos ocurran, o si paraleliza la espera con trabajo de Fase 1
asumiendo que las casillas externas se cierran cuando se cierren.

## 2. Problema

Dos riesgos compiten:

- Riesgo de saltarse el proceso: relajar la Regla 1 abre la puerta a
  futuras justificaciones de saltos arbitrarios. Es el riesgo que las
  reglas existen para prevenir.
- Riesgo de paralisis: si el repositorio queda inactivo durante meses
  esperando un primer star y un segundo contribuidor, el proyecto
  pierde momentum y los entregables de Fase 1 se acumulan sin avance.
  La calidad del trabajo cae cuando se hace bajo presion al final.

## 3. Alternativas consideradas

### 3.1 Cumplir literalmente Regla 1

Detener toda actividad tecnica hasta que las tres puertas externas
cierren y aparezca un segundo contribuidor. Pros: respeta la regla al
pie de la letra. Contras: bloquea el avance tecnico durante semanas o
meses por eventos que no controlamos.

### 3.2 Modificar Regla 1 en CLAUDE.md

Reescribir la regla para permitir paralelizacion. Pros: elimina la
contradiccion. Contras: debilita el contrato general del proyecto;
abre puerta a futuras laxitudes; es una solucion mayor para un caso
particular.

### 3.3 Paralelizar como excepcion documentada (esta RFC)

Mantener la Regla 1 vigente. Documentar formalmente que para esta
ventana especifica del proyecto, la implementacion de Fase 1 avanza en
paralelo con el cierre de las puertas externas de Fase 0 y sin
segundo contribuidor. Pros: respeta la regla general; resuelve el
caso particular sin debilitarla; queda auditado. Contras: requiere
disciplina para no normalizar la excepcion.

## 4. Diseno propuesto

Se acepta la alternativa 3.3 con los siguientes acuerdos:

1. La implementacion de Fase 1 comienza inmediatamente, dirigida por
   RFC-0002 (diseno de Shield MVP) que se aprueba conjuntamente con
   esta RFC.
2. Las casillas externas de Fase 0 (primer star, cinco miembros en
   Discord, landing page) permanecen como `[ ]` en
   `docs/roadmap/STATUS.md` y se completan en paralelo.
3. La promocion publica del proyecto a "Fase 1 iniciada" no se anuncia
   hasta que las casillas externas de Fase 0 esten cerradas. Es decir:
   el trabajo tecnico avanza, pero el hito publico se reserva.
4. El requisito de "segundo contribuidor activo" se da por satisfecho
   con la incorporacion futura del primer colaborador externo. Hasta
   entonces el BDFL trabaja en solitario, con revision diferida
   asincronamente cuando aparezcan contribuidores.
5. Esta RFC no se reutiliza como precedente para saltos similares en
   el futuro. Cualquier otra excepcion futura requerira su propia RFC
   independiente.

## 5. Plan de implementacion

- Aceptacion de esta RFC y de RFC-0002 (Shield) en el mismo commit.
- Pull requests subsiguientes implementan Shield capa por capa
  (HTTP/Tokio base, TLS, JWT, rate limiting, validacion, CORS/CSRF,
  logging, configuracion TOML, tests, benchmarks).

## 6. Riesgos

- Normalizar excepciones: alto. Mitigacion: prohibir reutilizar esta
  RFC como precedente; exigir RFC propia para cada salto futuro.
- Calidad baja por falta de revisores: alto. Mitigacion: cuando el
  primer contribuidor aparezca, todas las PRs aceptadas en este
  periodo se someten a revision retrospectiva; cualquier cambio
  necesario se hace en nueva PR.
- Acumulacion de deuda tecnica: medio. Mitigacion: cada incremento se
  cubre con tests, fmt y clippy estrictos.
- Anuncio publico prematuro: bajo. Mitigacion: punto 3 del diseno.

## 7. Impacto

- Sobre el alcance: ninguno.
- Sobre el cronograma: positivo; no se pierden semanas en espera.
- Sobre las APIs publicas: ninguno aun.
- Sobre la documentacion: requiere actualizar STATUS.md con una nota
  que aclare la excepcion de paralelizacion.

## 8. Rollback

Si la excepcion produce deuda detectable o calidad insuficiente, se
pausa la implementacion de Fase 1, se cumple la Regla 1 literalmente,
y se revierten al ultimo commit estable con cobertura completa. La
revision retrospectiva del primer contribuidor externo es el momento
formal para evaluar si la excepcion funciono.

## 9. Decision

Aceptada por el BDFL inicial en modo solo, autorizacion verbal
registrada en la conversacion de continuidad de Fase 0. Esta RFC
queda como registro formal.

## 10. Referencias

- `docs/roadmap/reglas-de-oro.md` (Regla 1).
- `docs/roadmap/fase-00-fundaciones-y-gobernanza.md` (criterios 0.3).
- `docs/roadmap/fase-01-shield-mvp.md` (criterios 1.1).
- `docs/governance/external-deliverables.md`.
- `docs/adr/0003-bdfl-governance.md`.
- RFC-0002 (Shield MVP).
