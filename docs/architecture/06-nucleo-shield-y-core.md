# Capitulo 6. Arquitectura del nucleo (ag-core): Shield y Core

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md, seccion 6
> Indice: [docs/architecture/README.md](./README.md)
> Anterior: [05-ecosistema-modulos.md](./05-ecosistema-modulos.md)
> Siguiente: [07-anti-dsl.md](./07-anti-dsl.md)

## 6. Arquitectura del núcleo (`ag-core`): Shield y Core

El núcleo de Anti-Gravital se organiza en dos capas conceptuales dentro de un único proceso Rust. La separación no es física: no hay IPC, no hay FFI, no hay shared memory entre runtimes. Las dos capas se comunican mediante llamadas de función Rust ordinarias, con cero overhead medible. La separación es lógica y existe por dos razones: claridad arquitectónica para el desarrollador, y posibilidad futura de extraer la Shield como gateway independiente si un caso de uso lo justifica.

### 6.1 Diagrama del núcleo

```
┌─────────────────────────────────────────────────────────────┐
│                  ag-core · Single Process                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────────────────────────────────────────────┐       │
│   │            CAPA A — The Shield                  │       │
│   │  (Tower middleware composable pipeline)         │       │
│   │                                                 │       │
│   │   ┌─────────┐  ┌─────────┐  ┌─────────────┐     │       │
│   │   │ TLS 1.3 │─►│   JWT   │─►│ Rate Limit  │     │       │
│   │   │ rustls  │  │ Ed25519 │  │  governor   │     │       │
│   │   └─────────┘  └─────────┘  └──────┬──────┘     │       │
│   │                                    │            │       │
│   │   ┌─────────┐  ┌─────────┐  ┌──────▼──────┐     │       │
│   │   │  CORS   │◄─│  RBAC   │◄─│ Validación  │     │       │
│   │   │  CSRF   │  │ Guards  │  │  Schema     │     │       │
│   │   └─────────┘  └─────────┘  └─────────────┘     │       │
│   └─────────────────────┬───────────────────────────┘       │
│                         │                                   │
│              Llamada de función Rust (0ns)                  │
│                         ▼                                   │
│   ┌─────────────────────────────────────────────────┐       │
│   │            CAPA B — The Core                    │       │
│   │  (Axum router · Handlers · Estado)              │       │
│   │                                                 │       │
│   │   ┌────────────┐   ┌─────────────────────┐      │       │
│   │   │  Router    │   │  Business handlers  │      │       │
│   │   │  Axum      │──►│  (generados por DSL)│      │       │
│   │   └────────────┘   └──────────┬──────────┘      │       │
│   │                               │                 │       │
│   │   ┌────────────┐   ┌──────────▼──────────┐      │       │
│   │   │ Extractores│   │  Estado compartido  │      │       │
│   │   │  tipados   │   │  AppState           │      │       │
│   │   └────────────┘   └─────────────────────┘      │       │
│   └─────────────────────────────────────────────────┘       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                            │
                  cargo build --release
                            ▼
                ┌────────────────────────┐
                │  Single Static Binary  │
                │  FROM scratch Docker   │
                └────────────────────────┘
```

### 6.2 The Shield: la capa de confianza

La Shield es responsable de todo lo que ocurre antes de que un request sea considerado confiable y entregado al código de negocio. Está implementada como una pipeline de capas Tower, el mismo modelo composable que Axum usa internamente. Cada capa es opcional y se configura desde el `schema.ag` del proyecto.

El stack técnico de la Shield es: Tokio como runtime async M:N (multiplexa millones de tareas sobre un thread pool fijo de tamaño igual a CPUs disponibles), Tower como modelo de middleware composable, rustls para TLS 1.3 sin dependencia de OpenSSL, serde y serde_json para serialización zero-copy donde es posible, ring para primitivas criptográficas de bajo nivel, governor para rate limiting con algoritmo token bucket sin locks contenciosos.

Las capas estándar de la Shield, en orden de ejecución sobre un request entrante:

