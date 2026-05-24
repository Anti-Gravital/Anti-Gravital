//! Demostracion del flujo de autenticacion con envio de correos.
//!
//! Muestra como conectar ag-auth y ag-mail para los tres flujos
//! transaccionales principales:
//!
//! 1. Registro con verificacion de email.
//! 2. Recuperacion de contrasena.
//! 3. Autenticacion sin contrasena (magic link).
//!
//! El ejemplo usa `NullSender` — no requiere servidor SMTP real.
//! Para usar SMTP real, sustituye `NullSender` por `SmtpSender`:
//!
//! ```no_run
//! use ag_mail::sender::smtp::{SmtpConfig, SmtpSender};
//! let config = SmtpConfig::new("smtp.example.com", 587, "user", "pass");
//! let sender = SmtpSender::new(config)?;
//! ```

use std::sync::Arc;

use ag_auth::{mailer::AuthMailer, AgAuth, AuthConfig};
use ag_mail::sender::test_utils::NullSender;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // --- Configuracion de autenticacion (valores de demo) ---
    let auth_config = AuthConfig {
        // En produccion estas claves se generan con: ag new --with-auth
        jwt_private_key_pem: "demo-private-key".to_owned(),
        jwt_public_key_pem: "demo-public-key".to_owned(),
        webauthn_rp_id: String::new(),
        webauthn_origin: String::new(),
        oauth_google_client_id: None,
        oauth_google_client_secret: None,
        oauth_github_client_id: None,
        oauth_github_client_secret: None,
    };

    // --- NullSender: captura los correos sin enviarlos ---
    let null_sender = Arc::new(NullSender::new());
    let sender: Arc<dyn ag_mail::sender::MailSender> = null_sender.clone();

    // --- AuthMailer inyectado en AgAuth ---
    let mailer = Arc::new(AuthMailer::new(
        sender,
        "no-reply@miapp.example.com",
        "MiApp Demo",
    ));

    let _auth = AgAuth::new(auth_config, reqwest::Client::new())
        .expect("configuracion de auth valida")
        .with_mail(Arc::clone(&mailer));

    println!("ag-auth + ag-mail conectados.");
    println!();

    let base_url = "https://miapp.example.com";

    // ---- Flujo 1: Registro con verificacion de email ---
    println!("--- Flujo 1: verificacion de email ---");
    let verification_token = ag_auth::generate_api_key("vrf").0;
    println!("Token de verificacion generado: {verification_token}");

    mailer
        .send_verification("nueva@usuario.com", &verification_token, base_url)
        .await
        .expect("envio de verificacion");

    println!(
        "Correo de verificacion capturado por NullSender. Total enviados: {}",
        null_sender.emails_sent()
    );
    if let Some(email) = null_sender.last_email() {
        println!("  Asunto: {}", email.subject);
        println!("  Enlace: {base_url}/verify?token={verification_token}");
    }
    println!();

    // ---- Flujo 2: Recuperacion de contrasena ---
    println!("--- Flujo 2: recuperacion de contrasena ---");
    let reset_token = ag_auth::generate_api_key("rst").0;
    println!("Token de recuperacion generado: {reset_token}");

    mailer
        .send_password_reset("usuario@ejemplo.com", &reset_token, base_url)
        .await
        .expect("envio de recuperacion");

    println!(
        "Correo de recuperacion capturado. Total enviados: {}",
        null_sender.emails_sent()
    );
    println!();

    // ---- Flujo 3: Magic link ---
    println!("--- Flujo 3: magic link ---");
    let magic_token = ag_auth::generate_api_key("mgk").0;
    println!("Token de magic link generado: {magic_token}");

    mailer
        .send_magic_link("usuario@ejemplo.com", &magic_token, base_url)
        .await
        .expect("envio de magic link");

    println!(
        "Correo de magic link capturado. Total enviados: {}",
        null_sender.emails_sent()
    );
    println!();

    // ---- Resumen ---
    println!(
        "Demo completado: {} correos enviados sin necesitar SMTP real.",
        null_sender.emails_sent()
    );
    println!("Para usar SMTP real, sustituye NullSender por SmtpSender en el codigo.");
}
