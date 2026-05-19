# plugins/

Plugins WASI oficiales del ecosistema Anti-Gravital. Cada plugin se
empaqueta como modulo WebAssembly compilado a WASI con su archivo
`plugin.toml` de metadata.

## Plugins oficiales previstos en Fase 9

- `prometheus-exporter`
- `datadog-exporter`
- `sentry`
- `honeycomb-exporter`
- `slack-notifier`
- `discord-webhook`

## Estado

Fase 0: vacio. La implementacion del runtime de plugins llega en
Fase 9 (vease `docs/roadmap/fase-09-plugins-wasi.md`).

## Reglas

- Cada plugin tiene su propio `Cargo.toml` o su equivalente en TinyGo
  o AssemblyScript.
- Cada plugin trae su README con: que hace, que permisos requiere,
  como se instala, como se prueba.
- El ABI estable se define en RFC antes de Fase 9.
