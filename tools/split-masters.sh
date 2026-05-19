#!/usr/bin/env bash
# Divide los documentos maestros en archivos derivados verbatim bajo docs/.
# Uso: ./tools/split-masters.sh
# Idempotente: si los archivos derivados ya existen, los sobrescribe.
# Reglas: el contenido del maestro se preserva byte por byte; solo se agrega
# un encabezado de breadcrumb al principio de cada archivo derivado.
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

# Arquitectura Tecnica: 20 capitulos.
# Limites obtenidos con: grep -n '^## ' del maestro.
write_arch 1  "01-resumen-ejecutivo.md"            "Resumen ejecutivo"                                  38  53   ""                                          "02-manifiesto-y-posicionamiento.md"
write_arch 2  "02-manifiesto-y-posicionamiento.md" "Manifiesto y posicionamiento"                       54  69   "01-resumen-ejecutivo.md"                   "03-alcance-y-limites.md"
write_arch 3  "03-alcance-y-limites.md"            "Que es Anti-Gravital y que no es (alcance y limites)" 70 105  "02-manifiesto-y-posicionamiento.md"        "04-estado-del-arte.md"
write_arch 4  "04-estado-del-arte.md"              "Analisis del estado del arte"                       106 139  "03-alcance-y-limites.md"                   "05-ecosistema-modulos.md"
write_arch 5  "05-ecosistema-modulos.md"           "Arquitectura del ecosistema: modulos y responsabilidades" 140 298 "04-estado-del-arte.md"            "06-nucleo-shield-y-core.md"
write_arch 6  "06-nucleo-shield-y-core.md"         "Arquitectura del nucleo (ag-core): Shield y Core"   299 421  "05-ecosistema-modulos.md"                  "07-anti-dsl.md"
write_arch 7  "07-anti-dsl.md"                     "El lenguaje Anti-DSL (ag-dsl)"                      422 585  "06-nucleo-shield-y-core.md"                "08-modulos-batteries-included.md"
write_arch 8  "08-modulos-batteries-included.md"   "Modulos batteries-included"                         586 658  "07-anti-dsl.md"                            "09-plugins-wasi.md"
write_arch 9  "09-plugins-wasi.md"                 "Sistema de plugins WASI (ag-wasm-host)"             659 726  "08-modulos-batteries-included.md"          "10-despliegue-ag-cloud.md"
write_arch 10 "10-despliegue-ag-cloud.md"          "Subsistema de despliegue (ag-cloud)"                727 806  "09-plugins-wasi.md"                        "11-ai-knowledge-graph.md"
write_arch 11 "11-ai-knowledge-graph.md"           "Integracion con IA (ag-ai) y el Knowledge Graph"    807 838  "10-despliegue-ag-cloud.md"                 "12-migracion-ag-migrate.md"
write_arch 12 "12-migracion-ag-migrate.md"         "Framework de migracion (ag-migrate): importadores"  839 868  "11-ai-knowledge-graph.md"                  "13-mobile-ag-mobile.md"
write_arch 13 "13-mobile-ag-mobile.md"             "Puente de aplicaciones nativas (ag-mobile)"         869 890  "12-migracion-ag-migrate.md"                "14-observabilidad-ag-observe.md"
write_arch 14 "14-observabilidad-ag-observe.md"    "Observabilidad (ag-observe)"                        891 914  "13-mobile-ag-mobile.md"                    "15-seguridad.md"
write_arch 15 "15-seguridad.md"                    "Modelo de seguridad"                                915 942  "14-observabilidad-ag-observe.md"           "16-rendimiento-y-validacion.md"
write_arch 16 "16-rendimiento-y-validacion.md"     "Objetivos de rendimiento y metodologia de validacion" 943 980 "15-seguridad.md"                          "17-gobernanza-open-source.md"
write_arch 17 "17-gobernanza-open-source.md"       "Modelo de gobernanza Open Source"                   981 1004 "16-rendimiento-y-validacion.md"            "18-riesgos-y-mitigaciones.md"
write_arch 18 "18-riesgos-y-mitigaciones.md"       "Analisis de riesgos y mitigaciones"                 1005 1038 "17-gobernanza-open-source.md"             "19-glosario.md"
write_arch 19 "19-glosario.md"                     "Glosario tecnico"                                   1039 1080 "18-riesgos-y-mitigaciones.md"             "20-apendice-comparativa.md"
write_arch 20 "20-apendice-comparativa.md"         "Apendice: comparativa de mercado"                   1081 1105 "19-glosario.md"                           ""

# Hoja de Ruta: 11 fases + preambulo y reglas.
write_road "preambulo.md"                       "Hoja de ruta: como leer este documento"           1   47   ""                                          "fase-00-fundaciones-y-gobernanza.md"
write_road "fase-00-fundaciones-y-gobernanza.md" "Fase 0 - Fundaciones y gobernanza"               49  90   "preambulo.md"                              "fase-01-shield-mvp.md"
write_road "fase-01-shield-mvp.md"               "Fase 1 - The Shield MVP"                         92  135  "fase-00-fundaciones-y-gobernanza.md"       "fase-02-core-mvp.md"
write_road "fase-02-core-mvp.md"                 "Fase 2 - The Core MVP y roundtrip completo"      137 175  "fase-01-shield-mvp.md"                     "fase-03-anti-dsl-alpha.md"
write_road "fase-03-anti-dsl-alpha.md"           "Fase 3 - Anti-DSL alpha (v0.1 a v0.4)"           177 223  "fase-02-core-mvp.md"                       "fase-04-modulos-estandar.md"
write_road "fase-04-modulos-estandar.md"         "Fase 4 - Modulos estandar"                       225 262  "fase-03-anti-dsl-alpha.md"                 "fase-05-ag-cloud.md"
write_road "fase-05-ag-cloud.md"                 "Fase 5 - ag-cloud despliegue simplificado"       264 301  "fase-04-modulos-estandar.md"               "fase-06-ag-ai-knowledge-graph.md"
write_road "fase-06-ag-ai-knowledge-graph.md"    "Fase 6 - ag-ai y Knowledge Graph"                303 337  "fase-05-ag-cloud.md"                       "fase-07-ag-migrate.md"
write_road "fase-07-ag-migrate.md"               "Fase 7 - ag-migrate importadores"                339 371  "fase-06-ag-ai-knowledge-graph.md"          "fase-08-ag-mobile.md"
write_road "fase-08-ag-mobile.md"                "Fase 8 - ag-mobile Flutter bridge"               373 406  "fase-07-ag-migrate.md"                     "fase-09-plugins-wasi.md"
write_road "fase-09-plugins-wasi.md"             "Fase 9 - Sistema de plugins WASI"                408 440  "fase-08-ag-mobile.md"                      "fase-10-endurecimiento-y-1.0.md"
write_road "fase-10-endurecimiento-y-1.0.md"     "Fase 10 - Endurecimiento y hito 1.0"             442 483  "fase-09-plugins-wasi.md"                   "mas-alla-de-1.0.md"
write_road "mas-alla-de-1.0.md"                  "Mas alla de la 1.0: hojas de ruta futuras"       485 497  "fase-10-endurecimiento-y-1.0.md"           "reglas-de-oro.md"
write_road "reglas-de-oro.md"                    "Reglas de oro del proceso"                       499 519  "mas-alla-de-1.0.md"                        ""

echo "OK: derivados generados."
