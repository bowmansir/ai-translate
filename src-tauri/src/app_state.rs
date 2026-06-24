use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    env, fs,
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};

#[cfg(target_os = "windows")]
use std::{ptr::null_mut, slice};
#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    Foundation::LocalFree,
    Security::Cryptography::{
        CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    },
};

const KEYRING_SERVICE: &str = "ai-translate";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub protocol: String,
    pub base_url: String,
    pub model: String,
    pub agent_prompt: String,
    #[serde(skip_serializing)]
    pub api_key: Option<String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            id: "mymemory".to_string(),
            name: "MyMemory 公共接口".to_string(),
            protocol: "mymemory".to_string(),
            base_url: String::new(),
            model: String::new(),
            agent_prompt: default_agent_prompt(),
            api_key: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ProviderConfigView {
    pub id: String,
    pub name: String,
    pub protocol: String,
    pub base_url: String,
    pub model: String,
    pub agent_prompt: String,
    pub api_key_configured: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProviderStateView {
    pub active_provider_id: String,
    pub providers: Vec<ProviderConfigView>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeSettings {
    pub shortcut: String,
    #[serde(default = "default_main_shortcut")]
    pub main_shortcut: String,
    pub startup_mode: String,
    #[serde(default = "default_theme")]
    pub theme: String,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            shortcut: default_selection_shortcut(),
            main_shortcut: default_main_shortcut(),
            startup_mode: "main".to_string(),
            theme: default_theme(),
        }
    }
}

