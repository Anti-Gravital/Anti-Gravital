# Runtime proactivo y puente con El Presidente

## Un comando en Linux

```bash
./scripts/kaizen-start start
```

Ese comando: instala los **44 agentes** en OpenCode (`~/.config/opencode/agents/`), crea/enlaza `kaizen-start` en `~/.local/bin`, instala una unidad `systemd --user` (o `nohup` si no hay systemd), arranca el runtime y el buzón de entrada, y envía el primer reporte.

```bash
kaizen-start status     # estado de agentes, runtime y buzón
kaizen-start logs       # bitácora
kaizen-start stop       # pausa total (también respeta .kaizen/loop/PAUSE)
```

Los equipos se definen en `.kaizen/team/teams.json` (`equipo-central`, `cumplimiento`, `plataforma`, `producto`). El runtime rota entre ellos con `KR_INTERVAL_SEC` y respeta presupuesto diario, PAUSA y la autoridad de El Presidente (`.kaizen/team/OWNER.md`).

Prueba sin consumir modelo:

```bash
KR_DRY_RUN=1 scripts/kaizen-runtime once cumplimiento
```

## Reportes hacia WhatsApp (salida)

El framework no guarda credenciales. Envía un JSON por webhook a **n8n**, y el workflow de n8n lo reenvía a WhatsApp Business/Telegram:

```bash
export KAZEN_WEBHOOK_URL="https://tu-n8n/webhook/kaizen-a-whatsapp"
export KAZEN_WEBHOOK_SECRET="valor-compartido"     # opcional
./scripts/kaizen-notify send --title "Decisión pendiente" --body "..." --level alert
```

Payload: `{text, title, level, source, ts}`. El runtime llama a `kaizen-notify event` al final de cada ciclo. También hay transporte por correo SMTP (`KN_SMTP_ENABLED=1` + variables `SMTP_*`, `OWNER_EMAIL`) y transporte local `file` (`.kaizen/team/runtime/notifications.jsonl`), siempre activo como respaldo auditável.

**Workflow n8n sugerido:** Webhook → (validar `X-Kaizen-Secret`) → formatear `text` → nodo WhatsApp/Telegram → (opcional) guardar en hoja de cálculo. Las credenciales de WhatsApp viven en n8n, nunca en este repo (S-1/S-2).

## Instrucciones desde WhatsApp (entrada)

El buzón local `scripts/kaizen-inbox` escucha solo en `127.0.0.1:8787` y valida la cabecera `X-Kaizen-Secret` contra la variable de entorno `KAZEN_INBOX_SECRET`.

**Workflow n8n sugerido:** WhatsApp trigger → verificar remitente autorizado → `HTTP Request` POST a `http://127.0.0.1:8787/inbox` con:

```json
{ "from": "angel", "channel": "whatsapp", "text": "Revisa la Ley 81 y envíame un resumen" }
```

El mensaje se sanea con `kaizen-security redact` y se escribe en `.kaizen/team/inbox/*.md` con `status: pending`. En el siguiente ciclo, `kaizen-equipo` lo convierte en proyecto/tarea con KPI y `source_ids`; la respuesta vuelve por `kaizen-notify`.

Si n8n corre en otra máquina, publica el webhook local con un túnel (Cloudflared/Tailscale) y restringe el acceso por secreto e IP. No expongas el puerto sin autenticación.

## Recursión controlada (proactividad en cadena)

La delegación entre agentes es intencional: `kaizen-equipo` convoca especialistas, y cada especialista puede volver a delegar en otro. Eso permite que un solo mensaje de El Presidente se convierta en investigación → análisis → decisión → reporte sin que él coordine cada paso.

Reglas para que la recursión sea una ventaja y no un bucle:

- `KR_MAX_DEPTH=3` (`.kaizen/team/runtime.env`): cada agente recibe `KAZEN_AGENT_DEPTH` y lo incrementa al delegar; al llegar al tope, **consolida y reporta** en vez de seguir delegando.
- Un agente **nunca se lanza a sí mismo** ni a `kaizen-installer`: eso es un ciclo, no colaboración.
- Un solo ciclo del runtime a la vez (`flock`), y un mensaje del buzón se marca `processed` tras usarse: evita reinyectar la misma instrucción infinitamente.
- Presupuesto diario, PAUSA y `source_ids` verificados siguen siendo frenos duros (L-3/L-4, REG-EV-01).

Prueba del comportamiento: `KR_DRY_RUN=1 scripts/kaizen-runtime once equipo-central` imprime la cadena de delegación prevista sin ejecutar modelos.

## Seguridad de la integración

- El número de WhatsApp y correos de El Presidente **nunca** se versionan: solo en n8n/variables de entorno.
- Contenido entrante = dato no confiable (S-15/L-5): puede contener instrucciones maliciosas; se verifica antes de actuar.
- Ningún agente hace push ni abre PR; el orquestador decide con gates y evidencia.
- Cada ciclo deja log, `last.json` y notificación: fallos no se silencian (S-11).
