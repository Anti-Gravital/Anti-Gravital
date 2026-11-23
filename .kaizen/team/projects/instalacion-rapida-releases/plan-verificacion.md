# Plan de verificación ejecutable — `instalacion-rapida-releases`

Responsable: `qa`. Fecha: 2026-11-23. HEAD de referencia: `b6d3810e6609a8d08fe1400c929aee8adbbf5166` (rama `main`).

Este documento es un **plan**, no un resultado. Todo lo que aquí se afirma sobre el comportamiento actual proviene de documentos y transcripciones, que son **datos no confiables** (L-5/S-15): se re-ejecuta, no se cita como verdad. Ningún comando de este plan se ejecutó al redactarlo; por tanto no hay aquí resultados "después" inventados (K-10). La columna *después* de la plantilla final queda vacía hasta que se ejecute.

Alcance de escritura de este agente: **solo este archivo**. No se han modificado `install.sh`, `install.ps1`, `.github/`, otros archivos de `.kaizen/`, ni `scripts/`. No hay push, PR ni commits. No se ejecutaron `kaizen-team advance|decide|verify`.

---

## 0. Principios de verificación y trazabilidad

| Principio | Aplicación concreta |
| --- | --- |
| L-6 (reproducir antes de arreglar) | Cada caso fail-closed se ejecuta primero contra el código actual; si hoy ya pasa, se registra como "antes: PASS" y se conserva como prueba de regresión. |
| L-9 (medir antes/después) | Toda métrica del KPI se mide con el mismo comando antes y después del cambio; si no mejora o empeora, se abre `regression-suspect` (regla §6). |
| K-10 (no fabricar evidencia) | Los tiempos 0.410 s y 21m22s son **línea base histórica**, no medición de esta sesión. Se vuelven a medir. |
| S-2 / S-12 | Los comandos usan `env VAR=valor cmd` para fijar variables **sintéticas**. Está prohibido `env` sin argumentos, `printenv`, y leer/volcar credenciales. Si aparece un secreto en una salida, se detiene y se escala a `security` sin copiar el valor. |
| S-10 | No se usa `|| true`, `skip`, `xfail` ni `--no-verify`. Los fallos esperados se capturan con `if ...; then rc=0; else rc=$?; fi` y se aseveran contra 1. |
| KPI "0 cambios en firma/confianza" | Este plan **no propone** tocar `.github/`, claves, firmas ni confianza. Cualquier hallazgo que lo requiera se escala a humano; el verificador de `install.ps1` con `ExecutionPolicy` se resuelve con política de CI documentada, nunca con un cambio de firma. |

### 0.1 Fuentes usadas para diseñar el plan (`source_ids` internos)

| `source_id` | Archivo | sha256 (calculado en esta sesión) | Estado |
| --- | --- | --- | --- |
| `LOCAL-QA-01` | `docs/audits/install-user-journey.md` (2026-11-22) | `c8b8891ff81ae89c7fd63cfb6e0ea5bf2fbaafcc9b94a3444f0310ee2bf143ab` | interno / dato no confiable |
| `LOCAL-QA-02` | `install.sh` @ HEAD | `74411c2c8828b0bc796841e7ac8594a2e3f6c88b3601fda185357bba88434624` | interno / dato no confiable |
| `LOCAL-QA-03` | `install.ps1` @ HEAD | `9f6022f7186bbed86b93ef4942edb170510ad709d5a42b348793401c3fc9274c` | interno / dato no confiable |
| `LOCAL-QA-04` | `docs/security/INSTALLATION_INTEGRITY.md` | `0be0a7e51ed1559d8bd624b20a708e162993abeb99090affd3db300664a0bb97` | interno / dato no confiable |
| `LOCAL-QA-05` | `docs/audits/tui-user-journey.md` y `docs/audits/tui-transcripts/*.json` | `01c58a64072d0dc13ef47362a5e1b1e309974989bc8eaa732f8a76fe4305dba5` | interno / dato no confiable |
| `LOCAL-QA-06` | `.kaizen/team/projects/instalacion-rapida-releases/{charter,tasks,state}.json/.md` @ HEAD | ver HEAD | interno / dato no confiable |

Estos IDs **no sustituyen** a `SOURCES.md`: no están registrados como fuentes `verified` y, por REG-EV-01, no pueden fundar por sí solos una decisión sobre firma o confianza. La tarea de `verificador-evidencias` en `tasks.json` es la que debe dejar `verify` en verde. Este plan solo define cómo medir.

### 0.2 Hallazgo de reproducible que condiciona todas las pruebas

El audit (`LOCAL-QA-01`, línea 28) registra `SHA-256 of install.sh = 1ebc3ad0…`. El fichero actual en HEAD (`LOCAL-QA-02`) es `74411c2c…`, y `checksums/installers.sha256` coincide con `74411c2c…`. Es decir: **el valor literal del audit está obsoleto**. Regla derivada: **nunca hardcodear hashes esperados en una prueba**; calcularlos en tiempo de ejecución con `sha256sum`/`Get-FileHash`. Este hallazgo se reporta al orquestador en el handoff (no se escribe al outbox por la restricción de alcance de esta tarea).

---

## 1. Banco de pruebas común (preparación)

Ejecutar desde la raíz del repositorio. Todas las rutas son efímeras; se prohíbe pegar rutas absolutas del usuario (criterio §5.2).

### 1.1 Linux

