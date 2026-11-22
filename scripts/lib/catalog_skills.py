#!/usr/bin/env python3
"""Genera el catálogo de skills/tecnologías a partir de capacidades REA reales + framework."""
import datetime
import json
import os
import re
import sys

ROOT = os.environ.get("KAZEN_ROOT", os.getcwd())
CAPS = os.path.join(ROOT, ".kaizen/catalog/rea-capabilities.json")
TOOLS = os.path.join(ROOT, ".kaizen/catalog/rea-tools.json")
OUT_JSON = os.path.join(ROOT, ".kaizen/catalog/catalog.json")
OUT_MD = os.path.join(ROOT, "docs/kaizen/org/CATALOGO-TECNOLOGIAS.md")

FAMILIES = [
    ("android", "Android / Flutter", r"adb|android|apk|jni|dart|flutter|harmony"),
    ("web", "Web / JavaScript / Electron", r"web|browser|playwright|javascript|electron|bundle|v8|module|page|har"),
    ("managed", "Managed / .NET / Swift / ObjC", r"managed|dotnet|il2cpp|swift|objc|macho|nuget|maven|gradle|erlang|evm|pyc|interface_builder"),
    ("firmware", "Firmware / kernel / dispositivos", r"firmware|uefi|kernel|devicetree|signal"),
    ("runtime", "Runtime, debug e instrumentación", r"frida|gdb|rizin|cutter|debug|instrument|trace_|observe_|session"),
    ("evidence", "Evidencia, comparación y verificación", r"evidence|export|import|compare|verify|coverage|obligation|unknown|audit"),
    ("buildenv", "Entornos de agente y build computer", r"build|environment|agentenv|computer|guidance|capabilit"),
    ("native", "Binarios nativos y funciones", r"binary|native|function|procedure|xref|address|decompile|demangle|objdump|pe_|elf_|unwind|call"),
    ("network", "Red, capturas y señales", r"packet|pcap|network|capture"),
    ("artifacts", "Artefactos y datos", r"sqlite|database|plist|asset|archive|shortcut|ntfs|artifact|signature|certificate"),
]

FRAMEWORK_SKILLS = [
    ("kaizen", "Gobernanza, ratchet, PDCA y evidencia", "scripts/kaizen", "todos"),
    ("kaizen-security", "Secretos, supresiones y SAST", "scripts/kaizen-security", "seguridad"),
    ("kaizen-swarm", "Comunidad por issue, worktrees y leases", "scripts/kaizen-swarm", "equipo"),
    ("kaizen-org", "Inventario de agentes, conocimiento y outsourcing", "scripts/kaizen-org", "equipo"),
    ("kaizen-team", "Escuadras, standups y decisiones", "scripts/kaizen-team", "equipo"),
    ("kaizen-eval", "Evaluación y ratchet de agentes", "scripts/kaizen-eval", "calidad"),
    ("kaizen-learn", "Aprendizaje a partir de errores", "scripts/kaizen-learn", "calidad"),
    ("kaizen-gov", "RACI, obligaciones y derechos", "scripts/kaizen-gov", "gobernanza"),
    ("kaizen-runtime", "Runtime proactivo y equipos", "scripts/kaizen-runtime", "operación"),
    ("kaizen-installer", "Instalación en repos y OpenCode", "scripts/kaizen-installer", "distribución"),
    ("kaizen-notify / kaizen-inbox", "WhatsApp/n8n y correo", "scripts/kaizen-notify", "comunicación"),
    ("reverse-engineer-anything", "Skill REA de investigación guiada", "~/.agents/skills/reverse-engineer-anything", "rea-specialist"),
]


