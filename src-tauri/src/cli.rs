use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone, Default)]
#[command(name = "handy", about = "Handy - Speech to Text")]
pub struct CliArgs {
    /// Start with the main window hidden
    #[arg(long)]
    pub start_hidden: bool,

    /// Disable the system tray icon
    #[arg(long)]
    pub no_tray: bool,

    /// Start recording if idle (sent to running instance)
    #[arg(long)]
    pub start_recording: bool,

    /// Wait for a --start-recording request to finish and write its transcription
    /// to stdout. The recording may be stopped by a shortcut or --stop-recording.
    #[arg(
        long,
        requires = "start_recording",
        conflicts_with_all = [
            "stop_recording",
            "toggle_recording",
            "toggle_transcription",
            "toggle_post_process",
            "cancel",
            "start_hidden",
            "no_tray",
            "transcribe_file",
            "model",
            "device_index",
            "list_devices",
            "list_models",
            "repeat"
        ]
    )]
    pub wait: bool,

    /// Internal response endpoint used by the --wait launcher.
    #[arg(long, hide = true, requires = "wait")]
    pub wait_endpoint: Option<String>,

    /// Internal response token used by the --wait launcher.
    #[arg(long, hide = true, requires = "wait_endpoint")]
    pub wait_token: Option<String>,

    /// Stop the active recording if one is running (sent to running instance)
    #[arg(long)]
    pub stop_recording: bool,

    /// Toggle recording on/off (sent to running instance)
    #[arg(long)]
    pub toggle_recording: bool,

    /// Apply AI post-processing to start/toggle recording commands
    #[arg(long)]
    pub post_process: bool,

    /// Toggle transcription on/off (legacy alias for --toggle-recording)
    #[arg(long)]
    pub toggle_transcription: bool,

    /// Toggle transcription with post-processing on/off (legacy alias for --toggle-recording --post-process)
    #[arg(long)]
    pub toggle_post_process: bool,

    /// Cancel the current operation (sent to running instance)
    #[arg(long)]
    pub cancel: bool,

    /// Enable debug mode with verbose logging
    #[arg(long)]
    pub debug: bool,

    /// Transcribe this WAV (16 kHz mono) headlessly and exit. Runs the same
    /// batch transcription path as the app — no mic, no VAD, no download
    /// (the model must already be installed).
    #[arg(short = 'f', long, value_name = "WAV")]
    pub transcribe_file: Option<PathBuf>,

    /// Model id to load for --transcribe-file (default: the selected model).
    #[arg(long)]
    pub model: Option<String>,

    /// Hard-select the compute device for --transcribe-file by its registry
    /// index (see --list-devices). Omit to use the persisted accelerator
    /// setting. transcribe-cpp (whisper-family) models only.
    #[arg(long, value_name = "N")]
    pub device_index: Option<usize>,

    /// List the transcribe-cpp compute devices (with indices) and exit.
    #[arg(long)]
    pub list_devices: bool,

    /// List the available models (with ids) and exit. Pass an id to --model.
    /// Honors --json for machine-readable output.
    #[arg(long)]
    pub list_models: bool,

    /// Repeat the transcription N times (best_ms reports the fastest run).
    #[arg(long, value_name = "N")]
    pub repeat: Option<usize>,

    /// Emit --transcribe-file results as JSON.
    #[arg(long)]
    pub json: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wait_requires_start_recording() {
        assert!(CliArgs::try_parse_from(["handy", "--wait"]).is_err());
        assert!(CliArgs::try_parse_from(["handy", "--start-recording", "--wait"]).is_ok());
    }

    #[test]
    fn wait_accepts_json_and_post_processing() {
        assert!(CliArgs::try_parse_from([
            "handy",
            "--start-recording",
            "--wait",
            "--json",
            "--post-process"
        ])
        .is_ok());
    }

    #[test]
    fn wait_rejects_other_recording_commands() {
        assert!(CliArgs::try_parse_from([
            "handy",
            "--start-recording",
            "--stop-recording",
            "--wait"
        ])
        .is_err());
        assert!(CliArgs::try_parse_from([
            "handy",
            "--start-recording",
            "--wait",
            "--start-hidden"
        ])
        .is_err());
    }
}
