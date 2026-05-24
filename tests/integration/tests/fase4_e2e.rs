//! Tests de integracion Fase 4.
//!
//! Tests unitarios (uno por modulo) y test cross-module E2E.
//! Ninguno requiere servicios externos — todo es embebido en proceso.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn generate_ed25519_keypair() -> (String, String) {
    use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;
    use ed25519_dalek::pkcs8::{EncodePrivateKey, EncodePublicKey};
    use ed25519_dalek::SigningKey;
    use rand_core::OsRng;

    let signing_key = SigningKey::generate(&mut OsRng);
    let priv_pem = signing_key
        .to_pkcs8_pem(LineEnding::LF)
        .unwrap()
        .to_string();
    let pub_pem = signing_key
        .verifying_key()
        .to_public_key_pem(LineEnding::LF)
        .unwrap();
    (priv_pem, pub_pem)
}

fn uuid_like() -> String {
    let mut b = [0u8; 16];
    getrandom::getrandom(&mut b).unwrap();
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes(b[0..4].try_into().unwrap()),
        u16::from_be_bytes(b[4..6].try_into().unwrap()),
        u16::from_be_bytes(b[6..8].try_into().unwrap()),
        u16::from_be_bytes(b[8..10].try_into().unwrap()),
        {
            let mut n = 0u64;
            for &byte in &b[10..16] {
                n = (n << 8) | byte as u64;
            }
            n
        }
    )
}

fn make_auth_config(priv_pem: String, pub_pem: String) -> ag_auth::AuthConfig {
    ag_auth::AuthConfig {
        jwt_private_key_pem: priv_pem,
        jwt_public_key_pem: pub_pem,
        webauthn_rp_id: String::new(),
        webauthn_origin: String::new(),
        oauth_google_client_id: None,
        oauth_google_client_secret: None,
        oauth_github_client_id: None,
        oauth_github_client_secret: None,
    }
}

