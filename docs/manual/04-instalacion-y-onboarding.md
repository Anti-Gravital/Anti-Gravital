English | Espanol

---

# Chapter 04 — Installation and Onboarding

This chapter guides you from a blank machine to a running Anti-Gravital project.

## Prerequisites

| Requirement | Minimum version | Check                    |
|-------------|-----------------|--------------------------|
| Rust        | 1.99.1 (source channel only) | `rustc --version` |
| Git         | any modern      | `git --version`          |

Rust is required only for the explicit `AG_INSTALLER_CHANNEL=source` channel;
the default `release` channel installs a prebuilt binary. No other system
dependencies are required for the `rest` template.

## Install the `ag` binary

The alpha (`v0.1.0-alpha.2`) has no crates.io package. The installer's default
`release` channel downloads the versioned prebuilt binary for your platform and
verifies its SHA-256 before installing; nothing is compiled. The checksum gives
**integrity, not authenticity**: there is no signature (#141 is open and
RFC-0030 is a draft). Building from source remains available as an explicit
channel. Use one of the methods below, then confirm the version with
`ag --version`. See the
[alpha release notes](../releases/0.1.0-alpha.2.md) for scope and limitations.

### Linux / macOS

```bash
git clone https://github.com/Anti-Gravital/Anti-Gravital.git
cd Anti-Gravital
bash install.sh
```

By default the script downloads the versioned prebuilt binary and verifies its
SHA-256; it does not compile. On the explicit `source` channel it verifies the
Rust toolchain, builds the workspace in release mode and installs `ag` into the
Cargo bin directory (about 21 minutes and two builds on the recorded host).
Security note: the script does not pipe untrusted content into a shell. Before
running it, follow the canonical
[installation integrity procedure](../security/INSTALLATION_INTEGRITY.md).

### Windows (PowerShell)

```powershell
git clone https://github.com/Anti-Gravital/Anti-Gravital.git
cd Anti-Gravital
.\install.ps1
```

### Manual (any platform)

```bash
cargo install --path crates/ag-cli --locked
```

### Channels and progress display