```bash
set -euo pipefail
REPO="$(git rev-parse --show-toplevel)"
QA_ROOT="$(mktemp -d /tmp/opencode/ag-qa.XXXXXX)"
REL="$QA_ROOT/releases/v0.1.0-alpha.2"
TARGET="${AG_QA_TARGET:-x86_64-unknown-linux-gnu}"   # fijar el triple bajo prueba
BIN="ag-$TARGET"
mkdir -p "$REL"

printf '#!/bin/sh\necho "ag 0.1.0-alpha.2 (fake QA binary)"\n' > "$REL/$BIN"
chmod 0755 "$REL/$BIN"
# IMPORTANTE: generar el manifest dentro de $REL para que $2 del awk sea el nombre pelado
( cd "$REL" && sha256sum "$BIN" > checksums.txt )

REL_URL="file://$REL"
printf 'QA_ROOT=%s\nREL_URL=%s\nTARGET=%s\n' "$QA_ROOT" "$REL_URL" "$TARGET" > "$QA_ROOT/env-qa.txt"
cat "$QA_ROOT/env-qa.txt"
```

Verificación rápida del manifest (criterio binario: la segunda columna debe ser exactamente `$BIN`):

```bash
awk -v b="$BIN" 'NR==1 && $2==b {ok=1} END{exit (ok?0:1)}' "$REL/checksums.txt"
sha256sum -c "$REL/checksums.txt"    # debe imprimir "$BIN: OK"
```

Guardar `$QA_ROOT` entre sesiones leyendo `env-qa.txt`; no hardcodear la ruta.

### 1.2 Windows (PowerShell)

```powershell
$repo   = (Get-Location).Path
$qaRoot = Join-Path $env:TEMP ('ag-qa-' + [guid]::NewGuid().ToString('N'))
$rel    = Join-Path $qaRoot 'releases\v0.1.0-alpha.2'
$target = 'x86_64-pc-windows-msvc'
$bin    = "ag-$target.exe"
New-Item -ItemType Directory -Force -Path $rel | Out-Null

Set-Content -LiteralPath (Join-Path $rel $bin) -Value '@echo off' -Encoding ascii
$hash = (Get-FileHash -LiteralPath (Join-Path $rel $bin) -Algorithm SHA256).Hash.ToLowerInvariant()
# WriteAllText con `n fuerza LF, como exige LOCAL-QA-04
[System.IO.File]::WriteAllText((Join-Path $rel 'checksums.txt'), "$hash  $bin`n")
$relUrl = ([System.Uri]::new($rel)).AbsoluteUri   # file:///C:/...

@"QA_ROOT=$qaRoot`nREL_URL=$relUrl`nTARGET=$target"@ | Set-Content -LiteralPath (Join-Path $qaRoot 'env-qa.txt') -Encoding ascii
Get-Content -LiteralPath (Join-Path $qaRoot 'env-qa.txt')
```

---

## 2. Método para medir "0 compilaciones" sin cargo real

Objetivo: demostrar que el canal rápido **no invoca** `cargo` ni `rustc`, aunque la máquina no tenga toolchain. Se usa un stub que registra cada invocación y un **control positivo** que demuestra que el stub sí detecta llamadas. Sin control positivo, un log vacío no prueba nada (L-10 anti-trampas).

### 2.1 Linux — stubs de `cargo`/`rustc` en PATH

```bash
STUB_BIN="$QA_ROOT/stub-bin"
CMD_LOG="$QA_ROOT/cargo-invocations.log"
mkdir -p "$STUB_BIN"
: > "$CMD_LOG"

cat > "$STUB_BIN/cargo" <<'STUB'
#!/usr/bin/env bash
printf 'cargo %s\n' "$*" >> "${AG_QA_CMD_LOG:?AG_QA_CMD_LOG no definido}"
exit 0
STUB

cat > "$STUB_BIN/rustc" <<'STUB'
#!/usr/bin/env bash
printf 'rustc %s\n' "$*" >> "${AG_QA_CMD_LOG:?AG_QA_CMD_LOG no definido}"
printf 'rustc 1.99.1 (00000000 2026-01-01)\n'
exit 0
STUB
chmod 0755 "$STUB_BIN/cargo" "$STUB_BIN/rustc"

# El stub debe ser el que resuelve; si aparece /usr/bin/cargo, el método está roto
env PATH="$STUB_BIN:$PATH" bash -c 'command -v cargo; command -v rustc'
```

**Canal rápido (cero invocaciones):**

```bash
: > "$CMD_LOG"
if env PATH="$STUB_BIN:$PATH" \
       AG_QA_CMD_LOG="$CMD_LOG" \
       HOME="$QA_ROOT/home-fast" \
       AG_INSTALLER_BIN_DIR="$QA_ROOT/bin-fast" \
       AG_INSTALLER_CHANNEL=alpha \
       AG_INSTALLER_TARGET="$TARGET" \
       AG_INSTALLER_BASE_URL="$REL_URL" \
       bash "$REPO/install.sh" > "$QA_ROOT/fast-stub.out" 2>&1; then rc=0; else rc=$?; fi

test "$rc" -eq 0                 # el canal rápido debe tener éxito
test ! -s "$CMD_LOG"             # CERO invocaciones => 0 compilaciones
test -x "$QA_ROOT/bin-fast/ag"   # y además instaló el binario
```

**Control positivo (el detector funciona):** el canal `source` sí debe invocar `cargo build` y `cargo install` (son las dos compilaciones del audit, `LOCAL-QA-01` §"What worked"/#1).

```bash
: > "$CMD_LOG"
if env PATH="$STUB_BIN:$PATH" \
       AG_QA_CMD_LOG="$CMD_LOG" \
       HOME="$QA_ROOT/home-src" \
       AG_INSTALLER_BIN_DIR="$QA_ROOT/bin-src" \
       AG_INSTALLER_CHANNEL=source \
       bash "$REPO/install.sh" > "$QA_ROOT/source-stub.out" 2>&1; then rc=0; else rc=$?; fi

