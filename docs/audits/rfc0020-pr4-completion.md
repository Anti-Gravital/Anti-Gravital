# RFC-0020 PR 4 - cierre honesto del alcance de configuracion declarada

- Fecha: 2026-11-23
- Decision: **PR 4 COMPLETO para configuracion de servicio declarable** (seis
  crates). `ag-cli` NO se migra en este PR: sus seis puntos de lectura directa
  son contexto de sesion, reporte de origen del propio loader o credenciales
  opt-in / feature-gated leidas donde se usan; ninguna es configuracion de
  aplicacion declarable, todas quedan fuera del alcance de PR 4 por diseno.
- Sin cambio de codigo de produccion, sin API publica nueva, sin default
  modificado, sin feature modificado y sin RFC/ADR nueva (RFC-0020 ya aceptada;
  A2 esta `implementable now` bajo esa RFC).

## 1. Que cubre PR 4 y por que `ag-cli` no es una migracion de configuracion

RFC-0020 seccion 5, PR 4: *"migrate the modules from direct `std::env::var`
reads to injected typed configuration, one crate per pull request"*. A2
(`docs/audits/oportunidades-arquitectura.md:99-107`) concreta el diseno: cada
crate declara sus claves con `EnvDeclaration` en su propio `config.rs` y las
carga en la raiz de composicion mediante `ShieldConfig::from_path_with_env` /
`EnvLoader::load_process`, con un PR por crate y tests de paridad de defaults.

Los seis crates ya migrados son **crates de servicio/biblioteca**: tienen un
`config.rs` con una estructura de configuracion, una funcion `*_env_declarations`
y un `from_env`/`apply_env` que se invoca en una raiz de composicion. Ese es el
objeto de PR 4.

`ag-cli` no es ese caso. Es la raiz de composicion y el despachador de
comandos: no tiene estructura de configuracion, no tiene `from_env` y
`main.rs` no contiene ninguna llamada global a `EnvLoader` (el unico uso de
`EnvLoader` esta dentro del comando `ag env`, PR 2, ya fusionado). No existe
una configuracion de servicio declarable que migrar. Forzar un loader global
declarado en `main.rs` seria:

- un cambio de arquitectura no documentado en el diseno aceptado (CLAUDE.md
  regla 5: bloqueo arquitectonico; regla 28: RFC para alterar limites o anadir
  comandos; regla 8: no adelantar fases);
- una abstraccion sin proposito runtime, del tipo que rechaza la regla 21
  (anti-complejidad: abstracciones prematuras, sistemas magicos, acoplamiento
  implicito);
- una segunda declaracion de las mismas credenciales que ya pertenecen a su
  servicio (`DATABASE_URL` al backend durable de `ag-workers`), lo que
  contravendria el objetivo de A2 de *un unico esquema declarado y auditable*;
- un riesgo real de romper la deteccion de TUI (RFC-0018 principio 4: `ag` nunca
  es interactivo por sorpresa) y los comandos feature-gated.

No existe un subconjunto seguro que migra sin empeorar la arquitectura: las
lecturas de `interactive.rs` usan semantica `OsString` (valores no UTF-8,
longitud cero, comparacion exacta con `dumb`) que un `EnvType::String` del loader
no preserva byte a byte; la lectura de `env.rs` es el propio reporte de origen
del loader del PR 2; y las credenciales de `workers.rs` y `dns/provider.rs`
deben leerse en el punto de uso para conservar su caracter opt-in y feature-gated.

## 2. Clasificacion de los seis puntos de lectura de `ag-cli`