fn make_jwt_claims(sub: &str, role: &str) -> ag_auth::Claims {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("reloj del sistema invalido")
        .as_secs();
    ag_auth::Claims {
        sub: sub.to_string(),
        exp: now + 3600,
        iat: now,
        jti: uuid_like(),
        role: role.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests unitarios — uno por modulo
// ---------------------------------------------------------------------------

#[test]
fn test_observe_init() {
    let config = ag_observe::ObserveConfig::default();
    let result = ag_observe::init(&config);
    // Puede fallar con AlreadyInitialized si otro test lo llamo antes; nunca panic.
    assert!(
        result.is_ok() || matches!(result, Err(ag_observe::ObserveError::AlreadyInitialized)),
        "init debe retornar Ok o AlreadyInitialized, no otro error"
    );
}

#[test]
fn test_auth_jwt_sign_verify() {
    let (priv_pem, pub_pem) = generate_ed25519_keypair();
    let config = make_auth_config(priv_pem, pub_pem);
    let auth = ag_auth::AgAuth::new(config, reqwest::Client::new())
        .expect("AgAuth::new debe tener exito con config valida");

    let claims = make_jwt_claims("user-42", "admin");
    let token = auth
        .jwt
        .sign(&claims)
        .expect("firma JWT debe tener exito con clave valida");

    let verified = auth
        .jwt
        .verify(&token)
        .expect("verificacion JWT debe tener exito con token valido");

    assert_eq!(verified.sub, "user-42");
    assert_eq!(verified.role, "admin");
    assert_eq!(verified.jti, claims.jti);
}

#[test]
fn test_auth_api_key_roundtrip() {
    let (priv_pem, pub_pem) = generate_ed25519_keypair();
    let config = make_auth_config(priv_pem, pub_pem);
    let auth = ag_auth::AgAuth::new(config, reqwest::Client::new()).unwrap();

    let (raw_key, hash) = auth.create_api_key("sk");
    assert!(
        raw_key.starts_with("sk_"),
        "raw key debe comenzar con el prefijo sk_"
    );
    assert!(
        auth.verify_api_key(&raw_key, &hash),
        "la API key generada debe verificar correctamente"
    );
    assert!(
        !auth.verify_api_key("sk_wrongkey", &hash),
        "una clave incorrecta debe fallar la verificacion"
    );
}

#[tokio::test]
async fn test_cache_set_get_invalidate() {
    let cache = ag_cache::l1::L1Cache::new(1024u64, Duration::from_secs(300));

    let value = b"perfil-del-usuario".to_vec();
    cache
        .set_bytes_tagged("user:42:profile", value.clone(), &["user:42"])
        .await;

    let hit = cache
        .get_bytes("user:42:profile")
        .await
        .expect("debe existir despues de set");
    assert_eq!(
        hit, value,
        "el valor recuperado debe ser identico al insertado"
    );

    cache.invalidate_tag("user:42").await;

    let miss = cache.get_bytes("user:42:profile").await;
    assert!(
        miss.is_none(),
        "despues de invalidar el tag la entrada debe desaparecer"
    );
}

#[tokio::test]
async fn test_realtime_broadcast_receive() {
    use ag_realtime::{AgRealtime, RealtimeConfig};

    let rt = AgRealtime::new(RealtimeConfig::default())
        .await
        .expect("AgRealtime::new debe tener exito en modo InProcess");

    let bus = rt.bus().expect("modo InProcess debe exponer un EventBus");
    let mut rx = bus.subscribe();

    rt.broadcast("user.created", b"payload".to_vec())
        .expect("broadcast debe tener exito");

    let event = rx.recv().await.expect("debe recibir el evento publicado");
    assert_eq!(event.subject, "user.created");
    assert_eq!(event.payload, b"payload");
}

#[tokio::test]
async fn test_storage_put_get_delete() {
    use ag_storage::{AgStorage, StorageConfig};
    use bytes::Bytes;

    let dir = tempfile::tempdir().expect("debe poder crear directorio temporal");
    let config = StorageConfig {
        root_path: dir.path().to_path_buf(),
        server_mode: false,
        ..StorageConfig::default()
    };

    let storage = AgStorage::new(config)
        .await
        .expect("AgStorage::new debe tener exito");

    let data = Bytes::from("contenido-de-prueba");
    storage
        .put("docs/test.txt", data.clone())
        .await
        .expect("put debe tener exito");

    let got = storage
        .get("docs/test.txt")
        .await
        .expect("get debe recuperar el objeto");
    assert_eq!(
        got, data,
        "el contenido recuperado debe ser identico al almacenado"
    );

    storage
        .delete("docs/test.txt")
        .await
        .expect("delete debe tener exito");

    let exists = storage
        .exists("docs/test.txt")
        .await
        .expect("exists debe operar sin error");
    assert!(!exists, "el objeto no debe existir despues de delete");
}

// ---------------------------------------------------------------------------
// Test E2E cross-module — 15 pasos en secuencia
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_e2e_fase4_cross_module() {
    use ag_cache::l1::L1Cache;
    use ag_realtime::{AgRealtime, RealtimeConfig};
    use ag_storage::{AgStorage, StorageConfig};
    use bytes::Bytes;

    // Paso 1: ag-observe — inicializar tracing (ignorar AlreadyInitialized)
    let observe_config = ag_observe::ObserveConfig::default();
    let _ = ag_observe::init(&observe_config);

    // Paso 2: ag-auth — generar keypair Ed25519 y construir AgAuth
    let (priv_pem, pub_pem) = generate_ed25519_keypair();
    let auth_config = make_auth_config(priv_pem, pub_pem);
    let auth = ag_auth::AgAuth::new(auth_config, reqwest::Client::new())
        .expect("AgAuth::new debe tener exito");

    // Paso 3: ag-auth — firmar JWT con sub: "user-42", role: "admin"
    let claims = make_jwt_claims("user-42", "admin");
    let token = auth.jwt.sign(&claims).expect("firma JWT debe tener exito");

    // Paso 4: ag-auth — verificar JWT y comprobar claims
    let verified = auth
        .jwt
        .verify(&token)
        .expect("verificacion JWT debe tener exito");
    assert_eq!(verified.sub, "user-42", "sub debe ser user-42");
    assert_eq!(verified.role, "admin", "role debe ser admin");

    // Paso 5: ag-cache — construir L1Cache
    let cache = L1Cache::new(1024u64, Duration::from_secs(300));

    // Paso 6: ag-cache — set_bytes_tagged con tag "user:42"
    let profile_json = serde_json::json!({
        "id": "user-42",
        "name": "Test User",
        "role": verified.role,
    });
    let json_bytes =
        serde_json::to_vec(&profile_json).expect("serializacion JSON debe tener exito");
    cache
        .set_bytes_tagged("user:42:profile", json_bytes.clone(), &["user:42"])
        .await;

    // Paso 7: ag-realtime — construir AgRealtime InProcess
    let rt = AgRealtime::new(RealtimeConfig::default())
        .await
        .expect("AgRealtime::new debe tener exito");

    // Paso 8: ag-realtime — suscribirse al subject "user.profile.updated"
    let bus = rt.bus().expect("InProcess debe exponer un EventBus");
    let mut rx = bus.subscribe();

    // Paso 9: ag-realtime — broadcast_json de payload
    let broadcast_payload = serde_json::json!({
        "user_id": "user-42",
        "event": "profile_updated",
    });
    rt.broadcast_json("user.profile.updated", &broadcast_payload)
        .expect("broadcast_json debe tener exito");

    // Paso 10: ag-realtime — verificar que el suscriptor recibio el evento
    let event = rx.recv().await.expect("debe recibir el evento publicado");
    assert_eq!(
        event.subject, "user.profile.updated",
        "el subject del evento debe coincidir"
    );
    let received_val: serde_json::Value =
        serde_json::from_slice(&event.payload).expect("payload debe ser JSON valido");
    assert_eq!(
        received_val["user_id"], "user-42",
        "user_id en el payload debe ser user-42"
    );

    // Paso 11: ag-cache — get("user:42:profile") -> hit
    let cached = cache
        .get_bytes("user:42:profile")
        .await
        .expect("debe encontrarse en cache (hit)");
    assert_eq!(
        cached, json_bytes,
        "los bytes en cache deben ser identicos a los insertados"
    );

    // Paso 12: ag-cache — invalidate_tag("user:42")
    cache.invalidate_tag("user:42").await;

    // Paso 13: ag-cache — get("user:42:profile") -> None (miss)
    let after_invalidation = cache.get_bytes("user:42:profile").await;
    assert!(
        after_invalidation.is_none(),
        "la entrada debe haber sido eliminada por invalidacion del tag"
    );

    // Paso 14: ag-storage — put("avatars/user-42.png", bytes)
    let dir = tempfile::tempdir().expect("debe poder crear directorio temporal");
    let storage_config = StorageConfig {
        root_path: dir.path().to_path_buf(),
        server_mode: false,
        ..StorageConfig::default()
    };
    let storage = AgStorage::new(storage_config)
        .await
        .expect("AgStorage::new debe tener exito");

    let avatar_bytes = Bytes::from(vec![0x89u8, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
    storage
        .put("avatars/user-42.png", avatar_bytes.clone())
        .await
        .expect("put del avatar debe tener exito");

    // Paso 15: ag-storage — get("avatars/user-42.png") -> mismos bytes
    let retrieved = storage
        .get("avatars/user-42.png")
        .await
        .expect("get del avatar debe tener exito");
    assert_eq!(
        retrieved, avatar_bytes,
        "los bytes recuperados deben ser identicos a los almacenados"
    );
}