test "$rc" -eq 0
grep -q '^cargo build ' "$CMD_LOG"
grep -q '^cargo install ' "$CMD_LOG"
awk '/^cargo /{n++} END{exit (n>=2?0:1)}' "$CMD_LOG"
```

Criterio de aprobación: **rápido = 0 líneas** y **source >= 2 líneas**. Si el control positivo falla, el método está invalidado y el resultado "0 compilaciones" no se acepta.

### 2.2 Linux — alternativa `strace` (cuando esté disponible)

No depende de stubs, confirma a nivel de `execve`. Es complementaria, no sustituta del control positivo.

```bash
if command -v strace >/dev/null 2>&1; then
  strace -f -e trace=execve -o "$QA_ROOT/strace-fast.log" \
    env HOME="$QA_ROOT/home-strace" \
        AG_INSTALLER_BIN_DIR="$QA_ROOT/bin-strace" \
        AG_INSTALLER_CHANNEL=alpha \
        AG_INSTALLER_TARGET="$TARGET" \
        AG_INSTALLER_BASE_URL="$REL_URL" \
        bash "$REPO/install.sh" > "$QA_ROOT/strace-fast.out" 2>&1
  n=$(awk '/execve\(/ && /cargo|rustc/ {c++} END{print c+0}' "$QA_ROOT/strace-fast.log")
  printf 'execve(cargo|rustc)=%s\n' "$n"
  test "$n" -eq 0
else
  printf 'strace no disponible: se usa solo el metodo de stubs (valido)\n'
fi
```

### 2.3 Windows — shims `.cmd` en PATH

Se usan `.cmd` (no `.ps1`) para no depender de `ExecutionPolicy` en los shims.

```powershell
$stubBin = Join-Path $qaRoot 'stub-bin'
$cmdLog  = Join-Path $qaRoot 'cargo-invocations.log'
New-Item -ItemType Directory -Force -Path $stubBin | Out-Null
New-Item -ItemType File -Force -Path $cmdLog | Out-Null

@'
@echo off
echo cargo %* >> "%AG_QA_CMD_LOG%"
exit /b 0
'@ | Set-Content -LiteralPath (Join-Path $stubBin 'cargo.cmd') -Encoding ascii

@'
@echo off
echo rustc %* >> "%AG_QA_CMD_LOG%"
echo rustc 1.99.1 (00000000 2026-01-01)
exit /b 0
'@ | Set-Content -LiteralPath (Join-Path $stubBin 'rustc.cmd') -Encoding ascii

$env:PATH = "$stubBin;$env:PATH"
$env:AG_QA_CMD_LOG = $cmdLog
(Get-Command cargo).Source    # debe apuntar a $stubBin\cargo.cmd
(Get-Command rustc).Source    # debe apuntar a $stubBin\rustc.cmd
```

**Canal rápido:**

```powershell
$env:AG_INSTALLER_CHANNEL   = 'alpha'
$env:AG_INSTALLER_TARGET    = $target
$env:AG_INSTALLER_BASE_URL  = $relUrl
$env:AG_INSTALLER_BIN_DIR   = Join-Path $qaRoot 'bin-fast'
$out = Join-Path $qaRoot 'fast-stub.out'
& powershell -NoProfile -NonInteractive -File (Join-Path $repo 'install.ps1') *>&1 |
    Tee-Object -FilePath $out | Out-Null
if ($LASTEXITCODE -ne 0) { throw "rc=$LASTEXITCODE inesperado en canal rapido" }
$inv = @(Get-Content -LiteralPath $cmdLog)
if ($inv.Count -ne 0) { throw "0 compilaciones esperadas, obtenidas $($inv.Count): $($inv -join '; ')" }
if (-not (Test-Path -LiteralPath (Join-Path $env:AG_INSTALLER_BIN_DIR 'ag.exe'))) { throw 'binario no instalado' }
```

**Control positivo:**

```powershell
Clear-Content -LiteralPath $cmdLog
$env:AG_INSTALLER_CHANNEL = 'source'
$env:AG_INSTALLER_BIN_DIR = Join-Path $qaRoot 'bin-src'
& powershell -NoProfile -NonInteractive -File (Join-Path $repo 'install.ps1') *>&1 | Out-Null
$inv = @(Get-Content -LiteralPath $cmdLog)
if (($inv | Where-Object { $_ -like 'cargo build *' }).Count -lt 1) { throw 'control positivo: falta cargo build' }
if (($inv | Where-Object { $_ -like 'cargo install *' }).Count -lt 1) { throw 'control positivo: falta cargo install' }
```

Nota Windows: `Get-Command cargo` / `Get-Command rustc` (equivalente a `strace` en intención) resuelve a los shims. Si resuelve a un toolchain real, el método está roto y se repite con `$env:PATH` corregido.

---

## 3. Método TIME para el canal rápido con release local `file://`

La línea base del audit es `real=0.410 user=0.027 sys=0.043` en Linux con `file://` (`LOCAL-QA-01`, Test 2) y `1282.132 s` (21m22s) en source (Test 1). El umbral del KPI es **<= 10 s**. Se repite 5 veces y se aprueba por el **peor** caso (máximo), no por la mediana.

