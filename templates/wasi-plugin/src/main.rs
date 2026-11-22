//! WASI plugin host skeleton.
//!
//! This scaffold defines the host-side lifecycle: manifest discovery and
//! validation, a capability policy, a registry and explicit load/unload
//! transitions. It does not execute WebAssembly modules. Execution is the
//! responsibility of `ag-wasm-host`; at the pinned revision that crate is a
//! Phase 0 skeleton without a public API, so the single execution hook below
//! is marked and intentionally left local.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// A capability a plugin may request from the host.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Capability {
    FsRead,
    FsWrite,
    NetOut,
    EnvRead,
}

impl Capability {
    fn as_str(&self) -> &'static str {
        match self {
            Capability::FsRead => "fs_read",
            Capability::FsWrite => "fs_write",
            Capability::NetOut => "net_out",
            Capability::EnvRead => "env_read",
        }
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Description of a plugin, independent of how it is discovered.
#[derive(Clone, Debug)]
struct PluginManifest {
    name: String,
    module_path: String,
    capabilities: Vec<Capability>,
}

/// Failures produced by the host lifecycle.
#[derive(Debug)]
enum HostError {
    Duplicate(String),
    NotFound(String),
    MissingModule(String),
    CapabilityDenied { plugin: String, capability: Capability },
}

impl fmt::Display for HostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HostError::Duplicate(name) => {
                write!(formatter, "plugin '{name}' is already registered")
            }
            HostError::NotFound(name) => write!(formatter, "plugin '{name}' is not registered"),
            HostError::MissingModule(name) => {
                write!(formatter, "plugin '{name}' has no module path set")
            }
            HostError::CapabilityDenied { plugin, capability } => {
                write!(formatter, "plugin '{plugin}' requests denied capability '{capability}'")
            }
        }
    }
}

impl Error for HostError {}

/// Registry with an explicit capability allow-list.
struct PluginRegistry {
    plugins: HashMap<String, PluginManifest>,
    policy: Vec<Capability>,
}

impl PluginRegistry {
    fn new(policy: Vec<Capability>) -> Self {
        Self {
            plugins: HashMap::new(),
            policy,
        }
    }

    /// Validate and store a manifest. Rejects duplicates and capabilities
    /// that are not in the host policy.
    fn register(&mut self, manifest: PluginManifest) -> Result<(), HostError> {
        if self.plugins.contains_key(&manifest.name) {
            return Err(HostError::Duplicate(manifest.name));
        }
        for capability in &manifest.capabilities {
            if !self.policy.contains(capability) {
                return Err(HostError::CapabilityDenied {
                    plugin: manifest.name.clone(),
                    capability: capability.clone(),
                });
            }
        }
        self.plugins.insert(manifest.name.clone(), manifest);
        Ok(())
    }

    fn get(&self, name: &str) -> Option<&PluginManifest> {
        self.plugins.get(name)
    }

    /// Registered names, sorted for deterministic output.
    fn list(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.plugins.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }

    fn unload(&mut self, name: &str) -> Result<PluginManifest, HostError> {
        self.plugins
            .remove(name)
            .ok_or_else(|| HostError::NotFound(name.to_owned()))
    }
}

/// Host that owns the registry and tracks which plugins are loaded.
struct WasiHost {
    registry: PluginRegistry,
    loaded: Vec<String>,
}

impl WasiHost {
    fn new(policy: Vec<Capability>) -> Self {
        Self {
            registry: PluginRegistry::new(policy),
            loaded: Vec::new(),
        }
    }

    fn load(&mut self, name: &str) -> Result<(), HostError> {
        let manifest = self
            .registry
            .get(name)
            .ok_or_else(|| HostError::NotFound(name.to_owned()))?;
        if manifest.module_path.is_empty() {
            return Err(HostError::MissingModule(name.to_owned()));
        }
        // EXECUTION HOOK: reserved for the ag-wasm-host public API. The local
        // skeleton records the transition instead of instantiating a runtime.
        self.loaded.push(name.to_owned());
        Ok(())
    }

    fn unload(&mut self, name: &str) -> Result<(), HostError> {
        let position = self
            .loaded
            .iter()
            .position(|loaded| loaded == name)
            .ok_or_else(|| HostError::NotFound(name.to_owned()))?;
        self.loaded.remove(position);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let mut host = WasiHost::new(vec![Capability::FsRead, Capability::EnvRead]);

    // A manifest is normally discovered from a manifest file. Here it is
    // assembled from code so the scaffold has no external input requirement.
    let demo = PluginManifest {
        name: "demo-transform".to_owned(),
        module_path: std::env::var("WASI_MODULE_PATH").unwrap_or_default(),
        capabilities: vec![Capability::FsRead],
    };
    host.registry.register(demo)?;

    // Demonstrate the capability policy: a manifest asking for permissions
    // outside the host allow-list is rejected at registration time.
    let rejected = PluginManifest {
        name: "deny-demo".to_owned(),
        module_path: "unused.wasm".to_owned(),
        capabilities: vec![Capability::NetOut, Capability::FsWrite],
    };
    match host.registry.register(rejected) {
        Ok(()) => tracing::error!("policy check unexpectedly accepted the manifest"),
        Err(error) => tracing::info!(%error, "manifest rejected by capability policy"),
    }

    tracing::info!(plugins = ?host.registry.list(), "plugin registry ready");

    // Loading succeeds only when a module path is supplied externally. With
    // no path the host stays in the registered-but-not-loaded state.
    if std::env::var("WASI_MODULE_PATH").is_ok() {
        host.load("demo-transform")?;
        tracing::info!(loaded = ?host.loaded, "plugin loaded");
        host.unload("demo-transform")?;
        tracing::info!(loaded = ?host.loaded, "plugin unloaded");
    } else {
        tracing::info!("WASI_MODULE_PATH is unset; skipping the load transition");
    }

    // Remove the demo from the registry to complete the lifecycle.
    let removed = host.registry.unload("demo-transform")?;
    tracing::info!(plugin = %removed.name, "plugin removed from registry");

    println!("{{name}} WASI host skeleton: lifecycle demo completed");
    Ok(())
}
