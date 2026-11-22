English | Espanol

---

# Chapter 04 — Installation and Onboarding

This chapter guides you from a blank machine to a running Anti-Gravital project.

## Prerequisites

| Requirement | Minimum version | Check                    |
|-------------|-----------------|--------------------------|
| Rust        | 1.99.1          | `rustc --version`        |
| Git         | any modern      | `git --version`          |

No other system dependencies are required for the `rest` template.

## Install the `ag` binary

The alpha (`v0.1.0-alpha.2`) is installed from source: there is no crates.io
package, and source install is the verified path. A prebuilt binary can also be
fetched through the opt-in checksum-only alpha channel once it is verified
end to end (#141). Use one of the methods below, then confirm the version with
`ag --version`. See the
[alpha release notes](../releases/0.1.0-alpha.2.md) for scope and limitations.

### Linux / macOS

```bash
git clone https://github.com/Anti-Gravital/Anti-Gravital.git
cd Anti-Gravital
bash install.sh
```

The script verifies the Rust toolchain, builds the workspace in release mode,
and installs `ag` into `~/.cargo/bin`. Security note: the script does not pipe
untrusted content into a shell. Before running it, follow the canonical
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

The server listens on `0.0.0.0:8080` by default. Note that `ag dev` passes
`BIND=127.0.0.1:8080` by default but the generated templates currently ignore
that variable and hardcode `0.0.0.0:8080` (issue #191). Treat the development
server as network-exposed; do not run it on untrusted networks until this is
fixed. Install `cargo-watch` for automatic reloading on file changes:

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
| Rust      | 1.99.1         | `rustc --version`        |
| Git       | cualquier      | `git --version`          |

## Instalar el binario `ag`

El alpha (`v0.1.0-alpha.2`) se instala desde el codigo fuente: no hay paquete
en crates.io y la instalacion desde el fuente es la ruta verificada. Tambien
se puede obtener un binario precompilado mediante el canal opt-in de solo
checksums una vez verific integralmente (#141). Usa uno de los metodos
siguientes y confirma la version con `ag --version`. Consulta las
[notas de release del alpha](../releases/0.1.0-alpha.2.md) para alcance y
limitaciones.

### Linux / macOS

```bash
git clone https://github.com/Anti-Gravital/Anti-Gravital.git
cd Anti-Gravital
bash install.sh
```

El script verifica el toolchain Rust, compila el workspace en modo release
e instala `ag` en `~/.cargo/bin`. Antes de ejecutarlo, sigue el
[procedimiento canonico de integridad](../security/INSTALLATION_INTEGRITY.md).

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

El servidor escucha en `0.0.0.0:8080` por defecto. `ag dev` pasa
`BIND=127.0.0.1:8080`, pero los templates generados ignoran esa variable y
fijan `0.0.0.0:8080` (issue #191). Trata el servidor de desarrollo como
expuesto a la red; no lo ejecutes en redes no confiables hasta corregirlo.

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