The default `release` channel resolves the version from `Cargo.toml`
(`0.1.0-alpha.2`), downloads the prebuilt binary for the detected platform and
checks its SHA-256 against the release `checksums.txt` before installing.
Nothing is compiled and no toolchain is downloaded. The checksum is integrity,
not authenticity (no signature; #141 open, RFC-0030 draft), and there is **no
automatic fallback** to `source`. To build from source, opt in explicitly:

```bash
AG_INSTALLER_CHANNEL=source bash install.sh
```

Progress appears as a native six-stage view: resolve version, detect platform,
download, verify SHA-256, install and done. In CI or any non-interactive shell,
use headless mode for one stable `[ag-install] N/6 ...` line per stage:

```bash
AG_INSTALLER_PROGRESS=headless bash install.sh
```

Headless is also selected automatically when `CI` is set, `TERM=dumb` or
`NO_COLOR` is present, or stdout is not a TTY. The fail-closed checksum order is
identical in both modes: progress display never hides a verification failure,
skips a stage or suppresses the `AG_INSTALLER_CHECKSUMS` warning. The release
`checksums.txt` covers the binary assets but **not** the installer scripts;
see the
[installation integrity procedure](../security/INSTALLATION_INTEGRITY.md) for
`AG_INSTALLER_CHECKSUMS` and `AG_INSTALLER_BINARY_CHECKSUMS`.

### Verify

```bash
ag --version
ag --help
```

On a clean checkout of the tagged alpha, `ag --version` reports
`0.1.0-alpha.2`. Source checkouts between tags may report a different
version; build from the tag to reproduce the release exactly.

## Create a project

```bash
ag new my-api
```

If the terminal is interactive, you are prompted to choose a template
(`rest`, `realtime`, or `fullstack`). In CI or scripts the default is `rest`.

To skip the prompt:

```bash
ag new my-api --template realtime
```

## Start the development server

```bash
cd my-api
ag dev
```

The server listens on `127.0.0.1:8080` by default. `ag dev` passes
`BIND=127.0.0.1:8080` unless you set `--bind`; the generated templates use the
same loopback default and honour `BIND`. To expose the server in a container or
on a LAN, pass `--bind 0.0.0.0:8080` explicitly and treat it as network-exposed.
Install `cargo-watch` for automatic reloading on file changes:

```bash
cargo install cargo-watch
```

## Configure email (optional)

Set environment variables before running:

```bash
export AG_SMTP_HOST=smtp.example.com
export AG_SMTP_PORT=587
export AG_SMTP_USER=myuser
export AG_SMTP_PASS=secret
export AG_MAIL_FROM=noreply@example.com
```

Test the configuration:

```bash
ag mail test --to me@example.com
```

## Configure DNS / TLS (optional)

```bash
export AG_CLOUDFLARE_TOKEN=your-token
export AG_DNS_ZONE_ID=your-zone-id

# Check propagation
ag domains check --domain example.com

# Apply SPF / DKIM / DMARC records
ag domains sync --schema schema.ag
```

## Operate an attached domain

The local attachment workflow does not require provider credentials:

```bash
ag domains attach example.com --project my-api --edge-host edge.example.net
ag domains instructions example.com --edge-host edge.example.net
ag domains export-zone example.com --edge-host edge.example.net
ag domains status example.com
ag domains list
ag domains verify example.com
ag domains detach example.com
ag domains diagnose example.com --edge-host edge.example.net
```

State is stored in `.ag/domains.json` by default. Use `--state` to select
another file. Run `ag domains COMMAND --help` for required apex IPs, edge
targets and lifecycle options. The [domain CLI reference](../ag-domains/reference/cli.md)
documents the full native workflow; provider-backed DNS synchronization is
documented separately by `ag domains sync --help`.

`ag deploy`, `ag migrate`, and `ag plugin` are future commands and are not
available in the current binary.

### Domain-to-deploy loop

The [domain-to-deploy guide](../dns/README.md) expands this workflow into a
full loop: prerequisites, the `dev`/`staging`/`prod` environments, public vs
private visibility, pointing a custom domain at a running service, and
troubleshooting (propagation, ownership, wildcards and TLS). It distinguishes
the `ag domains` commands that exist in the alpha from the wave-8 commands
that are still planned. Provider tokens stay in environment variables, never
in the repository.

## Troubleshooting

| Problem | Likely cause | Fix |
|---------|-------------|-----|
| `ag: command not found` | `~/.cargo/bin` not in PATH | `export PATH="$HOME/.cargo/bin:$PATH"` |
| `error: package 'ag-cli' not found` | Running outside the repo | Run from the repo root |
| SMTP connection refused | Wrong host/port | Check `AG_SMTP_HOST` and `AG_SMTP_PORT` |
| `DNS record not found` | Propagation delay | Wait up to 48 h and re-run `ag domains check` |

---

# Capitulo 04 — Instalacion y Onboarding

> Nota: la seccion en ingles es canonica. Esta seccion puede estar
> desactualizada; si difiere, prevalece la version en ingles.

Este capitulo te guia desde una maquina en blanco hasta un proyecto
Anti-Gravital en ejecucion.

## Prerequisitos

| Requisito | Version minima | Verificacion             |
|-----------|----------------|--------------------------|
| Rust      | 1.99.1 (solo canal source) | `rustc --version`  |
| Git       | cualquier      | `git --version`          |

Rust se requiere solo para el canal explicito `AG_INSTALLER_CHANNEL=source`;
el canal `release` por defecto instala un binario precompilado.

## Instalar el binario `ag`

El alpha (`v0.1.0-alpha.2`) no tiene paquete en crates.io. El canal `release`
por defecto del instalador descarga el binario precompilado versionado para tu
plataforma y verifica su SHA-256 antes de instalar; no se compila nada. El
checksum da **integridad, no autenticidad**: no hay firma (#141 sigue abierto y
RFC-0030 es borrador). Compilar desde el fuente sigue disponible como canal
explicito. Usa uno de los metodos siguientes y confirma la version con
`ag --version`. Consulta las
[notas de release del alpha](../releases/0.1.0-alpha.2.md) para alcance y
limitaciones.

### Linux / macOS

```bash
git clone https://github.com/Anti-Gravital/Anti-Gravital.git
cd Anti-Gravital
bash install.sh
```

Por defecto el script descarga el binario precompilado versionado y verifica su
SHA-256; no compila. En el canal explicito `source` verifica el toolchain Rust,
compila el workspace en modo release e instala `ag` en el directorio binario de
Cargo (unos 21 minutos y dos builds en el host medido). Antes de ejecutarlo,
sigue el [procedimiento canonico de integridad](../security/INSTALLATION_INTEGRITY.md).

### Windows (PowerShell)

```powershell
git clone https://github.com/Anti-Gravital/Anti-Gravital.git
cd Anti-Gravital
.\install.ps1
```

### Manual (cualquier plataforma)

```bash
cargo install --path crates/ag-cli --locked
```

### Canales y visualizacion del progreso

El canal `release` por defecto resuelve la version desde `Cargo.toml`
(`0.1.0-alpha.2`), descarga el binario precompilado para la plataforma detectada
y comprueba su SHA-256 contra el `checksums.txt` del release antes de instalar.
No se compila nada ni se descarga ningun toolchain. El checksum da integridad,
no autenticidad (sin firma; #141 abierto, RFC-0030 borrador), y **no hay fallback
automatico** a `source`. Para compilar desde el fuente, hazlo explicito:

```bash
AG_INSTALLER_CHANNEL=source bash install.sh
```

El progreso se muestra como una vista nativa de seis etapas: resolver version,
detectar plataforma, descargar, verificar SHA-256, instalar y listo. En CI o
cualquier shell no interactivo usa el modo headless para obtener una linea
estable `[ag-install] N/6 ...` por etapa:

```bash
AG_INSTALLER_PROGRESS=headless bash install.sh
```

El modo headless tambien se selecciona automaticamente cuando esta definido
`CI`, `TERM=dumb` o `NO_COLOR`, o cuando stdout no es una TTY. El orden
fail-closed de la verificacion es identico en ambos modos: el progreso nunca
oculta un fallo, salta etapas ni suprime el warning de `AG_INSTALLER_CHECKSUMS`.
El `checksums.txt` del release cubre los binarios, pero **no** los instaladores;
consulta el [procedimiento canonico de integridad](../security/INSTALLATION_INTEGRITY.md)
para `AG_INSTALLER_CHECKSUMS` y `AG_INSTALLER_BINARY_CHECKSUMS`.

## Verificar

```bash
ag --version
ag --help
```

En un checkout limpio del alpha etiquetado, `ag --version` muestra
`0.1.0-alpha.2`. Los checkouts intermedios entre etiquetas pueden mostrar
otra version; compila desde la etiqueta para reproducir el release exacto.

## Crear un proyecto

```bash
ag new mi-api
```

Si el terminal es interactivo, se te pedira elegir una plantilla
(`rest`, `realtime`, o `fullstack`). En CI o scripts el default es `rest`.

## Iniciar el servidor de desarrollo

```bash
cd mi-api
ag dev
```

El servidor escucha en `127.0.0.1:8080` por defecto. `ag dev` pasa
`BIND=127.0.0.1:8080` salvo que uses `--bind`; los templates generados usan el
mismo default de loopback y respetan `BIND`. Para exponerlo en un contenedor o
LAN, pasa `--bind 0.0.0.0:8080` explicitamente y tratalo como expuesto a la red.

## Operar dominios adjuntos

Los comandos implementados son `attach`, `instructions`, `export-zone`,
`status`, `list`, `verify`, `detach` y `diagnose`. Consulta `ag domains --help` y
`ag domains COMANDO --help` para los argumentos vigentes.

`ag deploy`, `ag migrate` y `ag plugin` son comandos futuros y no estan
disponibles en el binario actual.

### Circulo dominio-despliegue

La [guia del circulo dominio-despliegue](../dns/README.md) amplia este flujo
hasta un circulo completo: requisitos, los entornos `dev`/`staging`/`prod`,
visibilidad publica vs privada, como senalar un dominio propio a un servicio
en ejecucion y resolucion de problemas (propagacion, propiedad, comodines y
TLS). Distingue los comandos `ag domains` presentes en el alpha de los
comandos de la ola 8 aun planificados. Los tokens del proveedor quedan en
variables de entorno, nunca en el repositorio.

## Configurar correo (opcional)

```bash
export AG_SMTP_HOST=smtp.example.com
export AG_SMTP_PORT=587
export AG_SMTP_USER=usuario
export AG_SMTP_PASS=secreto
export AG_MAIL_FROM=noreply@example.com
ag mail test --to yo@example.com
```

## Configurar DNS / TLS (opcional)

```bash
export AG_CLOUDFLARE_TOKEN=tu-token
export AG_DNS_ZONE_ID=tu-zone-id
ag domains check --domain example.com
ag domains sync --schema schema.ag
```