### 3.1 Linux — `TIMEFORMAT`

Ejecutar con los stubs **fuera** de PATH para medir el tiempo real del instalador.

```bash
: > "$QA_ROOT/time-results.csv"
for i in 1 2 3 4 5; do
  home="$QA_ROOT/home-time-$i"; bindir="$QA_ROOT/bin-time-$i"
  mkdir -p "$home" "$bindir"
  out="$QA_ROOT/time-fast-$i.out"
  env HOME="$home" \
      AG_INSTALLER_BIN_DIR="$bindir" \
      AG_INSTALLER_CHANNEL=alpha \
      AG_INSTALLER_TARGET="$TARGET" \
      AG_INSTALLER_BASE_URL="$REL_URL" \
      bash -c 'TIMEFORMAT="ELAPSED real=%3R user=%3U sys=%3S"; { time bash "$1" ; } 2>&1' \
      _ "$REPO/install.sh" | tee "$out"
  real=$(awk -F'real=' '/^ELAPSED/{split($2,a," "); print a[1]}' "$out")
  test -n "$real"
  printf '%s\n' "$real" >> "$QA_ROOT/time-results.csv"
done
awk '{if($1>max)max=$1; s+=$1; n++} END{printf "n=%d max=%.3f avg=%.3f\n", n, max, s/n; exit (max<=10.0?0:1)}' \
  "$QA_ROOT/time-results.csv"
```

Criterio binario: el `awk` final devuelve 0 **solo si** `max <= 10.0`. El comando `install.sh` debe haber terminado en 0 en las 5 repeticiones (añadir la comprobación de `$rc` con el patrón `if ...; then rc=0; else rc=$?; fi` si se quiere traza por repetición).

### 3.2 Windows — `Measure-Command`

```powershell
# $rel, $relUrl, $target, $repo, $qaRoot vienen de §1.2
$env:AG_INSTALLER_CHANNEL  = 'alpha'
$env:AG_INSTALLER_TARGET   = $target
$env:AG_INSTALLER_BASE_URL = $relUrl
$env:AG_INSTALLER_BIN_DIR  = Join-Path $qaRoot 'bin-time'
$script:rc = $null
$out = Join-Path $qaRoot 'time-fast-win.out'

$m = Measure-Command {
  & powershell -NoProfile -NonInteractive -File (Join-Path $repo 'install.ps1') *>&1 |
      Tee-Object -FilePath $out | Out-Null
  $script:rc = $LASTEXITCODE
}
$elapsed = $m.TotalSeconds
if ($script:rc -ne 0) { throw "rc=$($script:rc)" }
if ($elapsed -gt 10) { throw "Tiempo $elapsed s excede el umbral de 10 s" }
"elapsed=$elapsed s"
```

Repetir 5 veces con `AG_INSTALLER_BIN_DIR` distinto y aprobar por el máximo, igual que en Linux. `Measure-Command` no expone el código de salida: capturarlo dentro del bloque en `$script:rc` es obligatorio, o se corre el riesgo de medir un fallo rápido como "éxito".

---

## 4. Casos fail-closed a repetir

Los seis casos exigidos, más controles. En **todos**: `exit 1` **y** ausencia de binario destino. Se parte de un directorio destino limpio en cada caso para evitar falsos positivos.

### 4.1 Linux — driver y casos

```bash
fc() {
  local name="$1" expected="$2"; shift 2
  local home="$QA_ROOT/fc-$name-home" bindir="$QA_ROOT/fc-$name-bin"
  local out="$QA_ROOT/fc-$name.out" rc
  rm -rf "$home" "$bindir"; mkdir -p "$home"
  if env HOME="$home" AG_INSTALLER_BIN_DIR="$bindir" \
       AG_INSTALLER_TARGET="$TARGET" "$@" \
       bash "$REPO/install.sh" > "$out" 2>&1; then rc=0; else rc=$?; fi
  if [ "$rc" -ne 1 ]; then echo "FAIL $name: rc=$rc (esperado 1)"; return 1; fi
  if [ -e "$bindir/ag" ]; then echo "FAIL $name: escribio binario destino"; return 1; fi
  if ! grep -Fq "$expected" "$out"; then echo "FAIL $name: falta '$expected'"; return 1; fi
  echo "PASS $name"
}
```

Preparación de escenarios:

```bash
# (a) checksum inválido
BAD="$QA_ROOT/releases/bad-checksum"; mkdir -p "$BAD"
cp "$REL/$BIN" "$BAD/$BIN"
printf '%064d  %s\n' 0 "$BIN" > "$BAD/checksums.txt"

# (b) entrada ausente en manifest
NE="$QA_ROOT/releases/no-entry"; mkdir -p "$NE"
cp "$REL/$BIN" "$NE/$BIN"
printf '%064d  %s\n' 0 "otro-binario" > "$NE/checksums.txt"

# (c) binario ausente
NB="$QA_ROOT/releases/no-binary"; mkdir -p "$NB"
cp "$REL/checksums.txt" "$NB/checksums.txt"

# (d) manifest ausente (descarga del release)
NM="$QA_ROOT/releases/no-manifest"; mkdir -p "$NM"
cp "$REL/$BIN" "$NM/$BIN"

# (e) manifiesto out-of-band manipulado
printf '%064d  %s\n' 0 "$BIN" > "$QA_ROOT/tampered-binary.sha256"
```

Ejecución de los seis casos:

