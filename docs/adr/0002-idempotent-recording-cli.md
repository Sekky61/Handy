# ADR 0002: Idempotent Recording CLI Flags

## Status

Implemented locally; pending validation.

## Context

Handy's remote-control CLI originally exposed toggle-oriented recording flags:

```text
handy --toggle-transcription
handy --toggle-post-process
```

These work well for simple launchers, but are fragile for integrations that can bind key press and key release separately. If a press/release event is missed or repeated, a toggle can leave Handy in the opposite state from the external key state.

This is especially visible in push-to-talk window-manager bindings where press should start recording and release should stop recording.

## Decision

Add idempotent recording lifecycle flags:

```text
handy --start-recording
handy --stop-recording
handy --toggle-recording
handy --start-recording --post-process
handy --toggle-recording --post-process
handy --cancel
```

The new names intentionally use `recording` instead of `stt`. In Handy, STT now also refers to the selected speech-to-text backend/provider, so `--stt-on` / `--stt-off` would be ambiguous.

Legacy flags remain supported as aliases:

```text
handy --toggle-transcription  # equivalent to --toggle-recording
handy --toggle-post-process   # equivalent to --toggle-recording --post-process
```

## Semantics

The lifecycle flags are idempotent where possible:

| Current state | `--start-recording` | `--stop-recording`     | `--toggle-recording`   |
| ------------- | ------------------- | ---------------------- | ---------------------- |
| idle          | start recording     | no-op                  | start recording        |
| recording     | no-op               | stop current recording | stop current recording |
| processing    | no-op               | no-op                  | no-op                  |

`--post-process` only affects start/toggle commands that begin a new recording. Stop does not need a post-processing flag because the coordinator remembers which recording action started the current session.

If multiple remote-control flags are supplied in one invocation, the single-instance handler applies the first matching action in this priority order:

1. `--cancel`
2. `--stop-recording`
3. `--start-recording`
4. `--toggle-recording`
5. `--toggle-post-process` legacy alias
6. `--toggle-transcription` legacy alias
7. show the main window

## Technical notes

- `CliArgs` now accepts the new flags so direct CLI parsing and `--help` know about them.
- The single-instance handler maps the new flags to coordinator methods instead of simulating shortcut key presses.
- `TranscriptionCoordinator` now has explicit commands for:
  - `StartRecording { post_process }`
  - `StopRecording`
  - `ToggleRecording { post_process }`
- Keyboard shortcuts and signal handlers continue to use the existing input path.
- Existing `--toggle-transcription` and `--toggle-post-process` behavior is preserved through the new toggle command.

## Example push-to-talk binding

```nix
{
  description = "Handy push to talk with post-processing";
  bind = {
    mods = ["SUPER"];
    key = "Z";
  };
  command = { exec = "handy --start-recording --post-process"; };
}
{
  description = "Handy push to talk with post-processing release";
  bind = {
    mods = ["SUPER"];
    key = "Z";
  };
  command = {
    exec = "handy --stop-recording";
    flags = ["release"];
  };
}
```