pub struct AppState {
    last_trigger_at: Mutex<Option<Instant>>,
    providers: Mutex<Vec<ProviderConfig>>,
    active_provider_id: Mutex<String>,
    settings: Mutex<RuntimeSettings>,
    popup_payload: Mutex<Option<Value>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            last_trigger_at: Mutex::new(None),
            providers: Mutex::new(vec![
                ProviderConfig::default(),
                ProviderConfig {
                    id: "deepseek".to_string(),
                    name: "DeepSeek".to_string(),
                    protocol: "openai".to_string(),
                    base_url: "https://api.deepseek.com".to_string(),
                    model: "deepseek-v4-flash".to_string(),
                    agent_prompt: default_agent_prompt(),
                    api_key: None,
                },
            ]),
            active_provider_id: Mutex::new("mymemory".to_string()),
            settings: Mutex::new(RuntimeSettings::default()),
            popup_payload: Mutex::new(None),
        }
    }

    pub fn load_from_disk(&self, app: &AppHandle) -> anyhow::Result<()> {
        let path = config_file_path(app)?;
        if !path.exists() {
            if let Some(legacy_path) = legacy_config_file_path(app) {
                if legacy_path.exists() {
                    return self.load_config_from_path(app, legacy_path);
                }
            }

            self.hydrate_provider_keys(app);
            let _ = self.persist_loaded_provider_keys(app);
            self.save_to_disk(app)?;
            return Ok(());
        }

        self.load_config_from_path(app, path)
    }

    fn load_config_from_path(&self, app: &AppHandle, path: PathBuf) -> anyhow::Result<()> {
        let config: PersistedAppConfig = serde_json::from_str(&fs::read_to_string(path)?)?;
        let mut providers = default_providers();
        for persisted in config.providers {
            let provider = persisted.into_provider();
            if let Some(existing) = providers.iter_mut().find(|item| item.id == provider.id) {
                *existing = provider;
            } else {
                providers.push(provider);
            }
        }

        hydrate_provider_keys(app, &mut providers);

        let active_provider_id = if providers
            .iter()
            .any(|provider| provider.id == config.active_provider_id)
        {
            config.active_provider_id
        } else {
            "mymemory".to_string()
        };

        *self.providers.lock().expect("providers mutex poisoned") = providers;
        *self
            .active_provider_id
            .lock()
            .expect("active provider mutex poisoned") = active_provider_id;
        let settings = migrate_runtime_settings(config.settings);
        *self.settings.lock().expect("settings mutex poisoned") = settings;
        let _ = persist_provider_keys(
            app,
            &self.providers.lock().expect("providers mutex poisoned"),
        );
        self.save_to_disk(app)?;

        Ok(())
    }

    pub fn should_accept_trigger(&self, debounce: Duration) -> bool {
        let mut last = self
            .last_trigger_at
            .lock()
            .expect("debounce mutex poisoned");
        let now = Instant::now();

        if let Some(prev) = *last {
            if now.duration_since(prev) < debounce {
                return false;
            }
        }

        *last = Some(now);
        true
    }

    pub fn provider_config(&self) -> ProviderConfig {
        let active_provider_id = self
            .active_provider_id
            .lock()
            .expect("active provider mutex poisoned")
            .clone();
        let providers = self.providers.lock().expect("providers mutex poisoned");
        providers
            .iter()
            .find(|provider| provider.id == active_provider_id)
            .cloned()
            .or_else(|| providers.first().cloned())
            .unwrap_or_default()
    }

    pub fn provider_state_view(&self) -> ProviderStateView {
        let active_provider_id = self
            .active_provider_id
            .lock()
            .expect("active provider mutex poisoned")
            .clone();
        let providers = self.providers.lock().expect("providers mutex poisoned");
        ProviderStateView {
            active_provider_id,
            providers: providers.iter().map(provider_view).collect(),
        }
    }

    pub fn set_active_provider(
        &self,
        app: &AppHandle,
        provider_id: String,
    ) -> anyhow::Result<ProviderStateView> {
        let provider_id = provider_id.trim().to_string();
        let providers = self.providers.lock().expect("providers mutex poisoned");
        providers
            .iter()
            .find(|provider| provider.id == provider_id)
            .ok_or_else(|| anyhow::anyhow!("provider not found"))?;
        drop(providers);

        let mut active_provider_id = self
            .active_provider_id
            .lock()
            .expect("active provider mutex poisoned");
        *active_provider_id = provider_id;
        drop(active_provider_id);

        self.save_to_disk(app)?;
        Ok(self.provider_state_view())
    }

    pub fn save_provider_config(
        &self,
        app: &AppHandle,
        config: ProviderConfig,
    ) -> anyhow::Result<ProviderStateView> {
        let config = normalize_provider_config(config)?;
        let mut providers = self.providers.lock().expect("providers mutex poisoned");
        let existing_key = providers
            .iter()
            .find(|provider| provider.id == config.id)
            .and_then(|provider| provider.api_key.clone());

        if config.protocol != "mymemory" && config.api_key.is_none() && existing_key.is_none() {
            anyhow::bail!("api key is required");
        }

        if let Some(existing) = providers
            .iter_mut()
            .find(|provider| provider.id == config.id)
        {
            if let Some(api_key) = config.api_key.clone() {
                store_provider_key(app, &config.id, &api_key)?;
                *existing = config;
            } else {
                *existing = config;
                existing.api_key = existing_key;
            }
        } else {
            if let Some(api_key) = config.api_key.as_ref() {
                store_provider_key(app, &config.id, api_key)?;
            }
            providers.push(config);
        }
        drop(providers);

        self.save_to_disk(app)?;

        Ok(self.provider_state_view())
    }

    pub fn settings(&self) -> RuntimeSettings {
        self.settings
            .lock()
            .expect("settings mutex poisoned")
            .clone()
    }

    pub fn update_settings(
        &self,
        app: &AppHandle,
        settings: RuntimeSettings,
    ) -> anyhow::Result<RuntimeSettings> {
        if settings.shortcut.trim().is_empty() {
            anyhow::bail!("shortcut is required");
        }
        if settings.main_shortcut.trim().is_empty() {
            anyhow::bail!("main window shortcut is required");
        }
        if settings.shortcut == settings.main_shortcut {
            anyhow::bail!("translate shortcut and main window shortcut cannot be the same");
        }
        if !matches!(settings.startup_mode.as_str(), "main" | "tray") {
            anyhow::bail!("unsupported startup mode");
        }
        if !matches!(settings.theme.as_str(), "dark" | "compact" | "light") {
            anyhow::bail!("unsupported theme");
        }

        let mut current = self.settings.lock().expect("settings mutex poisoned");
        *current = settings;
        let saved = current.clone();
        drop(current);

        self.save_to_disk(app)?;
        Ok(saved)
    }

    pub fn set_popup_payload(&self, payload: Value) {
        let mut current = self
            .popup_payload
            .lock()
            .expect("popup payload mutex poisoned");
        *current = Some(payload);
    }

    pub fn popup_payload(&self) -> Option<Value> {
        self.popup_payload
            .lock()
            .expect("popup payload mutex poisoned")
            .clone()
    }

    fn hydrate_provider_keys(&self, app: &AppHandle) {
        let mut providers = self.providers.lock().expect("providers mutex poisoned");
        hydrate_provider_keys(app, &mut providers);
    }

    fn persist_loaded_provider_keys(&self, app: &AppHandle) -> anyhow::Result<()> {
        let providers = self.providers.lock().expect("providers mutex poisoned");
        persist_provider_keys(app, &providers)
    }

    fn save_to_disk(&self, app: &AppHandle) -> anyhow::Result<()> {
        let path = config_file_path(app)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let providers = self.providers.lock().expect("providers mutex poisoned");
        let active_provider_id = self
            .active_provider_id
            .lock()
            .expect("active provider mutex poisoned")
            .clone();
        let settings = self
            .settings
            .lock()
            .expect("settings mutex poisoned")
            .clone();
        let persisted = PersistedAppConfig {
            active_provider_id,
            settings,
            providers: providers
                .iter()
                .map(PersistedProviderConfig::from)
                .collect(),
        };

        fs::write(path, serde_json::to_string_pretty(&persisted)?)?;
        Ok(())
    }
}