| Lectura | Archivo:linea | Que es | Declarable como config de servicio? | Decision |
| --- | --- | --- | --- | --- |
| `var_os(name)` en el closure `set` -> `CI`, `AG_NO_TUI`, `NO_COLOR` (`set` definido en :41, usado en :46, :47, :49) | `crates/ag-cli/src/interactive.rs:41,46,47,49` | `Session::from_environment` decide si el comando puede interactuar (RFC-0018 principio 4). Variables estandar definidas por herramientas de CI y terminales; se leen del entorno real incluso fuera de un proyecto. | **No.** Es contexto de sesion, no configuracion de aplicacion. Requiere semantica `OsString` no UTF-8 y chequeo de cadena vacia, no un `String` tipado con default. Declararlas en `ag.toml` + loader global cambiaria el comportamiento de `ag` sin proyecto. | Mantener lectura directa; fuera de alcance de PR 4 por diseno. |
| `var_os("TERM") == "dumb"` | `crates/ag-cli/src/interactive.rs:48` | Misma deteccion de capacidad de terminal que la fila anterior. | **No.** Mismo motivo: contexto de sesion y comparacion exacta de `OsString`. | Mantener lectura directa; fuera de alcance de PR 4 por diseno. |
| `var_os(declaration.name())` (solo presencia) | `crates/ag-cli/src/commands/env.rs:240` | `list_data` del comando `ag env list` (PR 2, ya fusionado) informa el origen `process` / `default` / `none` de cada declaracion. Comprueba presencia; nunca imprime valores. | **No.** Es el reporte de origen del propio loader, inherentemente un chequeo de presencia, y ya pertenece al PR 2 completado. Migrarlo seria circular. | Mantener lectura directa; parte de PR 2, no de PR 4. |
| `var("DATABASE_URL")` | `crates/ag-cli/src/commands/workers.rs:102` | `workers_backend()`, tras `#[cfg(feature = "workers-runtime")]`: credencial del backend durable leida en el comando que la necesita. | **No.** Credencial de feature opt-in, leida donde se usa. La migracion de `ag-workers` excluyo deliberadamente `DATABASE_URL` de sus declaraciones (verificado: ausente en `crates/ag-workers/src/config.rs`). Cargarla en arranque exigiria la credencial para toda invocacion de `ag`, incluidas `--help` o `ag new`. | Mantener lectura directa feature-gated; fuera de alcance de PR 4. |
| `var("DATABASE_URL").is_ok()` | `crates/ag-cli/src/commands/workers.rs:335` | `cmd_workers_doctor()` sondea la presencia del mismo backend opcional; la configuracion del crate ya pasa por `ag_workers::WorkersConfig::apply_env()` (migrada en PR 4). | **No.** Mismo motivo: sonda de presencia de un backend feature-gated, no config de arranque. | Mantener lectura directa feature-gated; fuera de alcance de PR 4. |
| `var("AG_CLOUDFLARE_TOKEN")` | `crates/ag-cli/src/commands/dns/provider.rs:32` | `apply_provider()`, solo con opt-in explicito `--provider cloudflare`. El propio codigo documenta: *"no credential is read from the manifest"*. | **No.** Credencial de proveedor opt-in, deliberadamente tomada del entorno y nunca de un manifiesto versionable; declararla abriria una via de fuga y crearia un segundo esquema. | Mantener lectura directa en el punto de uso; fuera de alcance de PR 4. |

Notas de alcance: las demas llamadas a `std::env` en `ag-cli` (`temp_dir()`,
`current_dir()`, `set_var`/`remove_var` dentro de `#[cfg(test)]`, y
`.env("BIND", bind)` en `cmd_dev`) no son lecturas de configuracion declarable.
En particular `cmd_dev` toma `--bind` del argumento CLI (default
`127.0.0.1:8080` en `cli.rs:119-124`) y lo pasa al proceso hijo; no lee `BIND`
del entorno.

## 3. Los seis crates migrados y su evidencia

| Crate | Declaracion (`config.rs`) | Paridad (`config_tests.rs`) | Guard de dependencias | Nota de estado RFC |
| --- | --- | --- | --- | --- |
| `ag-storage` | `storage_env_declarations`, 17 claves; secretos `STORE_TOKEN`, `STORAGE_JWT_PUBLIC_KEY`, `AWS_SECRET_ACCESS_KEY`, `STORAGE_SIGN_SECRET` | 42 lineas de asercion/paridad | `scripts/check-dependency-direction.sh:82` (`storage -> core`) | wave 22 |
| `ag-observe` | `observe_env_declarations`, 6 claves; `SERVICE_NAME` queda directo en `layer::build_resource` (gate `otlp`, cambio de API publica vedado) | 27 lineas de asercion/paridad | `:79` (`observe -> core`) | wave 23 |
| `ag-workers` | `workers_env_declarations`, 6 claves fijas + 2 plantillas por cola `_MIN`/`_MAX` | 72 lineas de asercion/paridad | `:83` (`workers -> core`) | wave 24 |
| `ag-realtime` | `realtime_env_declarations`, 11 claves; `NATS_TLS_CERT`/`NATS_TLS_KEY` son rutas, no secretos | 62 lineas de asercion/paridad | `:80` (`realtime -> core`) | wave 25 |
| `ag-auth` | `auth_env_declarations`, 8 claves; 4 secretos con `expose()` explicito | 37 lineas de asercion/paridad | `:64` (`auth -> core`) | wave 26 |
| `ag-cache` | `cache_env_declarations`, 6 claves; `REDIS_URL` marcado `secret(true)` con canario de redaccion | 40 lineas de asercion/paridad | `:67` (`cache -> core`, ya presente) | wave 27 |

