use crate::settings::{AppSettings, PostProcessProvider, SecretMap};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;

pub const OPENROUTER_PROVIDER_ID: &str = "openrouter";
pub const OPENROUTER_DEFAULT_STT_MODEL_ID: &str = "openai/whisper-large-v3-turbo";

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum SttBackend {
    Local,
    OpenRouter,
}

impl Default for SttBackend {
    fn default() -> Self {
        SttBackend::Local
    }
}

pub fn default_stt_provider_id() -> String {
    OPENROUTER_PROVIDER_ID.to_string()
}

pub fn default_stt_providers() -> Vec<PostProcessProvider> {
    vec![PostProcessProvider {
        id: OPENROUTER_PROVIDER_ID.to_string(),
        label: "OpenRouter".to_string(),
        base_url: "https://openrouter.ai/api/v1".to_string(),
        allow_base_url_edit: false,
        models_endpoint: Some("/models?output_modalities=transcription".to_string()),
        supports_structured_output: false,
    }]
}

pub fn default_stt_api_keys() -> SecretMap {
    let mut map = HashMap::new();
    for provider in default_stt_providers() {
        map.insert(provider.id, String::new());
    }
    SecretMap::new(map)
}

pub fn default_stt_models() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for provider in default_stt_providers() {
        map.insert(provider.id, OPENROUTER_DEFAULT_STT_MODEL_ID.to_string());
    }
    map
}

pub fn ensure_stt_defaults(settings: &mut AppSettings) -> bool {
    let mut changed = false;
    for provider in default_stt_providers() {
        match settings
            .stt_providers
            .iter_mut()
            .find(|existing| existing.id == provider.id)
        {
            Some(existing) => {
                if existing.base_url.is_empty() {
                    existing.base_url = provider.base_url.clone();
                    changed = true;
                }
                if existing.models_endpoint != provider.models_endpoint {
                    existing.models_endpoint = provider.models_endpoint.clone();
                    changed = true;
                }
            }
            None => {
                settings.stt_providers.push(provider.clone());
                changed = true;
            }
        }

        if !settings.stt_api_keys.contains_key(&provider.id) {
            settings
                .stt_api_keys
                .insert(provider.id.clone(), String::new());
            changed = true;
        }

        match settings.stt_models.get_mut(&provider.id) {
            Some(existing) => {
                if existing.is_empty() {
                    *existing = OPENROUTER_DEFAULT_STT_MODEL_ID.to_string();
                    changed = true;
                }
            }
            None => {
                settings.stt_models.insert(
                    provider.id.clone(),
                    OPENROUTER_DEFAULT_STT_MODEL_ID.to_string(),
                );
                changed = true;
            }
        }
    }

    changed
}
