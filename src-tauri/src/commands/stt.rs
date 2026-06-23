use crate::settings::{self, SttBackend};
use log::warn;
use tauri::AppHandle;

fn validate_stt_provider_exists(
    settings: &settings::AppSettings,
    provider_id: &str,
) -> Result<(), String> {
    if !settings
        .stt_providers
        .iter()
        .any(|provider| provider.id == provider_id)
    {
        return Err(format!("STT provider '{}' not found", provider_id));
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_stt_backend_setting(app: AppHandle, backend: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    settings.stt_backend = match backend.as_str() {
        "local" => SttBackend::Local,
        "open_router" | "openrouter" => SttBackend::OpenRouter,
        other => {
            warn!("Invalid STT backend '{}', defaulting to local", other);
            SttBackend::Local
        }
    };
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_stt_provider(app: AppHandle, provider_id: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    validate_stt_provider_exists(&settings, &provider_id)?;
    settings.stt_provider_id = provider_id;
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_stt_api_key_setting(
    app: AppHandle,
    provider_id: String,
    api_key: String,
) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    validate_stt_provider_exists(&settings, &provider_id)?;
    settings.stt_api_keys.insert(provider_id, api_key);
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_stt_model_setting(
    app: AppHandle,
    provider_id: String,
    model: String,
) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    validate_stt_provider_exists(&settings, &provider_id)?;
    settings.stt_models.insert(provider_id, model);
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_stt_models(app: AppHandle, provider_id: String) -> Result<Vec<String>, String> {
    let settings = settings::get_settings(&app);

    let provider = settings
        .stt_provider(&provider_id)
        .ok_or_else(|| format!("STT provider '{}' not found", provider_id))?;

    let api_key = settings
        .stt_api_keys
        .get(&provider_id)
        .cloned()
        .unwrap_or_default();

    crate::stt_client::fetch_transcription_models(provider, api_key).await
}
