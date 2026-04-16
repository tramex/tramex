//! AI settings shared state
//!
//! Stores API key and provider selection, shared between Settings UI and panels.

#[cfg(feature = "ai")]
use tramex_tools::ai::AIProvider;

#[cfg(feature = "ai")]
/// Shared AI configuration
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct AISettings {
    /// API key (not serialized for security — must be re-entered or loaded from env)
    #[serde(skip)]
    pub api_key: String,

    /// Selected AI provider
    pub provider: AIProvider,

    /// Selected model ID (e.g. "mistral-medium-latest")
    pub model: String,

    /// Whether to attempt loading the API key from environment variable
    pub use_env_var: bool,

    /// Environment variable name for the API key
    pub env_var_name: String,

    /// Whether env var was already checked this session
    #[serde(skip)]
    env_checked: bool,
}

#[cfg(feature = "ai")]
impl Default for AISettings {
    fn default() -> Self {
        let provider = AIProvider::default();
        let model = provider.default_model().to_string();
        Self {
            api_key: String::new(),
            provider,
            model,
            use_env_var: true,
            env_var_name: "TRAMEX_AI_API_KEY".to_string(),
            env_checked: false,
        }
    }
}

#[cfg(feature = "ai")]
impl AISettings {
    /// Try to load the API key from the environment variable (native only).
    /// Only attempts once per session.
    pub fn try_load_env_key(&mut self) {
        if self.env_checked || !self.use_env_var {
            return;
        }
        self.env_checked = true;

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(key) = std::env::var(&self.env_var_name) {
                if !key.is_empty() {
                    log::info!("Loaded AI API key from env var {}", self.env_var_name);
                    self.api_key = key;
                }
            }
        }
    }

    /// Check if a valid API key is available
    pub fn has_api_key(&self) -> bool {
        !self.api_key.is_empty()
    }

    /// Show the settings UI panel
    pub fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.heading("AI Explain");
        ui.add_space(4.0);

        // Provider selector
        let previous_provider = self.provider.clone();
        ui.horizontal(|ui| {
            ui.label("Provider:");
            eframe::egui::ComboBox::from_id_salt("ai_provider")
                .selected_text(self.provider.to_string())
                .show_ui(ui, |ui| {
                    for provider in AIProvider::all() {
                        ui.selectable_value(&mut self.provider, provider.clone(), provider.to_string());
                    }
                });
        });

        // Reset model to provider default when switching providers
        if self.provider != previous_provider {
            self.model = self.provider.default_model().to_string();
        }

        ui.add_space(4.0);

        // Model selector
        let models = self.provider.available_models();
        let current_display = models
            .iter()
            .find(|(_, id)| *id == self.model)
            .map(|(name, _)| *name)
            .unwrap_or("Unknown");
        ui.horizontal(|ui| {
            ui.label("Model:");
            eframe::egui::ComboBox::from_id_salt("ai_model")
                .selected_text(current_display)
                .show_ui(ui, |ui| {
                    for (display_name, model_id) in models {
                        ui.selectable_value(&mut self.model, model_id.to_string(), *display_name);
                    }
                });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // Env var option
        ui.checkbox(&mut self.use_env_var, "Load key from environment variable");
        if self.use_env_var {
            ui.horizontal(|ui| {
                ui.label("Env var:");
                ui.text_edit_singleline(&mut self.env_var_name);
            });
            ui.label("Set this in your system environment variables, then restart the app.");
        }

        ui.add_space(4.0);

        // API key input (always available as fallback / override)
        ui.label("API Key:");
        let key_edit = eframe::egui::TextEdit::singleline(&mut self.api_key)
            .password(true)
            .hint_text("Enter API key…")
            .desired_width(300.0);
        ui.add(key_edit);

        if self.api_key.is_empty() {
            ui.colored_label(eframe::egui::Color32::YELLOW, "No API key set");
        } else {
            ui.colored_label(eframe::egui::Color32::GREEN, "API key set");
        }
    }
}
