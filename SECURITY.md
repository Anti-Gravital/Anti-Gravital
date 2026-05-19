# SECURITY - Politica de seguridad de Anti-Gravital

Anti-Gravital toma la seguridad como criterio de calidad de primera
clase. La regla 16 de `CLAUDE.md` y la seccion 15 del maestro de
arquitectura describen el modelo de seguridad completo del proyecto.

## Estado

El proyecto se encuentra en Fase 0 (Fundaciones y Gobernanza). Aun no
hay codigo funcional liberado, de modo que no existe una superficie de
ataque productiva. Esta politica se aplica desde ya para acostumbrar
al proyecto al proceso y para cubrir cualquier vulnerabilidad
potencial en el material publicado (workflows, plantillas, scripts de
soporte, dependencias de tooling, etc.).

## Versiones cubiertas

Mientras el proyecto esta en pre-1.0, solo la rama `main` recibe
parches de seguridad. A partir de la version 1.0 se publicara una
matriz de soporte por release.

## Como reportar una vulnerabilidad

Reporte privado y coordinado. NO abra un issue publico.

Mientras los canales oficiales de la organizacion estan en proceso de
habilitacion, los reportes se envian al mantenedor inicial a traves
del mecanismo de divulgacion privada de GitHub Security Advisories del
repositorio:

1. Abra una nueva advisory privada en la pestana Security del
   repositorio.
2. Describa: version o commit afectado, plataforma, pasos de
   reproduccion, impacto observado o estimado.
3. Si dispone de un parche, adjuntelo.

Direccion de correo de respaldo: `security@gravital.io`. Esta
direccion aparece como pendiente de habilitacion en
`docs/governance/external-deliverables.md`. Mientras no este activa,
la divulgacion privada de GitHub es la via primaria.

## SLA inicial

- Acuse de recibo: 72 horas calendario.
- Triage y clasificacion: 7 dias calendario.
- Solucion para vulnerabilidades altas o criticas: 30 dias calendario
  desde el triage.
- Divulgacion coordinada: tras publicar la correccion, con un plazo de
  cortesia minimo de 7 dias para que los integradores actualicen.

Estos plazos se ajustaran cuando exista trafico real y un equipo
dedicado, manteniendo siempre o mejorando estos minimos.

## Alcance

Se considera vulnerabilidad cualquier defecto que comprometa
confidencialidad, integridad o disponibilidad, en:

- Los crates publicados del workspace (`crates/*`).
- Los plugins oficiales (a partir de Fase 9).
- Los workflows de CI y los scripts de soporte (`tools/*`).
- Los importadores de migracion (`ag-migrate` a partir de Fase 7).
- Los SDKs generados (TypeScript, Dart, a partir de Fase 3/8).

Quedan fuera de alcance, aunque cualquier reporte sera atendido:

- Ataques contra servicios de terceros (GitHub, crates.io,
  Cloudflare, etc.).
- Reportes que dependen de configuraciones explicitamente etiquetadas
  como inseguras en la documentacion (modo `dev`, `--insecure`, etc.).
- Vulnerabilidades en dependencias upstream que ya tienen aviso
  publico; en estos casos lo correcto es abrir un issue de upgrade.

## Buenas practicas que sigue el proyecto

- `cargo audit` en CI sobre cada pull request.
- `cargo deny` sobre licencias, advisories y bans.
- Prohibicion de bloques `unsafe` no documentados.
- TLS 1.3 desde Fase 1 (rustls).
- Cripto basada en `ring` y `ed25519-dalek` en las capas que las
  necesitan.
- Defaults seguros: sin CORS wildcard, sin CSRF abierto, sin secretos
  hardcodeados.

## Reconocimiento

Los reporteros que sigan este proceso seran reconocidos en el
CHANGELOG y, cuando exista, en una pagina dedicada de hall of fame de
seguridad en la web oficial del proyecto. No usamos atribucion
automatica de herramientas IA.
