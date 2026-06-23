use crate::settings::PostProcessProvider;
use base64::{engine::general_purpose, Engine as _};
use hound::{WavSpec, WavWriter};
use log::debug;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug, Serialize)]
struct InputAudio {
    data: String,
    format: String,
}

#[derive(Debug, Serialize)]
struct TranscriptionRequest {
    model: String,
    input_audio: InputAudio,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TranscriptionResponse {
    text: Option<String>,
}

/// Convert 16 kHz mono f32 samples to a 16-bit PCM WAV byte buffer.
fn samples_to_wav_bytes(samples: &[f32]) -> Result<Vec<u8>, String> {
    let spec = WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = WavWriter::new(&mut cursor, spec)
            .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

        for sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let sample_i16 = (clamped * i16::MAX as f32) as i16;
            writer
                .write_sample(sample_i16)
                .map_err(|e| format!("Failed to write WAV sample: {}", e))?;
        }

        writer
            .finalize()
            .map_err(|e| format!("Failed to finalize WAV data: {}", e))?;
    }

    Ok(cursor.into_inner())
}

fn build_headers(api_key: &str) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://github.com/cjpais/Handy"),
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Handy/1.0 (+https://github.com/cjpais/Handy)"),
    );
    headers.insert("X-Title", HeaderValue::from_static("Handy"));

    if !api_key.is_empty() {
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| format!("Invalid authorization header value: {}", e))?,
        );
    }

    Ok(headers)
}

fn create_client(api_key: &str) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .default_headers(build_headers(api_key)?)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

pub async fn transcribe_openrouter(
    provider: &PostProcessProvider,
    api_key: String,
    model: &str,
    samples: &[f32],
    language: Option<String>,
) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("OpenRouter API key is required for remote transcription".to_string());
    }

    let model = model.trim();
    if model.is_empty() {
        return Err("OpenRouter STT model is not configured".to_string());
    }

    let wav_bytes = samples_to_wav_bytes(samples)?;
    let encoded_audio = general_purpose::STANDARD.encode(wav_bytes);

    let request_body = TranscriptionRequest {
        model: model.to_string(),
        input_audio: InputAudio {
            data: encoded_audio,
            format: "wav".to_string(),
        },
        language: language.and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() || trimmed == "auto" {
                None
            } else {
                Some(trimmed)
            }
        }),
    };

    let base_url = provider.base_url.trim_end_matches('/');
    let url = format!("{}/audio/transcriptions", base_url);
    debug!("Sending OpenRouter STT request to: {}", url);

    let client = create_client(&api_key)?;
    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        return Err(format!(
            "STT request failed with status {}: {}",
            status, error_text
        ));
    }

    let parsed: TranscriptionResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse STT response: {}", e))?;

    Ok(parsed.text.unwrap_or_default())
}

pub async fn fetch_transcription_models(
    provider: &PostProcessProvider,
    api_key: String,
) -> Result<Vec<String>, String> {
    let base_url = provider.base_url.trim_end_matches('/');
    let endpoint = provider
        .models_endpoint
        .as_deref()
        .unwrap_or("/models?output_modalities=transcription");
    let url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        endpoint.to_string()
    } else {
        format!("{}{}", base_url, endpoint)
    };

    debug!("Fetching STT models from: {}", url);

    let client = create_client(&api_key)?;
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch STT models: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!(
            "STT model list request failed ({}): {}",
            status, error_text
        ));
    }

    let parsed: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse STT models response: {}", e))?;

    let mut models = Vec::new();
    if let Some(data) = parsed.get("data").and_then(|data| data.as_array()) {
        for entry in data {
            if let Some(id) = entry.get("id").and_then(|id| id.as_str()) {
                models.push(id.to_string());
            }
        }
    } else if let Some(array) = parsed.as_array() {
        for entry in array {
            if let Some(model) = entry.as_str() {
                models.push(model.to_string());
            } else if let Some(id) = entry.get("id").and_then(|id| id.as_str()) {
                models.push(id.to_string());
            }
        }
    }

    Ok(models)
}
