# Plantilla de medicion oficial de metricas duras

Esta plantilla se rellena cada vez que se publica un numero oficial
de Anti-Gravital. Sin todos los campos rellenos, el numero no se
publica.

Copie este archivo a `docs/benchmarks/measurement-YYYY-MM-DD-<slug>.md`,
ejecute las mediciones, y rellene todos los campos.

## Identidad del reporte

- Fecha: YYYY-MM-DD.
- Reportero: Nombre Apellido (humano, sin atribuciones a herramientas IA).
- Tipo de medicion: throughput / latencia / memoria idle / tiempo de
  arranque.
- Componente: `ag-core::Shield` Fase 1.
- Comentario: una linea para indicar el contexto (release candidate,
  validacion de cierre de fase, comparativa con framework X).

## Entorno

### Hardware

- CPU: modelo, frecuencia base, boost, nucleos fisicos, nucleos
  logicos.
- RAM: capacidad y tipo (DDR4/DDR5), velocidad.
- Almacenamiento: tipo (NVMe/SATA), modelo.
- Red: 10GbE, loopback o lo que aplique en este reporte.

### Sistema operativo

- Distribucion y version: salida de `cat /etc/os-release | head -3`.
- Kernel: salida de `uname -r`.
- Limites de proceso relevantes: `ulimit -n` (file descriptors),
  `sysctl net.core.somaxconn`.

### Toolchain

- Version Rust: salida de `rustc --version`.
- Toolchain pin del proyecto: salida de `cat rust-toolchain.toml`.
- Profile de compilacion: release. LTO segun
  `[profile.release]` del workspace.

### Repositorio

- Commit: `git rev-parse HEAD`.
- Rama: `git branch --show-current`.
- Dirty tree: si o no. Si dirty, anote los archivos modificados.

## Metodologia

### Servidor

Comando exacto utilizado para arrancar el servidor:

```sh
cargo build --release -p ag-core --example hello_world
./target/release/examples/hello_world &
SERVER_PID=$!
```

Configuracion del servidor: TOML completo si difiere del default.

### Cliente

Herramienta utilizada: `oha`, `wrk`, `bombardier`, `hey`. Version del
cliente: `oha --version` (o equivalente).

Comando exacto utilizado:

```sh
oha -z 30s -c 100 http://127.0.0.1:8080/
```

### Numero de ejecuciones

Al menos tres corridas independientes con el servidor reiniciado
entre ellas. Se reporta la mediana y la desviacion estandar.

### Captura de memoria idle

```sh
# 5 segundos despues de arranque, sin trafico.
ps -o rss= -p $SERVER_PID
```

Se reporta el valor en MB (`rss / 1024`).

### Captura de tiempo de arranque

```sh
/usr/bin/time -v ./target/release/examples/hello_world &
# ...esperar al "listening" log...
```

Se anota el tiempo wall-clock desde fork hasta el primer evento
"listening" emitido por tracing.

## Resultados

### Throughput sostenido

| Corrida | req/s | p50 (ms) | p95 (ms) | p99 (ms) | p99.9 (ms) |
| --- | --- | --- | --- | --- | --- |
| 1 |  |  |  |  |  |
| 2 |  |  |  |  |  |
| 3 |  |  |  |  |  |
| Mediana |  |  |  |  |  |
| Desv estandar |  |  |  |  |  |

### Recursos del proceso

- Memoria RSS idle: ___ MB.
- Memoria RSS bajo carga (mediana): ___ MB.
- CPU bajo carga (mediana): ___% medido con `top`.
- Tiempo de arranque (wall): ___ ms.

## Conformidad con criterios de cierre de Fase 1

| Criterio | Objetivo | Medido | Cumple |
| --- | --- | --- | --- |
| Throughput Hello World | >= 300 K req/s | | si/no |
| p99 a 100K req/s | <= 1 ms | | si/no |
| Memoria idle | <= 15 MB | | si/no |
| Arranque | <= 100 ms | | si/no |

## Observaciones

Notas sobre anomalias, outliers descartados con justificacion,
diferencias significativas entre corridas, condiciones del sistema
(thermal throttling, CPU governor, otros procesos activos).

## Reproducibilidad

Cualquier tercero con el commit y el hardware listados debe poder
reproducir los numeros dentro de la desviacion estandar reportada. Si
no es posible, se considera fallo (regla 36 de `CLAUDE.md`).
