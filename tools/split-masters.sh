#!/usr/bin/env bash
# Divide los documentos maestros en archivos derivados verbatim bajo docs/.
# Uso: ./tools/split-masters.sh
# Idempotente: si los archivos derivados ya existen, los sobrescribe.
# Reglas: el contenido del maestro se preserva byte por byte; solo se agrega
# un encabezado de breadcrumb al principio de cada archivo derivado.
#
# AVISO (ADR-0008): los maestros son ahora bilingues (seccion inglesa primero y
# canonica, seccion espanola despues). Los rangos de abajo apuntan a la SECCION
# INGLESA. Por tanto una regeneracion produce archivos derivados en INGLES y
# MIGRA los derivados historicos que aun estan en espanol bajo docs/architecture/
# y docs/roadmap/. Esa migracion es una accion deliberada y revisable, no una
# operacion rutinaria: ejecute este script solo dentro de una PR dedicada a esa
# migracion. La reconciliacion completa maestro<->derivados se rastrea en un
# GitHub Issue (etiqueta tech-debt). Los rangos se recalculan con:
#   grep -nE '^## ' <maestro>
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ARCH_MASTER="$ROOT/docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md"
ROAD_MASTER="$ROOT/docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md"

mkdir -p "$ROOT/docs/architecture" "$ROOT/docs/roadmap"

write_arch() {
  local num="$1" file="$2" title="$3" start="$4" end="$5" prev="$6" next="$7"
  local out="$ROOT/docs/architecture/$file"
  {
    printf '# Capitulo %s. %s\n\n' "$num" "$title"
    printf '> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion %s\n' "$num"
    printf '> Indice: [docs/architecture/README.md](./README.md)\n'
    if [ -n "$prev" ]; then printf '> Anterior: [%s](./%s)\n' "$prev" "$prev"; fi
    if [ -n "$next" ]; then printf '> Siguiente: [%s](./%s)\n' "$next" "$next"; fi
    printf '\n'
    sed -n "${start},${end}p" "$ARCH_MASTER"
  } > "$out"
}

write_road() {
  local file="$1" title="$2" start="$3" end="$4" prev="$5" next="$6"
  local out="$ROOT/docs/roadmap/$file"
  {
    printf '# %s\n\n' "$title"
    printf '> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md\n'
    printf '> Indice: [docs/roadmap/README.md](./README.md)\n'
    if [ -n "$prev" ]; then printf '> Anterior: [%s](./%s)\n' "$prev" "$prev"; fi
    if [ -n "$next" ]; then printf '> Siguiente: [%s](./%s)\n' "$next" "$next"; fi
    printf '\n'
    sed -n "${start},${end}p" "$ROAD_MASTER"
  } > "$out"
}

# Arquitectura Tecnica: 20 capitulos (seccion inglesa, lineas 48-1251).
# Limites obtenidos con: grep -nE '^## [0-9]+\.' del maestro.
write_arch 1  "01-resumen-ejecutivo.md"            "Resumen ejecutivo"                                  48  63   ""                                          "02-manifiesto-y-posicionamiento.md"
write_arch 2  "02-manifiesto-y-posicionamiento.md" "Manifiesto y posicionamiento"                       64  79   "01-resumen-ejecutivo.md"                   "03-alcance-y-limites.md"
write_arch 3  "03-alcance-y-limites.md"            "Que es Anti-Gravital y que no es (alcance y limites)" 80 118  "02-manifiesto-y-posicionamiento.md"        "04-estado-del-arte.md"
write_arch 4  "04-estado-del-arte.md"              "Analisis del estado del arte"                       119 152  "03-alcance-y-limites.md"                   "05-ecosistema-modulos.md"
write_arch 5  "05-ecosistema-modulos.md"           "Arquitectura del ecosistema: modulos y responsabilidades" 153 334 "04-estado-del-arte.md"            "06-nucleo-shield-y-core.md"
write_arch 6  "06-nucleo-shield-y-core.md"         "Arquitectura del nucleo (ag-core): Shield y Core"   335 457  "05-ecosistema-modulos.md"                  "07-anti-dsl.md"
write_arch 7  "07-anti-dsl.md"                     "El lenguaje Anti-DSL (ag-dsl)"                      458 622  "06-nucleo-shield-y-core.md"                "08-modulos-batteries-included.md"
write_arch 8  "08-modulos-batteries-included.md"   "Modulos batteries-included"                         623 781  "07-anti-dsl.md"                            "09-plugins-wasi.md"
write_arch 9  "09-plugins-wasi.md"                 "Sistema de plugins WASI (ag-wasm-host)"             782 849  "08-modulos-batteries-included.md"          "10-despliegue-ag-cloud.md"
write_arch 10 "10-despliegue-ag-cloud.md"          "Subsistema de despliegue (ag-cloud)"                850 942  "09-plugins-wasi.md"                        "11-ai-knowledge-graph.md"
write_arch 11 "11-ai-knowledge-graph.md"           "Integracion con IA (ag-ai) y el Knowledge Graph"    943 974  "10-despliegue-ag-cloud.md"                 "12-migracion-ag-migrate.md"
write_arch 12 "12-migracion-ag-migrate.md"         "Framework de migracion (ag-migrate): importadores"  975 1004 "11-ai-knowledge-graph.md"                  "13-mobile-ag-mobile.md"
write_arch 13 "13-mobile-ag-mobile.md"             "Puente de aplicaciones nativas (ag-mobile)"         1005 1026 "12-migracion-ag-migrate.md"               "14-observabilidad-ag-observe.md"
write_arch 14 "14-observabilidad-ag-observe.md"    "Observabilidad (ag-observe)"                        1027 1050 "13-mobile-ag-mobile.md"                   "15-seguridad.md"
write_arch 15 "15-seguridad.md"                    "Modelo de seguridad"                                1051 1078 "14-observabilidad-ag-observe.md"          "16-rendimiento-y-validacion.md"
write_arch 16 "16-rendimiento-y-validacion.md"     "Objetivos de rendimiento y metodologia de validacion" 1079 1116 "15-seguridad.md"                       "17-gobernanza-open-source.md"
write_arch 17 "17-gobernanza-open-source.md"       "Modelo de gobernanza Open Source"                   1117 1140 "16-rendimiento-y-validacion.md"           "18-riesgos-y-mitigaciones.md"
write_arch 18 "18-riesgos-y-mitigaciones.md"       "Analisis de riesgos y mitigaciones"                 1141 1174 "17-gobernanza-open-source.md"             "19-glosario.md"
write_arch 19 "19-glosario.md"                     "Glosario tecnico"                                   1175 1223 "18-riesgos-y-mitigaciones.md"             "20-apendice-comparativa.md"
write_arch 20 "20-apendice-comparativa.md"         "Apendice: comparativa de mercado"                   1224 1251 "19-glosario.md"                           ""

