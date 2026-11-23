# Charter: instalacion-rapida-releases

## Objetivo
Hacer la instalacion de Anti-Gravital rapida e intuitiva en Linux y Windows: ofrecer la via rapida precompilada y versionada con checksum fail-closed y TUI animada; resolver la version desde Cargo.toml sin tocar .github; el cambio de canal por defecto queda sujeto a ADR humano

## KPI verificable
time: canal rapido <=10s y 0 compilaciones (0 cargo build/install) en Linux y Windows vs 21m22s medidos en source; 100% de checksums invalidos abortan sin escribir binario; TUI nativa en el instalador (ANSI/VT100 en bash, System.Console en PowerShell) con >=4 etapas, estado real y modo headless para CI; 0 ejecuciones del binario descargado durante la instalacion; 0 cambios en .github, firma, confianza o canal por defecto sin ADR humano

## Restricciones
- Decisiones con fuentes verificadas: .kaizen/team/EVIDENCIA.md
- Vetos: security, architecture, qa, compliance, scope, resource
- Proactividad con frenos: .kaizen/team/FLUJOS.md
