# Capitulo 13. Puente de aplicaciones nativas (ag-mobile)

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 13
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [12-migracion-ag-migrate.md](./12-migracion-ag-migrate.md)
> Siguiente: [14-observabilidad-ag-observe.md](./14-observabilidad-ag-observe.md)

## 13. Native application bridge (`ag-mobile`): Flutter and generated clients

The most important repositioning derived from the critical analysis is that Anti-Gravital does not compete with Flutter. It positions itself as **the ideal native backend for Flutter applications**. This multiplies the strategic value of the project: instead of competing with a mature and very well designed cross-platform UI framework, Anti-Gravital becomes its natural companion.

### 13.1 Dart SDK generation

`ag-mobile` generates a complete Dart package from the `schema.ag`. The package includes types generated with freezed for immutability, an HTTP client based on dio with interceptors for authentication, a WebSocket client for realtime, Server-Sent Events support, and mocks for tests.

### 13.2 Native Flutter authentication

The module includes widgets and services ready for the authentication flows. The WebAuthn integration leverages the native platforms (Android Credential Manager API, iOS Passkeys via AuthenticationServices). The OAuth2 flow uses `flutter_appauth` with auto-generated configuration.

### 13.3 Offline-first and synchronization

`ag-mobile` offers an optional offline synchronization layer. Operations are queued locally in a SQLite database, replicated to the server when there is connectivity, and conflicts are resolved with declarative policies in the schema (last-write-wins, custom merge, server-wins). It is an ambitious functionality that is implemented in a late phase of the roadmap.

### 13.4 Other generated clients

Although Flutter is the priority target for mobile, the codegen system is extensible. Version 1.0 includes generators for Dart (Flutter), TypeScript (React, Vue, Svelte, Next.js), and Kotlin (native Android, optionally Kotlin Multiplatform). A later version may include Swift and Python.

---