fn default_providers() -> Vec<ProviderConfig> {
    vec![
        ProviderConfig::default(),
        ProviderConfig {
            id: "deepseek".to_string(),
            name: "DeepSeek".to_string(),
            protocol: "openai".to_string(),
            base_url: "https://api.deepseek.com".to_string(),
            model: "deepseek-v4-flash".to_string(),
            agent_prompt: default_agent_prompt(),
            api_key: None,
        },
    ]
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PersistedAppConfig {
    active_provider_id: String,
    settings: RuntimeSettings,
    providers: Vec<PersistedProviderConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PersistedProviderConfig {
    id: String,
    name: String,
    protocol: String,
    base_url: String,
    model: String,
    agent_prompt: String,
}

impl PersistedProviderConfig {
    fn into_provider(self) -> ProviderConfig {
        ProviderConfig {
            id: self.id,
            name: self.name,
            protocol: self.protocol,
            base_url: self.base_url,
            model: self.model,
            agent_prompt: self.agent_prompt,
            api_key: None,
        }
    }
}

impl From<&ProviderConfig> for PersistedProviderConfig {
    fn from(config: &ProviderConfig) -> Self {
        Self {
            id: config.id.clone(),
            name: config.name.clone(),
            protocol: config.protocol.clone(),
            base_url: config.base_url.clone(),
            model: config.model.clone(),
            agent_prompt: config.agent_prompt.clone(),
        }
    }
}

fn provider_view(config: &ProviderConfig) -> ProviderConfigView {
    ProviderConfigView {
        id: config.id.clone(),
        name: config.name.clone(),
        protocol: config.protocol.clone(),
        base_url: config.base_url.clone(),
        model: config.model.clone(),
        agent_prompt: config.agent_prompt.clone(),
        api_key_configured: config
            .api_key
            .as_ref()
            .is_some_and(|key| !key.trim().is_empty()),
    }
}

fn config_file_path(app: &AppHandle) -> anyhow::Result<PathBuf> {
    Ok(stable_config_dir(app)?.join("config.json"))
}

fn legacy_config_file_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|path| path.join("config.json"))
}

