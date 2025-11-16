/// Initialize the tracing subscriber with configurable output format
///
/// Uses RUST_LOG environment variable for filtering:
/// - `RUST_LOG=debug` - All debug logs
/// - `RUST_LOG=run_sous_bpm_api=debug,tower_http=info` - Specific module levels
/// - `RUST_LOG=error` - Only errors
pub fn init_tracing() {
    // Determine if we should use pretty or compact format based on environment
    let use_pretty = std::env
        ::var("LOG_FORMAT")
        .map(|f| f.to_lowercase() == "pretty")
        .unwrap_or(true); // Default to pretty for development

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default filter configuration
        // - Your app at INFO level
        // - Database queries at DEBUG
        // - Tower HTTP middleware at INFO
        // - Axum rejections at TRACE (helps debug extractor issues)
        "run_sous_bpm_api=info,run_sous_bpm_core=info,run_sous_bpm_integrations=info,sqlx=debug,tower_http=info,axum::rejection=trace".into()
    });

    let subscriber = tracing_subscriber
        ::fmt()
        .with_env_filter(env_filter)
        .with_target(true) // Show which module logged
        .with_line_number(true) // Show line numbers
        .with_thread_ids(false) // Don't show thread IDs (noisy)
        .with_file(false); // Don't show full file paths

    if use_pretty {
        subscriber.pretty().init();
    } else {
        subscriber.compact().init();
    }
}
