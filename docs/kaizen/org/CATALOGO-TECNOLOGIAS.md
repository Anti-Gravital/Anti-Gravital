# Catálogo de skills y tecnologías — v1.0.0

Generado: 2026-11-22T05:52:38.269114+00:00Z | Fuente: REA morluto/rea (MIT) + kaizen-swarm

Herramientas REA: **263** | Familias: 11 | Proveedores: 5 | Skills del framework: 12

> Contenido externo = dato no confiable (S-15/L-5). La instalación de REA fue aprobada por El Presidente (ADR-0009). Las operaciones con efectos (`launches_process`, `may_write_filesystem`, etc.) requieren el proceso `reverse_engineering`.

## Familias de capacidades REA

| Familia | Herramientas | Proveedores | Efectos |
| --- | ---: | --- | --- |
| Android / Flutter (`android`) | 37 | rea-mcp | - |
| Artefactos y datos (`artifacts`) | 11 | native-macos, rea-artifact-graph, rea-mcp | launches_process, may_write_filesystem |
| Entornos de agente y build computer (`buildenv`) | 12 | rea-mcp | - |
| Evidencia, comparación y verificación (`evidence`) | 17 | rea-mcp | - |
| Firmware / kernel / dispositivos (`firmware`) | 9 | rea-mcp | - |
| Managed / .NET / Swift / ObjC (`managed`) | 19 | native-macos, rea-artifact-graph, rea-dotnet-static | launches_process, may_write_filesystem |
| Binarios nativos y funciones (`native`) | 51 | native-macos, rea-mcp | launches_process, may_access_network, may_show_ui |
| Red, capturas y señales (`network`) | 5 | rea-mcp | - |
| Inspección general (`other`) | 35 | native-macos, rea-mcp | launches_process |
| Runtime, debug e instrumentación (`runtime`) | 30 | native-macos, rea-artifact-graph, rea-mcp | launches_process, may_access_network, may_show_ui |
| Web / JavaScript / Electron (`web`) | 37 | rea-mcp | - |

### Operaciones por familia

**Android / Flutter** (37): `capture_adb_screen`, `collect_adb_bugreport`, `decode_android_resources`, `identify_flutter_build`, `inspect_adb_client`, `inspect_adb_device`, `inspect_adb_display`, `inspect_adb_package`, `inspect_adb_window`, `inspect_android_class`, `inspect_android_jni_bindings`, `inspect_android_method`, `inspect_android_package`, `inspect_apk_signature`, `inspect_apktool_client`, `inspect_dart_aot`, `install_adb_package`, `list_adb_devices`, `list_adb_directory`, `list_adb_features`, `list_adb_packages`, `list_adb_processes`, `list_adb_services`, `project_android_application_graph`, `project_harmony_application_graph`, `pull_adb_file`, `pull_adb_package`, `push_adb_file`, `read_adb_logcat`, `read_adb_setting`, `resolve_adb_packages`, `search_android_classes`, `start_adb_activity`, `start_adb_app`, `stop_adb_app`, `trace_android_references`, `uninstall_adb_package`

**Artefactos y datos** (11): `extract_artifact`, `inspect_artifact`, `inspect_asset_catalog`, `inspect_keyed_archive`, `inspect_model_artifact`, `inspect_ntfs_journal`, `inspect_plist`, `inspect_signature`, `inspect_sqlite_database`, `inspect_wasm_artifact`, `inspect_windows_shortcut`

**Entornos de agente y build computer** (12): `act_on_build_computer`, `build_call_path`, `check_build_task`, `close_build_computer`, `mark_build_checkpoint`, `measure_build_task`, `read_guidance_section`, `record_build_consent`, `search_guidance`, `start_build_capture`, `start_build_computer`, `stop_build_capture`