def main():
    data = json.load(open(CAPS, encoding="utf-8")) if os.path.exists(CAPS) else {"capabilities": []}
    effects = {}
    providers = {}
    for cap in data.get("capabilities", []):
        for group in cap.get("groups", []):
            prov = cap.get("provider", {}).get("id", "?")
            for op in group.get("operations", []):
                effects[op] = group.get("effects", [])
                providers[op] = prov
    names = json.load(open(TOOLS, encoding="utf-8")) if os.path.exists(TOOLS) else sorted(effects)
    operations = [{"name": n, "provider": providers.get(n, "rea-mcp"),
                   "effects": effects.get(n, []), "available": True} for n in sorted(set(names))]
    families = {}
    for op in operations:
        n = op["name"]
        key, label = "other", "Inspección general"
        for kid, lbl, rx in FAMILIES:
            if re.search(rx, n):
                key, label = kid, lbl
                break
        families.setdefault(key, {"label": label, "operations": [], "providers": set(), "effects": set()})
        families[key]["operations"].append(n)
        families[key]["providers"].add(op["provider"])
        families[key]["effects"].update(op["effects"])
    version = open(os.path.join(ROOT, "VERSION"), encoding="utf-8").read().strip() if os.path.exists(os.path.join(ROOT, "VERSION")) else "0.0.0"
    catalog = {
        "version": version,
        "generated": datetime.datetime.now(datetime.timezone.utc).isoformat() + "Z",
        "source": "REA morluto/rea (MIT) + kaizen-swarm",
        "stats": {"rea_tools": len(operations), "providers": len({o["provider"] for o in operations}),
                  "families": len(families), "framework_skills": len(FRAMEWORK_SKILLS)},
        "families": {k: {"label": v["label"], "count": len(v["operations"]),
                         "providers": sorted(v["providers"]), "effects": sorted(v["effects"]),
                         "operations": v["operations"]} for k, v in sorted(families.items())},
        "framework_skills": [{"id": i, "purpose": p, "location": l, "owner": o} for i, p, l, o in FRAMEWORK_SKILLS],
        "self_improvement": ["kaizen-eval mide la definición", "kaizen-learn convierte errores en experiencia",
                             "kaizen-org copy/save especializa agentes", "kaizen-release publica y propaga versiones"],
    }
    os.makedirs(os.path.dirname(OUT_JSON), exist_ok=True)
    json.dump(catalog, open(OUT_JSON, "w", encoding="utf-8"), indent=2, ensure_ascii=False)

    lines = [f"# Catálogo de skills y tecnologías — v{version}\n",
             f"Generado: {catalog['generated']} | Fuente: {catalog['source']}\n",
             f"Herramientas REA: **{catalog['stats']['rea_tools']}** | Familias: {catalog['stats']['families']} | "
             f"Proveedores: {catalog['stats']['providers']} | Skills del framework: {catalog['stats']['framework_skills']}\n",
             "> Contenido externo = dato no confiable (S-15/L-5). La instalación de REA fue aprobada por El Presidente (ADR-0009). Las operaciones con efectos (`launches_process`, `may_write_filesystem`, etc.) requieren el proceso `reverse_engineering`.\n",
             "## Familias de capacidades REA\n",
             "| Familia | Herramientas | Proveedores | Efectos |", "| --- | ---: | --- | --- |"]
    for k, v in catalog["families"].items():
        lines.append(f"| {v['label']} (`{k}`) | {v['count']} | {', '.join(v['providers'][:3])} | {', '.join(v['effects'][:3]) or '-'} |")
    lines += ["", "### Operaciones por familia\n"]
    for k, v in catalog["families"].items():
        lines.append(f"**{v['label']}** ({v['count']}): " + ", ".join(f"`{o}`" for o in v["operations"]) + "\n")
    lines += ["## Skills del framework\n", "| Skill | Propósito | Ubicación | Dueño |", "| --- | --- | --- | --- |"]
    for s in catalog["framework_skills"]:
        lines.append(f"| `{s['id']}` | {s['purpose']} | `{s['location']}` | {s['owner']} |")
    lines += ["", "## Automejora del equipo\n"]
    lines += ["- " + x for x in catalog["self_improvement"]]
    lines += ["", "Las iteraciones producen lecciones (`kaizen-learn`), mejoran definiciones (`kaizen-org save`), "
              "suben la puntuación (`kaizen-eval`) y se publican como nueva versión (`kaizen-release`)."]
    os.makedirs(os.path.dirname(OUT_MD), exist_ok=True)
    open(OUT_MD, "w", encoding="utf-8").write("\n".join(lines) + "\n")
    print(f"catálogo: {catalog['stats']['rea_tools']} herramientas, {len(families)} familias, v{version}")


if __name__ == "__main__":
    main()
