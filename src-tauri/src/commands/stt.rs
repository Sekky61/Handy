use crate::settings::{self, AppSettings, SttBackend};
use crate::stt_settings::SttPreset;
use log::warn;
use tauri::AppHandle;

fn capture_stt_preset(settings: &AppSettings, id: String, name: String) -> SttPreset {
    let model = match settings.stt_backend {
        SttBackend::Local => settings.selected_model.clone(),
        SttBackend::OpenRouter => settings
            .stt_models
            .get(&settings.stt_provider_id)
            .cloned()
            .unwrap_or_default(),
    };

    SttPreset {
        id,
        name,
        backend: settings.stt_backend,
        model,
        stt_provider_id: settings.stt_provider_id.clone(),
        post_process_enabled: settings.post_process_enabled,
        post_process_provider_id: settings.post_process_provider_id.clone(),
        post_process_model: settings
            .post_process_models
            .get(&settings.post_process_provider_id)
            .cloned()
            .unwrap_or_default(),
        post_process_prompt_id: settings.post_process_selected_prompt_id.clone(),
        post_process_disable_reasoning: settings.post_process_disable_reasoning,
    }
}

fn validate_preset_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Preset name cannot be empty".to_string());
    }
    if name.chars().count() > 60 {
        return Err("Preset name must be 60 characters or fewer".to_string());
    }
    Ok(name.to_string())
}

fn ensure_unique_preset_name(
    settings: &AppSettings,
    name: &str,
    except_id: Option<&str>,
) -> Result<(), String> {
    if settings.stt_presets.iter().any(|preset| {
        Some(preset.id.as_str()) != except_id && preset.name.eq_ignore_ascii_case(name)
    }) {
        return Err("A preset with that name already exists".to_string());
    }
    Ok(())
}

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

#[tauri::command]
#[specta::specta]
pub fn save_stt_preset(app: AppHandle, name: String) -> Result<SttPreset, String> {
    let name = validate_preset_name(&name)?;
    let mut settings = settings::get_settings(&app);
    ensure_unique_preset_name(&settings, &name, None)?;

    let timestamp = chrono::Utc::now().timestamp_millis();
    let mut suffix = 0;
    let id = loop {
        let candidate = if suffix == 0 {
            format!("stt_preset_{timestamp}")
        } else {
            format!("stt_preset_{timestamp}_{suffix}")
        };
        if settings
            .stt_presets
            .iter()
            .all(|preset| preset.id != candidate)
        {
            break candidate;
        }
        suffix += 1;
    };
    let preset = capture_stt_preset(&settings, id, name);
    settings.stt_presets.push(preset.clone());
    settings::write_settings(&app, settings);
    Ok(preset)
}

#[tauri::command]
#[specta::specta]
pub fn update_stt_preset(app: AppHandle, id: String, name: String) -> Result<SttPreset, String> {
    let name = validate_preset_name(&name)?;
    let mut settings = settings::get_settings(&app);
    ensure_unique_preset_name(&settings, &name, Some(&id))?;

    let preset = capture_stt_preset(&settings, id.clone(), name);
    let Some(existing) = settings
        .stt_presets
        .iter_mut()
        .find(|preset| preset.id == id)
    else {
        return Err(format!("STT preset '{}' not found", id));
    };
    *existing = preset.clone();
    settings::write_settings(&app, settings);
    Ok(preset)
}

#[tauri::command]
#[specta::specta]
pub fn delete_stt_preset(app: AppHandle, id: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    let original_len = settings.stt_presets.len();
    settings.stt_presets.retain(|preset| preset.id != id);
    if settings.stt_presets.len() == original_len {
        return Err(format!("STT preset '{}' not found", id));
    }
    settings::write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn activate_stt_preset(app: AppHandle, id: String) -> Result<(), String> {
    let mut settings = settings::get_settings(&app);
    let preset = settings
        .stt_presets
        .iter()
        .find(|preset| preset.id == id)
        .cloned()
        .ok_or_else(|| format!("STT preset '{}' not found", id))?;

    if preset.backend == SttBackend::OpenRouter {
        validate_stt_provider_exists(&settings, &preset.stt_provider_id)?;
    }
    if !settings
        .post_process_providers
        .iter()
        .any(|provider| provider.id == preset.post_process_provider_id)
    {
        return Err(format!(
            "Post-processing provider '{}' used by this preset is no longer available",
            preset.post_process_provider_id
        ));
    }
    if let Some(prompt_id) = &preset.post_process_prompt_id {
        if !settings
            .post_process_prompts
            .iter()
            .any(|prompt| &prompt.id == prompt_id)
        {
            return Err(
                "The prompt used by this preset no longer exists. Update or remove the preset."
                    .to_string(),
            );
        }
    }

    let post_process_enabled_changed = settings.post_process_enabled != preset.post_process_enabled;
    settings.stt_backend = preset.backend;
    settings.stt_provider_id = preset.stt_provider_id.clone();
    match preset.backend {
        SttBackend::Local => settings.selected_model = preset.model,
        SttBackend::OpenRouter => {
            settings
                .stt_models
                .insert(preset.stt_provider_id, preset.model);
        }
    }
    settings.post_process_enabled = preset.post_process_enabled;
    settings.post_process_provider_id = preset.post_process_provider_id.clone();
    settings
        .post_process_models
        .insert(preset.post_process_provider_id, preset.post_process_model);
    settings.post_process_selected_prompt_id = preset.post_process_prompt_id;
    settings.post_process_disable_reasoning = preset.post_process_disable_reasoning;

    settings::write_settings(&app, settings);
    if post_process_enabled_changed {
        crate::shortcut::reconcile_post_process_enabled(&app, preset.post_process_enabled);
    }
    Ok(())
}
