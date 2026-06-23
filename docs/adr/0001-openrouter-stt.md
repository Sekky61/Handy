# ADR 0001: OpenRouter Speech-to-Text Backend

## Status

Implemented; pending live validation against OpenRouter.

## What changed

Handy now has an optional remote speech-to-text backend using OpenRouter. Local/offline transcription remains the default.

The new flow is:

```text
recorded audio -> OpenRouter STT -> transcript -> optional existing post-processing -> paste
```

A new STT settings section lets users select:

- STT backend: Local or OpenRouter
- STT provider: OpenRouter
- API key
- transcription model

## Why

This keeps Handy usable without local model downloads for users who prefer remote transcription, while preserving the existing local-first behavior by default.

The implementation intentionally keeps remote STT separate from the existing post-processing provider settings so transcription and cleanup can be configured independently.

## Technical notes

Backend additions:

- `src-tauri/src/stt_settings.rs`
  - owns STT-specific defaults and migration helpers
  - defines `SttBackend`
  - keeps most STT settings logic out of `settings.rs` to reduce upstream merge conflicts
- `src-tauri/src/stt_client.rs`
  - converts 16 kHz mono `f32` samples to 16-bit WAV bytes
  - base64-encodes WAV data
  - calls OpenRouter `/audio/transcriptions`
  - fetches transcription-capable models
- `src-tauri/src/commands/stt.rs`
  - provides Tauri commands for STT backend/provider/API key/model/model-fetch settings

Minimal integration points:

- `settings.rs` adds persisted STT fields to `AppSettings`
- `actions.rs` switches only the transcription call site based on `stt_backend`
- local model preload is skipped when OpenRouter STT is selected
- onboarding treats configured remote STT as sufficient to avoid requiring a local model

Frontend additions:

- `src/components/settings/stt/SttSettings.tsx`
- STT sidebar tab
- Zustand store and `useSettings` helpers for STT settings
- English i18n strings

## References

- [OpenRouter Speech-to-Text documentation](https://openrouter.ai/docs/guides/overview/multimodal/stt)
