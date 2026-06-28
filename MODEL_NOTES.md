# Poznámky k modelům

Moje rychlé poznámky k STT a textovým/post-processing modelům v Handy.

## Aktuální preference

- STT: OpenRouter / `openai/gpt-4o-mini-transcribe`
- Předchozí baseline: `openai/whisper-large-v3-turbo`
- Post-processing: OpenRouter / `@preset/superspeed`

## STT

- `openai/whisper-large-v3-turbo` — předchozí baseline. Rychlý, ale slabší na českou diktaci s programátorskými anglicismy.
- `google/chirp-3` — lepší v anglicismech než baseline, ale moc pomalý; není useful pro běžné používání. Taky trochu dražší.
- `openai/gpt-4o-mini-transcribe` — kvalitou zhruba jako `google/chirp-3`, ale levnější. Asi poloviční rychlost než baseline.
- `openai/whisper-1` — ok, ale trochu pomalejší než baseline.
- `openai/gpt-4o-transcribe` — podobné jako `openai/gpt-4o-mini-transcribe`.

## Textové / post-processing

- `@preset/superspeed` — current model. Rychlý; u programátorské češtiny hlídat, jestli neopravuje technické výrazy špatným směrem.