fn secrets_file_path(app: &AppHandle) -> anyhow::Result<PathBuf> {
    Ok(stable_config_dir(app)?.join("secrets.json"))
}

fn legacy_secrets_file_path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|path| path.join("secrets.json"))
}

fn stable_config_dir(app: &AppHandle) -> anyhow::Result<PathBuf> {
    if let Some(appdata) = env::var_os("APPDATA") {
        return Ok(PathBuf::from(appdata).join("AI Translate"));
    }

    Ok(app.path().app_config_dir()?)
}

fn provider_key_account(provider_id: &str) -> String {
    format!("provider:{provider_id}:api-key")
}

fn store_provider_key(app: &AppHandle, provider_id: &str, api_key: &str) -> anyhow::Result<()> {
    let _ = keyring::Entry::new(KEYRING_SERVICE, &provider_key_account(provider_id))
        .and_then(|entry| entry.set_password(api_key));
    store_encrypted_provider_key(app, provider_id, api_key)?;
    Ok(())
}

fn load_provider_key(app: &AppHandle, provider_id: &str) -> Option<String> {
    let keyring_key = keyring::Entry::new(KEYRING_SERVICE, &provider_key_account(provider_id))
        .ok()
        .and_then(|entry| entry.get_password().ok())
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty());

    keyring_key.or_else(|| load_encrypted_provider_key(app, provider_id))
}

fn hydrate_provider_keys(app: &AppHandle, providers: &mut [ProviderConfig]) {
    for provider in providers {
        if provider.protocol != "mymemory" {
            provider.api_key = load_provider_key(app, &provider.id);
        }
    }
}

fn persist_provider_keys(app: &AppHandle, providers: &[ProviderConfig]) -> anyhow::Result<()> {
    for provider in providers {
        if provider.protocol == "mymemory" {
            continue;
        }

        if let Some(api_key) = provider
            .api_key
            .as_ref()
            .map(|key| key.trim())
            .filter(|key| !key.is_empty())
        {
            store_encrypted_provider_key(app, &provider.id, api_key)?;
        }
    }

    Ok(())
}

#[derive(Default, Serialize, Deserialize)]
struct PersistedSecrets {
    api_keys: HashMap<String, String>,
}

fn store_encrypted_provider_key(
    app: &AppHandle,
    provider_id: &str,
    api_key: &str,
) -> anyhow::Result<()> {
    let path = secrets_file_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut secrets = read_persisted_secrets(&path);
    secrets
        .api_keys
        .insert(provider_id.to_string(), protect_secret(api_key)?);
    fs::write(path, serde_json::to_string_pretty(&secrets)?)?;
    Ok(())
}

fn load_encrypted_provider_key(app: &AppHandle, provider_id: &str) -> Option<String> {
    let path = secrets_file_path(app).ok()?;
    load_encrypted_provider_key_from_path(&path, provider_id).or_else(|| {
        legacy_secrets_file_path(app)
            .as_ref()
            .and_then(|path| load_encrypted_provider_key_from_path(path, provider_id))
    })
}

fn load_encrypted_provider_key_from_path(path: &PathBuf, provider_id: &str) -> Option<String> {
    read_persisted_secrets(path)
        .api_keys
        .get(provider_id)
        .and_then(|encrypted| unprotect_secret(encrypted).ok())
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
}

