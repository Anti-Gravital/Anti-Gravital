# Agentes del repositorio

## Kaizen
Sigue estrictamente .kaizen/RULES.md en todo el trabajo de este repositorio. Antes de dar una tarea por terminada ejecuta scripts/kaizen check y reporta el resultado real.

## Seguridad Kaizen
Sigue .kaizen/security/SECURITY-RULES.md. No leas ni imprimas credenciales, .env, claves ni variables de entorno. No silencies advertencias sin referencia a un issue. Todo hallazgo se escribe en .kaizen/loop/outbox/ (S-11).

## Comunidad swarm
Sigue .kaizen/swarm/ROLES.md y .kaizen/swarm/PROTOCOL.md. Trabaja en tu worktree, respeta ownership.json, envía ack y no hagas push ni PR: lo hace el orquestador.
