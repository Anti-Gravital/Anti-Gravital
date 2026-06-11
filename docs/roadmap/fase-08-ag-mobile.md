# Fase 8 - ag-mobile Flutter bridge

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-07-ag-migrate.md](./fase-07-ag-migrate.md)
> Siguiente: [fase-09-plugins-wasi.md](./fase-09-plugins-wasi.md)

## Phase 8 — `ag-mobile` Flutter bridge

**Objective.** Build the integration with Flutter as the priority mobile target. Generation of complete Dart SDK, native auth, realtime.

### 8.1 Entry criteria

- [ ] Phase 7 completed.
- [ ] At least one collaborator with significant Flutter experience has joined the project.

### 8.2 Deliverables

- [ ] `ag-mobile` crate with Dart generator.
- [ ] `anti_gravital` pub package published on pub.dev:
  - [ ] Types generated with freezed.
  - [ ] HTTP client with dio + interceptors.
  - [ ] WebSocket client.
  - [ ] SSE client.
  - [ ] Mocks for tests.
- [ ] Authentication widgets: registration and login with native WebAuthn (Android Credential Manager, iOS Passkeys), OAuth2.
- [ ] `flutter-fullstack` example in `examples/`: complete Flutter app with Anti-Gravital backend.
- [ ] Documentation: Flutter user guide.

### 8.3 Exit criteria (gate before Phase 9)

- [ ] The `anti_gravital` package on pub.dev has at least 50 likes.
- [ ] The `flutter-fullstack` example runs on Android, iOS and web.
- [ ] At least one external Flutter application uses Anti-Gravital in staging or production.
- [ ] At least 4 500 stars on the repository.

### 8.4 Phase risks

The main risk is that the Rust → Dart context switch has unforeseen frictions. The mitigation is to start with the simplest case (CRUD) and build incrementally.

---