**Evidencia, comparación y verificación** (17): `audit_release_exposure`, `build_reconstruction_obligation_ledger`, `compare_application_versions`, `compare_artifacts`, `compare_functions`, `compare_process_captures`, `compare_static_and_observed_endpoints`, `compare_wasm_interfaces`, `evaluate_reconstruction_coverage`, `export_build_environment`, `inspect_native_exports`, `inspect_pe_imports`, `list_unknowns`, `record_unknown`, `update_unknown`, `verify_reconstruction`, `verify_unknown_resolution`

**Firmware / kernel / dispositivos** (9): `decode_signal_recording`, `extract_firmware`, `extract_uefi_firmware`, `inspect_devicetree`, `inspect_firmware_regions`, `inspect_kernel_build_config`, `inspect_signal_decoders`, `inspect_uefi_firmware`, `inspect_uefi_nvram`

**Managed / .NET / Swift / ObjC** (19): `analyze_swift_types`, `compare_managed_members`, `decode_interface_builder`, `demangle_swift`, `get_objc_classes`, `get_objc_protocols`, `import_managed_reconstruction`, `inspect_erlang_beam`, `inspect_evm_interface`, `inspect_il2cpp_metadata`, `inspect_macho`, `inspect_managed_artifact`, `inspect_managed_members`, `inspect_managed_native_boundaries`, `inspect_maven_gradle_metadata`, `inspect_nuget_nupkg`, `inspect_python_pyc`, `project_managed_application_graph`, `verify_managed_native_boundaries`

**Binarios nativos y funciones** (51): `address_name`, `address_to_file_offset`, `analyze_function`, `annotate_native_function`, `batch_decompile`, `binary_overview`, `capture_native_ui_scenario`, `close_binary`, `current_address`, `current_procedure`, `decompile_all`, `decompile_jeb_item`, `find_xrefs_to_name`, `get_call_graph`, `goto_address`, `inspect_address_context`, `inspect_binary_layout`, `inspect_elf_dependencies`, `inspect_elf_notes`, `inspect_elf_unwind_ranges`, `inspect_go_binary`, `inspect_native_api`, `inspect_native_data_type`, `inspect_native_dispatch_metadata`, `inspect_native_initializers`, `inspect_native_instruction`, `inspect_native_load_image`, `inspect_pe_certificates`, `inspect_pe_load_config`, `inspect_pe_resources`, `inspect_rust_binary`, `inspect_with_objdump`, `list_procedures`, `match_native_functions`, `next_address`, `open_binary`, `prev_address`, `procedure_address`, `procedure_assembly`, `procedure_callees`, `procedure_callers`, `procedure_info`, `procedure_pseudo_code`, `procedure_references`, `read_function_instructions`, `resolve_containing_procedure`, `resolve_native_call_targets`, `search_procedures`, `set_address_name`, `set_addresses_names`, `xrefs`

**Red, capturas y señales** (5): `capture_process_scenario`, `inspect_packet_capture`, `inspect_packet_fields`, `inspect_pcap_capture`, `inspect_pcap_client`

**Inspección general** (35): `comment`, `correlate_static_and_runtime`, `current_document`, `find_changed_behavior`, `get_navigation_context`, `guided_workflows`, `inline_comment`, `inspect_analysis_settings`, `inspect_analysis_view`, `inspect_api_contract`, `inspect_apple_ips`, `inspect_byte_entropy`, `inspect_desktop_application`, `inspect_dwarf_units`, `inspect_gpu_code_objects`, `inspect_jeb_client`, `inspect_recorded_crash`, `list_architectures`, `list_bookmarks`, `list_documents`, `list_jeb_units`, `list_names`, `list_segments`, `list_strings`, `open_jeb_project`, `project_apple_application_graph`, `read_bytes`, `reset_investigation`, `search_bytes`, `search_strings`, `set_bookmark`, `set_comment`, `set_inline_comment`, `unset_bookmark`, `update_analysis_settings`

