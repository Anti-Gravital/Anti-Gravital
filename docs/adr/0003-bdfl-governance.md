# ADR-0003 - Gobernanza BDFL inicial con transicion a comite tecnico

- Estado: aceptado
- Fecha: 2026-05-19
- RFC origen: ninguna; decision fundacional alineada con la seccion 17
  del maestro de arquitectura.

## Contexto

El proyecto necesita un modelo de gobernanza desde el dia cero. El
maestro de arquitectura describe una transicion BDFL a comite tecnico
a partir de Fase 4. Esta ADR fija como se opera mientras tanto y como
se ejecuta la transicion.

## Decision

- Fase 0 a Fase 3: BDFL (Benevolent Dictator For Life) acotado en el
  tiempo. Mantenedor inicial: Angel Nereira en nombre de Gravital
  Labs.
- Fase 4: se constituye un comite tecnico con al menos cinco miembros
  activos, no todos pertenecientes a Gravital Labs.
- Fase 5 en adelante: el comite tecnico decide; el BDFL conserva veto
  de seguridad.
- Fase 10: el comite ratifica la promocion a 1.0 por unanimidad. A
  partir de la 1.0 el BDFL pasa a ser un miembro mas del comite.
- Toda decision arquitectonica importante requiere RFC, durante
  cualquier fase. El BDFL no puede saltar el proceso.
- La marca Anti-Gravital y los dominios oficiales quedan bajo control
  de Gravital Labs.

## Consecuencias

Positivas:

- Velocidad de decision en las fases tempranas, cuando el proyecto
  necesita avanzar deprisa.
- Garantia explicita de transicion: el modelo BDFL no es permanente.
- Separacion clara entre control de codigo (comite) y control de
  marca (organizacion).

Negativas:

- Riesgo de centralismo durante las primeras fases. Mitigacion: RFC
  publica obligatoria.
- Necesidad de encontrar cinco miembros del comite para Fase 4.
  Mitigacion: identificacion temprana de candidatos en Fase 2 a 3.

## Alternativas consideradas

- Comite tecnico desde Fase 0: rechazado por inviable; aun no hay
  comunidad ni base contributiva.
- Fundacion externa (estilo Apache): rechazado por sobrecarga
  administrativa desproporcionada en pre-1.0.
- BDFL perpetuo: rechazado por incompatibilidad con la sostenibilidad
  open source.

## Notas

Detalle operativo en `GOVERNANCE.md` y en
`docs/architecture/17-gobernanza-open-source.md`.