# Hoja de Ruta: 12 fases (incluye 4.5 aditiva) + preambulo y reglas.
# Seccion inglesa, lineas 14-664. Limites: grep -nE '^## ' del maestro.
write_road "preambulo.md"                       "Hoja de ruta: como leer este documento"           14  83   ""                                          "fase-00-fundaciones-y-gobernanza.md"
write_road "fase-00-fundaciones-y-gobernanza.md" "Fase 0 - Fundaciones y gobernanza"               84  126  "preambulo.md"                              "fase-01-shield-mvp.md"
write_road "fase-01-shield-mvp.md"               "Fase 1 - The Shield MVP"                         127 173  "fase-00-fundaciones-y-gobernanza.md"       "fase-02-core-mvp.md"
write_road "fase-02-core-mvp.md"                 "Fase 2 - The Core MVP y roundtrip completo"      174 213  "fase-01-shield-mvp.md"                     "fase-03-anti-dsl-alpha.md"
write_road "fase-03-anti-dsl-alpha.md"           "Fase 3 - Anti-DSL alpha (v0.1 a v0.4)"           214 261  "fase-02-core-mvp.md"                       "fase-04-modulos-estandar.md"
write_road "fase-04-modulos-estandar.md"         "Fase 4 - Modulos estandar"                       262 300  "fase-03-anti-dsl-alpha.md"                 "fase-04-5-ag-mail-y-ag-domains.md"
write_road "fase-04-5-ag-mail-y-ag-domains.md"   "Fase 4.5 - ag-mail y ag-domains: comunicacion y dominios" 301 405 "fase-04-modulos-estandar.md"      "fase-05-ag-cloud.md"
write_road "fase-05-ag-cloud.md"                 "Fase 5 - ag-cloud despliegue simplificado"       406 444  "fase-04-5-ag-mail-y-ag-domains.md"         "fase-06-ag-ai-knowledge-graph.md"
write_road "fase-06-ag-ai-knowledge-graph.md"    "Fase 6 - ag-ai y Knowledge Graph"                445 480  "fase-05-ag-cloud.md"                       "fase-07-ag-migrate.md"
write_road "fase-07-ag-migrate.md"               "Fase 7 - ag-migrate importadores"                481 514  "fase-06-ag-ai-knowledge-graph.md"          "fase-08-ag-mobile.md"
write_road "fase-08-ag-mobile.md"                "Fase 8 - ag-mobile Flutter bridge"               515 549  "fase-07-ag-migrate.md"                     "fase-09-plugins-wasi.md"
write_road "fase-09-plugins-wasi.md"             "Fase 9 - Sistema de plugins WASI"                550 583  "fase-08-ag-mobile.md"                      "fase-10-endurecimiento-y-1.0.md"
write_road "fase-10-endurecimiento-y-1.0.md"     "Fase 10 - Endurecimiento y hito 1.0"             584 626  "fase-09-plugins-wasi.md"                   "mas-alla-de-1.0.md"
write_road "mas-alla-de-1.0.md"                  "Mas alla de la 1.0: hojas de ruta futuras"       627 640  "fase-10-endurecimiento-y-1.0.md"           "reglas-de-oro.md"
write_road "reglas-de-oro.md"                    "Reglas de oro del proceso"                       641 664  "mas-alla-de-1.0.md"                        ""

echo "OK: derivados generados."
