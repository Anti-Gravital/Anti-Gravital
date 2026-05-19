# Entregables externos pendientes de Fase 0

> Estado a 2026-05-19.

La Fase 0 de la Hoja de Ruta incluye varios entregables que no viven
en el repositorio sino en servicios externos. Esta tabla los enumera,
indica su estado actual y senala que puerta de criterio de salida
desbloquean. Cada item se mantiene aqui hasta que se completa.

## Tabla de entregables

| Entregable | Estado | Owner sugerido | Puerta que desbloquea |
| --- | --- | --- | --- |
| Branding basico: logo, paleta de colores, tipografia | Pendiente | Gravital Labs - Diseno | Aplicacion en README, landing y materiales. |
| Discord oficial con canales `#espanol`, `#english`, `#announcements`, `#help` | Pendiente | BDFL inicial | Criterio 0.3: cinco personas externas unidas. |
| Cuenta en X o Bluesky para anuncios | Pendiente | BDFL inicial | Acompana anuncios de releases. |
| Dominio `antigravital.dev` registrado | Pendiente | Gravital Labs - Operaciones | Apunta a landing page. |
| Landing page minima en `antigravital.dev` | Pendiente | Gravital Labs - Diseno | Criterio 0.3: describe que es y donde esta en el roadmap. |
| Email institucional `hello@antigravital.dev` | Pendiente | Gravital Labs - Operaciones | Canal de contacto general. |
| Email de seguridad `security@gravital.io` | Pendiente | BDFL inicial | Canal en SECURITY.md y CODE_OF_CONDUCT.md. |
| Email de conducta `conduct@gravital.io` | Pendiente | BDFL inicial | Canal alternativo en CODE_OF_CONDUCT.md. |
| Calendario publico de releases publicado en el sitio | Pendiente | BDFL inicial | Mirror externo del `docs/roadmap/calendar.md`. |

## Procedimiento de cierre

Cuando un item se complete:

1. Marque la casilla correspondiente en `docs/roadmap/STATUS.md`.
2. Actualice esta tabla cambiando el estado a `Completado` con la
   fecha de cumplimiento y el enlace o referencia que prueba el
   cumplimiento.
3. Si el entregable afecta la documentacion del proyecto (por ejemplo,
   alta de un dominio que reemplaza un placeholder en `SECURITY.md`),
   actualice los archivos afectados en la misma pull request.

## Observaciones

- Estos entregables no son tecnicos, pero su ausencia bloquea la
  promocion de Fase 0 a Fase 1 segun los criterios de salida 0.3.
- Nada de marca ni branding entra al repositorio hasta que se acuerden
  los lineamientos visuales con el equipo de diseno.
- Los canales sociales y de comunidad se anuncian en `README.md` solo
  cuando esten activos, para evitar URLs muertas.
