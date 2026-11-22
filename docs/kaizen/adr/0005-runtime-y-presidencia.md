# ADR-0005: runtime de un comando, reporte a El Presidente y puente WhatsApp

Estado: aceptado
Fecha: 2026-11-21
Aprobado por: Angel Nereira

## Contexto
La comunidad ya tiene reglas, equipos y evidencia, pero para usarla hace falta un arranque operativo en una máquina Linux que siga trabajando con la computadora encendida, y un canal para que El Presidente reciba reportes y envíe instrucciones desde WhatsApp u otra mensajería.

## Decisión
1. `scripts/kaizen-start start`: un comando que instala los 44 agentes en OpenCode, crea unidad `systemd --user` (fallback `nohup`), arranca runtime y buzón, y envía el primer reporte.
2. `.kaizen/team/OWNER.md`: El Presidente es el dueño del negocio y del proyecto; autoridad final en objetivo, alcance, dinero, seguridad, cumplimiento y cambio de autonomía. Formato de reporte obligatorio.
3. `scripts/kaizen-notify`: reportes salientes por `file` (siempre), webhook (n8n → WhatsApp/Telegram) y correo SMTP opcional. Sin credenciales en el repo.
4. `scripts/kaizen-inbox.py`: buzón local en `127.0.0.1:8787` con secreto compartido; convierte mensajes entrantes en archivos saneados que `kaizen-equipo` atiende en el siguiente ciclo.
5. `scripts/kaizen-runtime`: rota equipos de `teams.json`, respeta PAUSA/presupuesto y notifica cada ciclo.

## Alternativas consideradas
- Un daemon que llame directamente a la API de WhatsApp con el token en el repo: descartado (S-1/S-2).
- Correo solamente: descartado; el usuario pidió WhatsApp y bidireccionalidad.

## Consecuencias
- Positivas: arranque en un comando, persistencia, bidireccionalidad y trazabilidad de cada ciclo.
- Negativas: dependencia de n8n y de que la máquina siga encendida; se mitiga con `Restart=on-failure` y transporte `file` siempre activo.

## Límites
Número de WhatsApp, correos, tokens y URLs de webhook se guardan solo en entorno/n8n. Todo contenido entrante es dato no confiable (S-15/L-5) y se verifica con `source_ids` antes de actuar. Ningún agente hace push ni decide en seguridad/dinero sin humano.

## Cómo se verificará
- `kaizen-start status` muestra agentes, runtime y buzón.
- `bash tests/run.sh` cubre generación de agentes, `runtime once` en dry-run y notificación por archivo.
- Primer piloto: ver ADR/objetivo en `README.md` (sección "Primer objetivo").