**Runtime, debug e instrumentación** (30): `binary_session`, `close_frida_session`, `close_gdb_session`, `close_rizin_debug_session`, `cutter_command`, `execute_rizin_command`, `frida_session_status`, `gdb_console`, `gdb_session_status`, `inspect_elf_debug_link`, `instrument_with_frida`, `list_cutter_sessions`, `list_frida_devices`, `list_frida_processes`, `load_frida_script`, `observe_native_calls`, `observe_native_ui`, `resume_frida_session`, `rizin_debug_command`, `rizin_debug_session_status`, `start_frida_session`, `start_gdb_session`, `start_rizin_debug_session`, `trace_application_feature`, `trace_call_path`, `trace_dylib_resolution`, `trace_feature`, `trace_native_ui_action`, `trace_native_values`, `unload_frida_script`

**Web / JavaScript / Electron** (37): `analyze_javascript_application`, `analyze_web_bundle`, `capture_browser_scenario`, `capture_electron_scenario`, `capture_web_screenshot`, `compare_bundles`, `compare_javascript_export_shapes`, `compare_source_to_bundle`, `compare_web_captures`, `compare_web_screenshots`, `discover_webmcp_tools`, `export_environment_bundle`, `export_evidence_bundle`, `export_web_scripts`, `get_evidence_bundle`, `import_evidence_bundle`, `inspect_binary_hardening`, `inspect_electron_fuses`, `inspect_electron_page`, `inspect_kernel_module`, `inspect_playwright_trace`, `inspect_v8_code_cache`, `inspect_web_event_listeners`, `inspect_web_network_capture`, `inspect_web_page`, `list_browser_targets`, `list_electron_targets`, `list_javascript_runtime_targets`, `observe_javascript_runtime`, `observe_web_execution`, `observe_web_session`, `open_bundle_resources`, `reconcile_javascript_runtime`, `recover_javascript_sources`, `trace_javascript_semantics`, `trace_web_module_imports`, `trace_web_source_location`

## Skills del framework

| Skill | Propósito | Ubicación | Dueño |
| --- | --- | --- | --- |
| `kaizen` | Gobernanza, ratchet, PDCA y evidencia | `scripts/kaizen` | todos |
| `kaizen-security` | Secretos, supresiones y SAST | `scripts/kaizen-security` | seguridad |
| `kaizen-swarm` | Comunidad por issue, worktrees y leases | `scripts/kaizen-swarm` | equipo |
| `kaizen-org` | Inventario de agentes, conocimiento y outsourcing | `scripts/kaizen-org` | equipo |
| `kaizen-team` | Escuadras, standups y decisiones | `scripts/kaizen-team` | equipo |
| `kaizen-eval` | Evaluación y ratchet de agentes | `scripts/kaizen-eval` | calidad |
| `kaizen-learn` | Aprendizaje a partir de errores | `scripts/kaizen-learn` | calidad |
| `kaizen-gov` | RACI, obligaciones y derechos | `scripts/kaizen-gov` | gobernanza |
| `kaizen-runtime` | Runtime proactivo y equipos | `scripts/kaizen-runtime` | operación |
| `kaizen-installer` | Instalación en repos y OpenCode | `scripts/kaizen-installer` | distribución |
| `kaizen-notify / kaizen-inbox` | WhatsApp/n8n y correo | `scripts/kaizen-notify` | comunicación |
| `reverse-engineer-anything` | Skill REA de investigación guiada | `~/.agents/skills/reverse-engineer-anything` | rea-specialist |

## Automejora del equipo

- kaizen-eval mide la definición
- kaizen-learn convierte errores en experiencia
- kaizen-org copy/save especializa agentes
- kaizen-release publica y propaga versiones

Las iteraciones producen lecciones (`kaizen-learn`), mejoran definiciones (`kaizen-org save`), suben la puntuación (`kaizen-eval`) y se publican como nueva versión (`kaizen-release`).
