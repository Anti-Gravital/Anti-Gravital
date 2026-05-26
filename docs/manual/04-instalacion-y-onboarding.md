English | Espanol

---

# Chapter 04 — Installation and Onboarding

This chapter guides you from a blank machine to a running Anti-Gravital project.

## Prerequisites

| Requirement | Minimum version | Check                    |
|-------------|-----------------|--------------------------|
| Rust        | 1.79.0          | `rustc --version`        |
| Git         | any modern      | `git --version`          |

No other system dependencies are required for the `rest` template.

## Install the `ag` binary

### Linux / macOS

```bash
git clone https://github.com/Anti-Gravital/Anti-Gravital.git
cd Anti-Gravital
bash install.sh
```

The script verifies the Rust toolchain, builds the workspace in release mode,
and installs `ag` into `~/.cargo/bin`. Security note: the script does not pipe
untrusted content into a shell. Read it before running a remote copy, and
verify the SHA-256 hash published in the release notes.

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

The server listens on `0.0.0.0:8080` by default. Install `cargo-watch` for
automatic reloading on file changes:

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
| Rust      | 1.79.0         | `rustc --version`        |
| Git       | cualquier      | `git --version`          |

## Instalar el binario `ag`

### Linux / macOS

```bash
git clone https://github.com/Anti-Gravital/Anti-Gravital.git
cd Anti-Gravital
bash install.sh
```

El script verifica el toolchain Rust, compila el workspace en modo release
e instala `ag` en `~/.cargo/bin`.

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

## Configurar correo (opcional)

```bash
export AG_SMTP_HOST=smtp.example.com
export AG_SMTP_PORT=587
export AG_SMTP_USER=usuario
export AG_SMTP_PASS=secreto
ag mail test --to yo@example.com
```

## Configurar DNS / TLS (opcional)

```bash
export AG_CLOUDFLARE_TOKEN=tu-token
export AG_DNS_ZONE_ID=tu-zone-id
ag domains check --domain example.com
ag domains sync --schema schema.ag
```