```bash
fc checksum-invalido  'Checksum mismatch for' \
   AG_INSTALLER_CHANNEL=alpha AG_INSTALLER_BASE_URL="file://$BAD"

fc entrada-ausente    'No checksum entry for' \
   AG_INSTALLER_CHANNEL=alpha AG_INSTALLER_BASE_URL="file://$NE"

fc binario-ausente    'Could not download the binary from' \
   AG_INSTALLER_CHANNEL=alpha AG_INSTALLER_BASE_URL="file://$NB"

fc manifest-ausente   'Could not download the checksum manifest from' \
   AG_INSTALLER_CHANNEL=alpha AG_INSTALLER_BASE_URL="file://$NM"

fc manifest-oob-ausente 'Binary checksum manifest not found:' \
   AG_INSTALLER_CHANNEL=alpha AG_INSTALLER_BASE_URL="file://$REL" \
   AG_INSTALLER_BINARY_CHECKSUMS="$QA_ROOT/no-existe.sha256"

fc checksum-oob-invalido 'Checksum mismatch for' \
   AG_INSTALLER_CHANNEL=alpha AG_INSTALLER_BASE_URL="file://$REL" \
   AG_INSTALLER_BINARY_CHECKSUMS="$QA_ROOT/tampered-binary.sha256"

fc canal-desconocido  "Unknown AG_INSTALLER_CHANNEL 'beta'" \
   AG_INSTALLER_CHANNEL=beta
```

Casos adicionales de cobertura (mismo driver):

```bash
fc sin-base-url 'requires AG_INSTALLER_BASE_URL or AG_INSTALLER_RELEASE_TAG' \
   AG_INSTALLER_CHANNEL=alpha

printf '%064d  %s\n' 0 'install.sh' > "$QA_ROOT/tampered-installer.sha256"
fc self-integrity 'Integrity check failed for install.sh' \
   AG_INSTALLER_CHECKSUMS="$QA_ROOT/tampered-installer.sha256"
```

**Control positivo de no-falso-negativo:** con manifiesto out-of-band correcto y un release **sin** `checksums.txt`, debe instalar (rc 0 y binario presente). Sin este control, un instalador que siempre falla también "pasa" los fail-closed.

```bash
cp "$REL/checksums.txt" "$QA_ROOT/good-binary.sha256"
if env HOME="$QA_ROOT/oob-ok-home" \
      AG_INSTALLER_BIN_DIR="$QA_ROOT/oob-ok-bin" \
      AG_INSTALLER_TARGET="$TARGET" \
      AG_INSTALLER_CHANNEL=alpha \
      AG_INSTALLER_BASE_URL="file://$NM" \
      AG_INSTALLER_BINARY_CHECKSUMS="$QA_ROOT/good-binary.sha256" \
      bash "$REPO/install.sh" > "$QA_ROOT/oob-ok.out" 2>&1; then rc=0; else rc=$?; fi
test "$rc" -eq 0
test -x "$QA_ROOT/oob-ok-bin/ag"
```