Cada crate conservo su firma publica, defaults, orden de resolucion
(entorno de proceso, luego default; RFC-0020 seccion 4.2) y features; uso
`load_process()` y una fuente inyectada solo en `#[cfg(test)]` para eliminar
las carreras de `set_var` (audit A1). Ningun cambio de este cierre toca esos
archivos.

## 4. Declaracion exacta de cierre

**RFC-0020 PR 4 is complete for declarable service configuration: the six
service crates (`ag-storage`, `ag-observe`, `ag-workers`, `ag-realtime`,
`ag-auth`, `ag-cache`) now load their declared keys through
`ag_core::EnvLoader` with byte-identical defaults, resolution order, public
APIs and feature gates. `ag-cli` is not migrated in PR 4 by design: its six
direct environment reads are runtime session/capability detection
(`interactive.rs`), the `ag env list` loader's own source report (`env.rs`,
PR 2), and opt-in / feature-gated credentials read where they are used
(`workers.rs` `DATABASE_URL`, `dns/provider.rs` `AG_CLOUDFLARE_TOKEN`). None is
declarable application configuration; a global declared loader in `main.rs`
would be an undocumented architecture change (CLAUDE.md rules 5, 8, 21 and 28)
and would break TUI detection and feature-gated commands. No production code,
public API, default or feature changed in this close.**

## 5. Verificacion

Resultados reales de este cierre (ver descriptor
`docs/kaizen/pr-drafts/community28-env-cli-decision.md`). Comandos ejecutados en
el worktree `community28/env-cli-decision`:

- `git diff --check` - exit 0, sin salida.
- Escaneo de contenido sobre los cinco archivos tocados: sin secretos, sin
  atribucion a IA, sin emojis, sin supresiones S-10 nuevas, sin cambios bajo
  `crates/`. Un primer patron amplio coincidio con la palabra inglesa "only"
  (`presence-only`, `progress only`); el escaneo preciso con limites de palabra
  devolvio "NONE (OK)". No se anadio ninguna supresion.
- `bash scripts/kaizen check` - exit 0. Salida exacta:
  `ok: descriptores validados` / `ok: politicas de contenido` /
  `ok: formato` / `ok: lint` / `ok: tests` / `ok: deuda 0 <= 0` /
  `ok: kaizen check: todo en verde`.
- `bash scripts/ag-check` - exit 0. Salida exacta: `PASS: descriptors`,
  `PASS: fmt`, `PASS: clippy`, `PASS: test`, `PASS: audit`, `PASS: deny`,
  `PASS: masters`, `PASS: prohibited`, `Summary: PASS=8 FAIL=0 SKIP=0`,
  `ag-check: all selected checks passed.`
- `RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps` - exit 0.
  Termino con estas lineas exactas de la salida:

  ```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.94s
     Generated /home/angel-nereira/Github-local-angelnereira/worktrees28/env-cli-decision/target/doc/ag_ai/index.html and 28 other files
  ```

  Documento `ag-cli` y el resto del workspace sin advertencias.

## 6. Fuentes verificadas (source_ids)

- `docs/rfc/RFC-0020-environment-and-secrets.md:13-27,39-153,297-304`
  (estado de implementacion y seccion 5 PR 4).
- `docs/audits/oportunidades-arquitectura.md:88-125` (A2, orden de migracion,
  diseno y criterio de descarte).
- `CLAUDE.md:143-160` (regla 5), `201-212` (regla 8), `506-523` (regla 21),
  `698-727` (regla 28).
- `crates/ag-cli/src/interactive.rs:38-61`;
  `crates/ag-cli/src/commands/env.rs:233-263`;
  `crates/ag-cli/src/commands/workers.rs:99-108,325-346`;
  `crates/ag-cli/src/commands/dns/provider.rs:17-49`;
  `crates/ag-cli/src/main.rs:64-90`;
  `crates/ag-cli/src/cli.rs:119-124,246-264`;
  `crates/ag-cli/src/commands/project.rs:205-230`.
- `crates/ag-{storage,observe,workers,realtime,auth,cache}/src/config.rs` y
  `config_tests.rs`; `scripts/check-dependency-direction.sh:64-84`.
