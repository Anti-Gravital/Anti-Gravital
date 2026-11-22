# Previsión realista de capacidades y evolución de la organización de agentes

Fecha: 2026-11-21 | Estado: vivo, se revisa cada trimestre con ADR (K-1/K-9)

## 1. Qué pueden hacer bien los modelos hoy (2026)

Con **roles acotados, reglas explícitas, contratos escritos y evidencia verificable**, los modelos actuales son fiables para: tareas cortas y reversibles, lectura/escritura de archivos con límites, pruebas reproducibles, revisión con checklist, propuestas con evidencia y resolución de disputas en 2 turnos con árbitro. Es lo que implementa este repo.

## 2. Qué NO se les puede delegar hoy (aunque la autonomía sea 3)

- Planificar a largo plazo sin revalidación: la memoria en archivos (L-2) es más fiable que la "intuición" de una sesión.
- Decisiones sin evidencia reproducible: la palabra del agente no cuenta como prueba (K-10/L-10).
- Seguridad, pagos, autenticación, migraciones destructivas, borrado de datos y modificar el propio loop: siempre humano (L-3/L-4).
- Confianza ciega en telemetría, issues, web o buzones: son datos no confiables (L-5/S-15).
- "Organizaciones autónomas" tipo fictional: un modelo no sostiene por sí solo continuidad, responsabilidad ni criterio de negocio sin frenos externos.

## 3. Por qué esta guía está diseñada así

Ante esa realidad, el diseño no es "un superagente que dirige todo", sino **muchos agentes estrechos + orquestador + puertas**: 17 roles técnicos (`.kaizen/swarm/ROSTER.md`) y los puestos de negocio del inventario (`.kaizen/org/`), incluidos `cumplimiento-panama` y `capacidades-negocio`. Cada uno tiene propósito, entradas, salidas, KPI y veto limitado. La proactividad es **proponer con evidencia**, no actuar sin control. La investigación en internet se gestiona con `.kaizen/org/RESEARCH.md` (fuentes oficiales, contraste y fecha de corte), porque un modelo sin cita ni fecha no es fiable para normativa ni estándares.

## 4. Camino de mejora continua de la IA (pronóstico acotado)

| Horizonte | Supuesto observable | Cambio previsto en esta guía |
| --- | --- | --- |
| Corto (2026-2027) | Tool-use más fiable, contextos más largos, coste por token a la baja | Más rondas por issue, más documentación alimentada por agente, pulsos proactivos más frecuentes. Mismo conjunto de frenos. |
| Medio (2027-2028) | Agentes sostienen tareas de varias horas con menos deriva; verificación automática más precisa | Autonomía `scoped` para más puestos, previo ADR y KPI en verde. El veto humano se mantiene en seguridad/dinero. |
| Largo | Modelos con memoria persistente fiable y calibración demostrable | Revisar si el orquestador puede reducirse; nunca eliminar evidencia, ratchet ni CODEOWNERS. |

Este pronóstico es **falsable**: si los KPI (tasa de revert, ruido de issues, coste por cambio, tiempo a PR verificado) empeoran, se pausa y se ajusta el diseño mediante PR humano, no subiendo la autonomía (ver señales de daño en KAIZEN-LOOP.md §11).

## 5. Cómo se refinan los agentes (el inventario es el mecanismo)

1. `kaizen-org list` → inventario vivo de puestos, departamentos, autonomía y KPI.
2. `kaizen-org copy <src> <dst> --name ...` → **copiar** un puesto y especializarlo (marketing general → marketing visual; finanzas → finanzas panameñas) sin tocar el original.
3. `kaizen-org edit <id>` / `save <id>` → **editar y guardar** con subida de versión, diario y ratchet.
4. `kaizen-org feed <id> <documento>` → **alimentar conocimiento**; el agente lo consume en su próxima iteración y el `digest` lo resume saneado.
5. Retro quincenal: hasta 3 mejoras a definiciones, con dueño, issue y fecha (K-9).

## 6. Límites honestos

Los modelos se equivocan con seguridad aparente; el escalado de autonomía no sustituye a pruebas, CI ni revisión. La telemetría y la web pueden intentar manipular al agente. Nada de lo anterior es asesoría legal, fiscal ni financiera: el puesto de `finanzas-panamas` produce análisis y propuestas verificables, nunca decisiones fiscales sin profesional humano responsable.