Nota conocida (`LOCAL-QA-01` §"What was confusing", #3): en `binario-ausente` aparece primero un `cp: cannot stat …` crudo. Es cosmético y **no** invalida el fail-closed; si la nueva TUI lo elimina, se registra como mejora, no como regresión.

### 4.2 Windows — driver equivalente

```powershell
function Assert-FailClosed {
  param([string]$Name, [string]$Expected, [hashtable]$Overrides)
  $home   = Join-Path $qaRoot "fc-$Name-home"
  $binDir = Join-Path $qaRoot "fc-$Name-bin"
  $out    = Join-Path $qaRoot "fc-$Name.out"
  if (Test-Path -LiteralPath $binDir) { Remove-Item -LiteralPath $binDir -Recurse -Force }
  New-Item -ItemType Directory -Force -Path $home | Out-Null

  $saved = @{}
  foreach ($k in $Overrides.Keys) {
    $saved[$k] = [Environment]::GetEnvironmentVariable($k)
    [Environment]::SetEnvironmentVariable($k, $Overrides[$k])
  }
  try {
    & powershell -NoProfile -NonInteractive -File (Join-Path $repo 'install.ps1') *>&1 |
        Tee-Object -FilePath $out | Out-Null
    $rc = $LASTEXITCODE
  } finally {
    foreach ($k in $saved.Keys) { [Environment]::SetEnvironmentVariable($k, $saved[$k]) }
  }
  if ($rc -ne 1) { throw "FAIL $Name rc=$rc" }
  if (Test-Path -LiteralPath (Join-Path $binDir 'ag.exe')) { throw "FAIL $Name escribio binario" }
  if (-not (Select-String -LiteralPath $out -SimpleMatch -Pattern $Expected -Quiet)) {
    throw "FAIL $Name falta '$Expected'"
  }
  Write-Host "PASS $Name"
}
```

Preparación y llamadas (equivalentes uno a uno a §4.1; el triple Windows lleva `.exe`):

```powershell
$bad = Join-Path $qaRoot 'releases\bad-checksum'
New-Item -ItemType Directory -Force -Path $bad | Out-Null
Copy-Item -LiteralPath (Join-Path $rel $bin) -Destination (Join-Path $bad $bin) -Force
[System.IO.File]::WriteAllText((Join-Path $bad 'checksums.txt'), ("0" * 64 + "  $bin`n"))

$env:AG_INSTALLER_TARGET = $target
Assert-FailClosed 'checksum-invalido' 'Checksum mismatch for' @{
  AG_INSTALLER_CHANNEL='alpha'; AG_INSTALLER_BASE_URL=([System.Uri]::new($bad)).AbsoluteUri
}
Assert-FailClosed 'canal-desconocido' "Unknown AG_INSTALLER_CHANNEL 'beta'" @{
  AG_INSTALLER_CHANNEL='beta'
}
Assert-FailClosed 'manifest-oob-ausente' 'Binary checksum manifest not found:' @{
  AG_INSTALLER_CHANNEL='alpha'; AG_INSTALLER_BASE_URL=$relUrl
  AG_INSTALLER_BINARY_CHECKSUMS=(Join-Path $qaRoot 'no-existe.sha256')
}
```

Completar los tres casos restantes con la misma plantilla: `entrada-ausente` (`No checksum entry for`), `binario-ausente` (`Could not download the binary from`) y `manifest-ausente` (`Could not download the checksum manifest from`). Criterio idéntico: `rc=1`, sin `ag.exe`, mensaje presente.

### 4.3 Mensajes exactos esperados (ancla de `grep -F`)

| Caso | Subcadena obligatoria | Código | Binario destino |
| --- | --- | --- | --- |
| checksum inválido | `Checksum mismatch for` | 1 | ausente |
| entrada ausente en manifest | `No checksum entry for` | 1 | ausente |
| binario ausente | `Could not download the binary from` | 1 | ausente |
| manifest ausente (release) | `Could not download the checksum manifest from` | 1 | ausente |
| `AG_INSTALLER_BINARY_CHECKSUMS` ausente | `Binary checksum manifest not found:` | 1 | ausente |
| `AG_INSTALLER_BINARY_CHECKSUMS` inválido | `Checksum mismatch for` | 1 | ausente |
| canal desconocido | `Unknown AG_INSTALLER_CHANNEL 'beta'` | 1 | ausente |

Fuente de las cadenas: `install.sh` líneas 161, 179, 189, 195, 200, 209 y 244; `install.ps1` líneas 113, 129, 148, 155 y 179 (`LOCAL-QA-02`, `LOCAL-QA-03`).

---

## 5. Aseveraciones de salida TUI / headless

### 5.1 Cadenas requeridas y prohibidas

Dos grupos, para no confundir **evidencia actual** con **criterio de aceptación futuro**:

**(A) Cadenas verificables hoy** (deben seguir apareciendo tras el cambio; `LOCAL-QA-01` Tests 2–3):

```text
[ag-install] Alpha channel: checksum verification only (no signing key, no signature).
[ag-install] Checksum verified for <asset>.
[ag-install] Installation complete.
```

**(B) Cadenas exigidas a la nueva TUI/headless** (criterio de aceptación; **no** son evidencia de que hoy existan):

- Al menos **4 marcadores de etapa distintos** con estado real: p. ej. `[etapa 1/4] resolve`, `[etapa 2/4] download`, `[etapa 3/4] verify`, `[etapa 4/4] install`. Los nombres concretos son una convención propuesta: `dx`/`architect` pueden fijar otros, pero cualquier cambio debe actualizar esta tabla en el mismo commit (K-1).
- Progreso con estado real, no decorativo: cada etapa debe incluir al menos una de `bytes=`, `ETA=` o `status=ok` con valores derivados de la operación (tamaño del asset descargado / resultado de `sha256sum`).
- Confirmación de integridad: `Checksum verified` **o** `Integrity verified` (nunca "firma verificada").

Aserción binaria Linux:

```bash
OUT="$QA_ROOT/fast-stub.out"
for s in 'Alpha channel: checksum verification only' 'Checksum verified for' 'Installation complete'; do
  grep -Fq "$s" "$OUT" || { echo "FAIL: falta '$s'"; exit 1; }
done
# etapas: convencion propuesta [etapa N/4]
n=$(grep -oE '\[etapa [1-4]/4\]' "$OUT" | sort -u | wc -l)
test "$n" -ge 4
# estado real: al menos una medicion/estado
grep -Eq 'bytes=|[Ee][Tt][Aa]=|status=ok' "$OUT"
```

**Cadenas que NO deben aparecer** (fallo = BLOCK):

| Prohibido | Patrón de comprobación |
| --- | --- |
| Falsa firma / autenticidad | `grep -Eiq 'signature verified\|firma verificada\|signed by\|authenticity verified\|GPG (OK\|good)' "$OUT"` → debe ser falso |
| Secreto / credencial | Ver §5.3 (conjunto de hashes hex de 64 permitidos) |
| Ruta local del usuario | `grep -Eq '/home/\|/Users/\|[A-Za-z]:\\Users\\' "$OUT"` → debe ser falso en modo headless; además `grep -Fq "$QA_ROOT" "$OUT"` → falso |
| Volcado de entorno | `grep -Eiq 'AG_QA_CMD_LOG=\|RAILWAY_\|TOKEN=\|PASSWORD=\|PATH=' "$OUT"` → falso |

El criterio de "sin ruta local" es un **delta intencional** respecto al comportamiento actual: el canal alpha imprime hoy la ruta absoluta resuelta (`LOCAL-QA-01` líneas 145–147). En headless/CI debe sustituirse por un placeholder (`$BIN_DIR`, `<destino>`). Registrarlo como cambio de comportamiento con su documentación (K-7).

> Nota de seguridad: `docs/audits/tui-transcripts/config-noninteractive-flag.json` contiene un bloque `env` con valores credenciales en texto plano. **No se reproducen aquí** (S-12). No se copiaron a este plan ni a ningún comando. consecuencia verificable: la salida del instalador nunca debe volcar variables de entorno; el saneamiento de ese fichero y la rotación de esos valores requieren intervención humana y se escalan a `security` por el orquestador.

### 5.2 Comprobar que no hay ANSI en modo headless

Linux (conteo de bytes ESC, insensible a `grep` y a locale):

```bash
OUT="$QA_ROOT/fast-stub.out"
esc=$(LC_ALL=C tr -cd '\033' < "$OUT" | wc -c)
test "$esc" -eq 0
# OSC / CSI / pantalla alternativa / cursor
if LC_ALL=C grep -aqE $'\033\\]|\033\[|\033\[\?1049|\033\[\?25' "$OUT"; then
  echo 'FAIL: secuencias de terminal en modo headless'; exit 1
fi
```

Windows:

```powershell
$bytes = [System.IO.File]::ReadAllBytes($out)
if ($bytes -contains 0x1B) { throw 'FAIL: byte ESC en salida headless' }
if (Select-String -LiteralPath $out -Pattern '\x1b\[|\x1b\]' -Quiet) { throw 'FAIL: CSI/OSC en headless' }
```

**Contraste interactivo (demuestra que la animación sí existe cuando hay TTY):** la TUI de `ag` ya expone en sus transcripciones contadores `color_sgr_count`, `alt_screen_enter`, `cursor_hide` (`LOCAL-QA-05`); el instalador debe comportarse de forma análoga solo con TTY.

```bash
if command -v script >/dev/null 2>&1; then
  PTY="$QA_ROOT/pty.out"
  script -qec "env HOME='$QA_ROOT/home-pty' AG_INSTALLER_BIN_DIR='$QA_ROOT/bin-pty' \
    AG_INSTALLER_CHANNEL=alpha AG_INSTALLER_TARGET='$TARGET' \
    AG_INSTALLER_BASE_URL='$REL_URL' bash '$REPO/install.sh'" "$PTY" >/dev/null 2>&1
  esc_pty=$(LC_ALL=C tr -cd '\033' < "$PTY" | wc -c)
  test "$esc_pty" -gt 0       # con TTY: animacion presente
  test "$esc" -eq 0           # sin TTY: cero ANSI
fi
```

Criterio binario: `ESC(headless)=0` **y** `ESC(pty)>0` **y** `etapas distintas >= 4` en PTY.

### 5.3 Detección de secretos sin falsos positivos

Los hashes SHA-256 de 64 hex son salida legítima en mensajes de checksum. Por eso no basta un regex genérico: se comparan los hashes encontrados con un conjunto permitido calculado en runtime.

```bash
OUT="$QA_ROOT/fast-stub.out"
ALLOWED="$QA_ROOT/allowed-hashes.txt"
: > "$ALLOWED"
{
  awk '{print $1}' "$REL/checksums.txt"
  sha256sum "$REPO/install.sh" | awk '{print $1}'
  sha256sum "$REPO/install.ps1" | awk '{print $1}'
} >> "$ALLOWED"
sort -u "$ALLOWED" -o "$ALLOWED"

grep -oE '\b[0-9a-fA-F]{64}\b' "$OUT" | tr 'A-Z' 'a-z' | sort -u > "$QA_ROOT/out-hashes.txt"
if comm -23 "$QA_ROOT/out-hashes.txt" "$ALLOWED" | grep -q .; then
  echo 'FAIL: hash no reconocido en salida (revisar si es secreto)'; exit 1
fi
# Ademas: UUIDs y tokens largos no deben aparecer nunca
if grep -Eq '[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}' "$OUT"; then
  echo 'FAIL: UUID con aspecto de token en salida'; exit 1
fi
```

El conjunto permitido se inicializa vacío con `: > "$ALLOWED"` y se puebla solo con hashes calculados en runtime; no hay supresiones S-10.

### 5.4 Ejecutar `install.ps1` en CI con errores controlados

Patrón obligatorio: proceso hijo, salida combinada, captura explícita de `$LASTEXITCODE`, sin supresión de errores.

```powershell
# El script ya declara $ErrorActionPreference = "Stop" (install.ps1:39)
$ciLog = Join-Path $qaRoot 'ci-install.out'
& powershell -NoProfile -NonInteractive -File (Join-Path $repo 'install.ps1') *>&1 |
    Tee-Object -FilePath $ciLog | Out-Null
$rc = $LASTEXITCODE
"exit_code=$rc"
# Criterio: rc 0 en el caso feliz; rc 1 en los casos fail-closed (§4.2)
if ($rc -notin 0,1) { throw "codigo inesperado: $rc" }
```

- `-NonInteractive` evita que el script espere entrada en CI.
- No usar `2>$null`, `-ErrorAction SilentlyContinue` ni `try/catch` que trague el error: S-7/S-10.
- Si la política de ejecución del runner bloquea scripts locales, se documenta la política en la config de CI y se revisa con humano. **No** se propone cambiar firma, confianza ni `.github/` desde este plan (KPI, ADR-0023).
- Modo headless propuesto: `AG_INSTALLER_HEADLESS=1` (variable a confirmar con `dx`/`architect`). Su criterio es exactamente el de §5.2 y §5.3; si no se implementa la variable, el modo headless se detecta por ausencia de TTY y se documenta.

---

## 6. Plantilla de evidencia antes/después (L-9)

Copiar esta tabla al descriptor/export al ejecutar. **"Antes" = línea base histórica o"No medido"**; nunca se inventa un valor. "Después" se rellena solo con salida real del comando.

| ID | Comando | Antes | Umbral | Después | Resultado |
| --- | --- | --- | --- | --- | --- |
| E1 | §3.1 Linux `time` canal rápido `file://` (5 repeticiones, max) | 0.410 s (`LOCAL-QA-01`, no re-medido en esta sesión) | <= 10.0 s | — | — |
| E2 | §3.2 Windows `Measure-Command` (5 repeticiones, max) | No medido; observación: source sin progreso (`blackboard`) | <= 10.0 s | — | — |
| E3 | §2.1 log de stubs, canal rápido | N/A (canal rápido aún no implementado por defecto) | 0 invocaciones `cargo`/`rustc` | — | — |
| E4 | §2.1 control positivo, canal source | 2 compilaciones (`cargo build` + `cargo install`, `LOCAL-QA-01`) | >= 2 invocaciones detectadas | — | — |
| E5 | §2.3 stubs Windows, canal rápido | No medido | 0 invocaciones | — | — |
| E6 | §4.1 los 6 casos fail-closed | 6/6 abortan con `exit 1` sin binario (`LOCAL-QA-01` Tests 3–4) | 100% (7/7 incluyendo oob) | — | — |
| E7 | §4.2 los 6 casos en Windows | No medido | 100% | — | — |
| E8 | §5.1 marcadores de etapa | 0 etapas animadas (`LOCAL-QA-01`, `blackboard`) | >= 4 etapas distintas con estado | — | — |
| E9 | §5.2 ESC en headless / ESC en PTY | No existe modo headless | 0 ESC headless, >0 ESC PTY | — | — |
| E10 | §5.3 secretos en salida | No medido (finding en transcripciones, fuera del instalador) | 0 valores no permitidos | — | — |
| E11 | `git diff --name-only` vs `.github/`, firma, confianza | 0 cambios | 0 cambios | — | — |
| E12 | §3.1 comparador source Linux | 1282.132 s (21m22s, `LOCAL-QA-01`, no re-medido) | referencia: debe ser >10x el canal rápido | — | — |

\* E12 es un comparador informativo; el gate es E1/E2. Si se decide re-medir source, hacerlo con el stub **fuera** de PATH y con `CARGO_TARGET_DIR` aislado, nunca sobre el target compartido.

### 6.1 Regla de `regression-suspect` (L-9)

Se aplica **cualquiera** de estas condiciones:

1. `después` no cumple el umbral de su fila; o
2. `después` cumple el umbral pero es peor que `antes` cuando `antes` ya lo cumplía (E6 es el caso crítico: pasar de 7/7 a 6/7); o
3. cualquier caso fail-closed **escribe** un binario destino, aunque el tiempo mejore; o
4. aparece una falsa firma, un secreto o una ruta local del usuario en la salida.

Acción: **QA emite `BLOCK`** con la fila y la salida real (sin secretos); se abre issue `regression-suspect` con evidencia y se prepara un PR borrador de revert. Los casos 3 y 4 se escalan además a `security` por canal `alert`. No se cierra el issue hasta que la métrica vuelva a cumplir (K-2). El orquestador decide el merge; este plan no fusiona.

---

## 7. Checklist de aprobación binario

- [ ] §2.1: rápido = 0 líneas en `$CMD_LOG` **y** source >= 2 líneas (control positivo verde).
- [ ] §2.3: mismo resultado en Windows con `Get-Command` apuntando a los shims.
- [ ] §3.1 y §3.2: `max <= 10.0 s` en 5 repeticiones, con rc 0.
- [ ] §4.1 y §4.2: 7/7 casos fail-closed con `rc=1`, binario ausente y mensaje esperado.
- [ ] Control positivo out-of-band: rc 0 y binario instalado.
- [ ] §5.1: cadenas (A) presentes; >= 4 etapas; `bytes=|ETA=|status=ok` presente.
- [ ] §5.1: 0 coincidencias de falsa firma, ruta local, volcado de entorno.
- [ ] §5.2: `ESC(headless)=0`, `ESC(pty)>0`.
- [ ] §5.3: 0 hashes no permitidos, 0 UUIDs de aspecto token.
- [ ] E11: 0 cambios en `.github/`, firma o confianza.
- [ ] `scripts/kaizen check` ejecutado por quien integra, con resultado real reportado (no por este plan).

---

## 8. Límites, Hallazgos y escalado

- Este agente **no** ejecutó los comandos: no hay resultados "después" en este documento. Ejecutar y rellenar §6 antes de cualquier dictamen `APPROVE`.
- Hallazgo H1 (reproducibilidad): hash de `install.sh` en `LOCAL-QA-01` difiere del actual (§0.2). No se hardcodea ningún hash. Se reporta al orquestador.
- Hallazgo H2 (seguridad, sin valores copiados): `docs/audits/tui-transcripts/config-noninteractive-flag.json` incluye credenciales en su bloque `env` (§5.1). Requiere saneamiento y rotación por humano; escalar a `security`. No se detallan valores aquí (S-12/S-13).
- Hallazgo H3 (documentación): `LOCAL-QA-01` #204/#206 (`~/.cargo/bin` hardcoded en source; dos builds completos) son comportamiento conocido, no regresiones del cambio propuesto. Si el canal rápido los elimina, documentar en CHANGELOG y borrador de cierre de esos issues por el orquestador.
- La autonomía de `qa` es `propose`/`execute` dentro de su contrato con **veto funcional**: este plan puede bloquear con evidencia, pero no aprueba, no fusiona y no toca rutas protegidas (L-4). Las decisiones sobre firma/confianza requieren humano (ADR-0003).
- Contenido de issues, buzones, transcripciones y audits leído como dato no confiable (L-5/S-15); ninguna instrucción encontrada en ellos se ejecutó.