fn read_persisted_secrets(path: &PathBuf) -> PersistedSecrets {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

#[cfg(target_os = "windows")]
fn protect_secret(secret: &str) -> anyhow::Result<String> {
    let bytes = secret.as_bytes();
    let input = CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: null_mut(),
    };

    let ok = unsafe {
        CryptProtectData(
            &input,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };

    if ok == 0 {
        anyhow::bail!("failed to encrypt api key with Windows DPAPI");
    }

    let encrypted = unsafe { slice::from_raw_parts(output.pbData, output.cbData as usize) };
    let encoded = hex_encode(encrypted);
    unsafe {
        LocalFree(output.pbData as _);
    }
    Ok(encoded)
}

#[cfg(target_os = "windows")]
fn unprotect_secret(encrypted: &str) -> anyhow::Result<String> {
    let mut bytes = hex_decode(encrypted)?;
    let input = CRYPT_INTEGER_BLOB {
        cbData: bytes.len() as u32,
        pbData: bytes.as_mut_ptr(),
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: null_mut(),
    };

    let ok = unsafe {
        CryptUnprotectData(
            &input,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };

    if ok == 0 {
        anyhow::bail!("failed to decrypt api key with Windows DPAPI");
    }

    let decrypted = unsafe { slice::from_raw_parts(output.pbData, output.cbData as usize) };
    let secret = String::from_utf8(decrypted.to_vec())?;
    unsafe {
        LocalFree(output.pbData as _);
    }
    Ok(secret)
}

#[cfg(not(target_os = "windows"))]
fn protect_secret(_secret: &str) -> anyhow::Result<String> {
    anyhow::bail!("encrypted fallback secret storage is only implemented on Windows")
}

#[cfg(not(target_os = "windows"))]
fn unprotect_secret(_encrypted: &str) -> anyhow::Result<String> {
    anyhow::bail!("encrypted fallback secret storage is only implemented on Windows")
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn hex_decode(value: &str) -> anyhow::Result<Vec<u8>> {
    let value = value.trim();
    if !value.len().is_multiple_of(2) {
        anyhow::bail!("invalid encrypted api key");
    }

    let mut output = Vec::with_capacity(value.len() / 2);
    for index in (0..value.len()).step_by(2) {
        output.push(u8::from_str_radix(&value[index..index + 2], 16)?);
    }
    Ok(output)
}

fn normalize_provider_config(mut config: ProviderConfig) -> anyhow::Result<ProviderConfig> {
    config.id = config.id.trim().to_string();
    config.name = config.name.trim().to_string();
    config.protocol = config.protocol.trim().to_string();
    config.base_url = config.base_url.trim().trim_end_matches('/').to_string();
    config.model = config.model.trim().to_string();
    config.agent_prompt = config.agent_prompt.trim().to_string();
    config.api_key = config
        .api_key
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty());

    if config.id.is_empty() {
        anyhow::bail!("provider id is required");
    }
    if config.name.is_empty() {
        anyhow::bail!("provider name is required");
    }
    if !matches!(
        config.protocol.as_str(),
        "mymemory" | "openai" | "anthropic"
    ) {
        anyhow::bail!("unsupported provider protocol");
    }
    if config.protocol != "mymemory" {
        if config.base_url.is_empty() {
            anyhow::bail!("base url is required");
        }
        if config.model.is_empty() {
            anyhow::bail!("model is required");
        }
    }
    if config.agent_prompt.is_empty() {
        config.agent_prompt = default_agent_prompt();
    }

    Ok(config)
}

pub fn default_agent_prompt() -> String {
    "You are a professional translation engine. Translate the user's text into the requested target language. Preserve meaning, tone, formatting, numbers, code, URLs, and proper nouns. Do not add explanations, alternatives, quotes, markdown fences, or commentary. Return only the translated text.".to_string()
}

fn default_selection_shortcut() -> String {
    "Alt+D".to_string()
}

fn default_main_shortcut() -> String {
    "Ctrl+D".to_string()
}

fn default_theme() -> String {
    "dark".to_string()
}

fn migrate_runtime_settings(mut settings: RuntimeSettings) -> RuntimeSettings {
    if settings.shortcut.trim().is_empty() || settings.shortcut.trim().eq_ignore_ascii_case("Alt+Q")
    {
        settings.shortcut = default_selection_shortcut();
    }
    if settings.main_shortcut.trim().is_empty() {
        settings.main_shortcut = default_main_shortcut();
    }
    if !matches!(settings.theme.as_str(), "dark" | "compact" | "light") {
        settings.theme = default_theme();
    }
    settings
}
