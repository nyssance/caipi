use tauri::{Runtime, plugin::TauriPlugin};
use tauri_plugin_log::{
    Builder, Target, TargetKind, TimezoneStrategy,
    fern::colors::{Color, ColoredLevelConfig},
};

pub fn log_init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new()
        .target(Target::new(TargetKind::Webview))
        .timezone_strategy(TimezoneStrategy::UseLocal)
        .with_colors(ColoredLevelConfig::new().debug(Color::Magenta))
        .level_for("tao", log::LevelFilter::Info) // 必须在timezone_strategy后才有效
        .build()
}
