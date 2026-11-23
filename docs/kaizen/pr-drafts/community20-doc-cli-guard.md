## Resumen
Add opt-in doc-vs-CLI coherence detector and fix the two live drift markers it guards (OP-24 / FEAT A1).

## Componente afectado
Herramientas de verificacion y documentacion (task P0, proyecto top5-roadmap).

## Tipo de cambio
- [x] enhancement
- [x] documentation

## Documentos relacionados
- `docs/audits/oportunidades-catalogo.md` OP-24 / `docs/audits/oportunidades-funciones.md` FEAT A1
- `docs/audits/terminal-user-review.md` T14 / F1 (`#205`, `#191`)
- `.kaizen/team/EVIDENCIA.md` REG-EV-01/07; `.kaizen/RULES.md` K-1/K-7; L-4

## Plan de prueba
- `bash -n scripts/check-doc-cli-coherence.sh`
- `bash scripts/check-doc-cli-coherence.sh` (arbol vivo: COHERENCE: PASS, salida 0)
- Copia temporal: inyectar un marcador obsoleto en una doc y ejecutar el detector (COHERENCE: FAIL, salida 1, con file:line)
- `git diff --check`
- `bash scripts/ag-check`
- Escaneo de contenido: `grep -nE 'issue #191|dispatch wiring is owned by the clidev role' README.md docs/manual/04-instalacion-y-onboarding.md CHANGELOG.md` -> sin coincidencias

## Impacto en calidad (ratchet)
- Cobertura: igual (detector en shell, sin codigo Rust nuevo)
- Advertencias: igual
- Deuda (TODO/FIXME): igual

## Justificacion S-3 (literales en el detector)
`scripts/check-doc-cli-coherence.sh` contiene los literales `127.0.0.1:8080` y
`0.0.0.0:8080` de forma intencionada: son los marcadores de deriva y las anclas
de codigo que el detector debe comparar (un detector no puede parametrizar aquello
que verifica). No son configuracion operativa, hosts de servicio, puertos de
despliegue ni secretos: el script es una prueba off-line, no lee entorno ni
credenciales y nunca construye una conexion con esos valores. S-3 protege la
configuracion runtime; aqui los literales son datos de la asercion, por lo que
su presencia es requisito de la funcionalidad, no hardcoding de infraestructura.

## Criterios de salida que avanza
- Detector opt-in, sin red, sin secretos, sin gate en `.github/`, sin auto-fix.
- Cada deriva confirmada lleva evidencia real (codigo + comando) y fue corregida en el arbol vivo.
- El detector pasa en el arbol limpio y falla cerrado ante una deriva simulada.

## Aprendizaje (opcional)
Las tres derivas del catalogo ya estaban corregidas en el codigo base; la verificacion con comandos reales encontro que el texto obsoleto sobrevivia en otros ficheros (manual y CHANGELOG). No se fabrico una correccion donde no habia deriva: se reporto con evidencia y se corrigio solo lo vivo.

## Cierre de issues
Closes: none (tarea de tablero P0; Refs OP-24, FEAT A1, #205, #191)

## Checklist
- [x] No modifica rutas protegidas
- [x] Detector opt-in y falla cerrado
- [x] Documentacion actualizada (manual EN+ES, CHANGELOG)
- [x] Linea base no empeora
