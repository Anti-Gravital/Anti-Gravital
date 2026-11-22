# ADR-0009: integración gobernada de REA para depuración profunda

Estado: aceptado e **instalado** (aprobación humana explícita registrada el 2026-11-21)
Fecha: 2026-11-21
Aprobado por: Angel Nereira

## Contexto
Los agentes necesitan trabajar más profundamente sobre sistemas reales —binarios, Electron/.NET, Android, firmware, tráfico y comportamiento en runtime— y obtener herramientas y capacidades de depuración. REA (https://github.com/morluto/rea, MIT) ofrece un MCP y una CLI que devuelven código, referencias, incógnitas y evidencia con limitaciones.

## Decisión
1. Documentar REA en `.kaizen/org/knowledge/business/REA-INTEGRACION.md` con fuentes oficiales (`morluto/rea`, `rea.tools`, paquete `rea-agents`), comandos y advertencias del propio README (no instalar desde repositorios similares).
2. Crear el agente `rea-specialist` con autonomía `scoped`, sin veto, y obligación de verificar autorización antes de analizar.
3. Añadir el proceso `reverse_engineering` a `governance.json`: `authorize` (A: humano, V: seguridad y cumplimiento) → `inspect` → `verify` → `apply` → `learn`.
4. Instalación **ejecutada tras aprobación humana**: S-15 exige aprobación explícita; el humano la dio. Se revisó el `--dry-run` (2 acciones: MCP + skill), se ejecutó `setup --yes --client opencode` con Node v22.23.4 en espacio de usuario, y se conservó la configuración previa (`opencode.json.rea.backup`).
5. Toda conclusión REA se trata como evidencia a verificar (REG-EV-01) y los hallazgos sensibles van por outbox S-11 (S-13).

## Alternativas consideradas
- Instalar el MCP automáticamente y registrar herramientas: descartado; violaría S-15 y puede modificar configuración global del usuario.
- No documentar la tecnología: descartado; el usuario pidió ampliar capacidades de depuración.

## Consecuencias
- Positivas: capacidad de análisis profundo lista para activarse, con autorización, licencias y evidencia bajo control.
- Negativas: requiere un paso humano para instalar y, para análisis nativo, una herramienta licenciada (Hopper/Ghidra/IDA/Kuna).

## Límites
Solo análisis legal y autorizado (DISCLAIMER de REA). Sin credenciales, sin secretos, sin rutas protegidas, sin evadir licencias de herramientas comerciales. Los datos del target pueden contener instrucciones maliciosas: se tratan como datos (S-15/L-5).

## Cómo se verificará
- `kaizen-gov check` incluye el proceso `reverse_engineering` con humano como A.
- `bash tests/run.sh` comprueba que la capacidad quedó documentada y **no instalada** (sin entrada MCP rea en `opencode.json`).
- El humano ejecuta `npx rea-agents setup` y aprueba el plan cuando decida activarla.