La primera capa es la terminación TLS, gestionada por rustls. Soporta TLS 1.3 con cipher suites modernas, OCSP stapling y ALPN para negociación HTTP/1.1 vs HTTP/2. Para entornos donde la terminación TLS la realiza un balanceador externo (Cloudflare, AWS ALB, Nginx), esta capa se desactiva con una opción en el schema.

La segunda capa es la deserialización y validación del payload. Para requests con body, se aplica el contrato definido en el `.ag`: tipos, restricciones de longitud, formato de email, regex, rangos numéricos. Una violación produce un error 422 con detalle estructurado de qué campo falló y por qué.

La tercera capa es la autenticación. Soporta JWT firmado con Ed25519 (curva Edwards25519, más rápida y segura que RS256), Passkeys/WebAuthn (FIDO2), API keys, y sesiones cookie-based. La verificación es eager para endpoints marcados como `auth required`.

La cuarta capa es el rate limiting. Implementado con governor sobre algoritmo token bucket, soporta límites por IP, por usuario autenticado, por endpoint, y por combinaciones. Los límites se declaran en el schema.

La quinta capa es la autorización RBAC. Las políticas se declaran en el `.ag` como expresiones que se evalúan contra los claims del JWT y los parámetros del request. Por ejemplo: `policy "user.role == ADMIN || user.id == params.id"`.

La sexta capa es CORS y CSRF. Configurada por defecto con valores seguros (no wildcard); cualquier desviación requiere declaración explícita.

### 6.3 The Core: la capa de lógica de negocio

The Core es donde vive el 80% del código de aplicación que el desarrollador escribe. Es Axum con una capa fina de convenciones encima.

Los handlers tienen una firma generada por el compilador del DSL a partir del endpoint declarado:

```rust
// Generado automáticamente por `ag generate` desde schema.ag
// El desarrollador solo escribe el cuerpo del handler.
pub async fn create_user(
    State(state): State<AppState>,
    ValidatedBody(req): ValidatedBody<CreateUserRequest>,
    Claims(claims): Claims<AuthClaims>,
) -> Result<Json<User>, AgError> {
    // El desarrollador solo escribe esto:
    let user = state.db.users()
        .create(CreateUserParams {
            email: req.email,
            name: req.name,
            created_by: claims.user_id,
        })
        .await?;

    state.events.emit("user.created", &user).await?;
    Ok(Json(user))
}
```

El tipo `ValidatedBody<T>` garantiza que el body ya pasó la validación de la Shield. El tipo `Claims<T>` garantiza que el JWT ya fue verificado. El tipo `AgError` es un enum que cubre todos los errores declarados en el endpoint, y la conversión a respuesta HTTP es automática vía `IntoResponse`.

El estado de la aplicación (`AppState`) es un struct generado que contiene clientes a los recursos del proyecto: el pool de base de datos, el cliente NATS, el cliente Redis, el cliente S3. Se construye en el arranque del binario y se comparte por referencia (clones baratos de `Arc`) entre todos los handlers.

### 6.4 Manejo de errores

El sistema de errores de Anti-Gravital sigue tres principios. El primero: cada endpoint declara explícitamente qué errores puede producir en su definición `.ag`. Esto produce un enum `EndpointError` tipado del que cada variante es un error específico. El segundo: los errores se propagan con el operador `?` de Rust, y la conversión a respuesta HTTP es automática y consistente. El tercero: ningún error se descarta silenciosamente. Los errores no esperados producen un 500 estructurado con un correlation ID que se enlaza al stack trace en el sistema de tracing.

### 6.5 Runtime y configuración Tokio

Anti-Gravital usa Tokio en modo multi-thread con configuración por defecto: un worker por CPU disponible, blocking pool de 512 threads. Para cargas IO-bound estándar esta configuración es óptima. El schema permite ajustes:

```yaml
runtime:
  workers: auto              # Por defecto = núm CPUs
  blocking_threads: 512
  thread_stack: 2MB
  shutdown_timeout: 30s
```

---

