use color_eyre::eyre::Result;

pub fn setup_logging() -> Result<()> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

    std::env::set_var("RUST_LOG", "debug");

    let log_path = "./editor_log";
    let log_file = std::fs::File::create(log_path)?;

    let subscriber = tracing_subscriber::fmt::layer()
        .pretty()
        .with_file(true)
        .with_writer(log_file)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_filter(tracing_subscriber::EnvFilter::from_default_env());

    tracing_subscriber::registry()
        .with(subscriber)
        .with(ErrorLayer::default())
        .init();

    Ok(())
}

pub fn setup_panic_handler() -> Result<()> {
    //#[cfg(feature = "capture-spantrace")]
    //install_tracing();

    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default().into_hooks();

    eyre_hook.install()?;

    std::panic::set_hook(Box::new(move |pi| {
        if let Ok(mut t) = crate::terminal::Terminal::new() {
            if let Err(r) = t.exit() {
                tracing::error!("Unable to exit Terminal: {:?}", r);
            }
        }

        tracing::error!("{}", panic_hook.panic_report(pi));

        #[cfg(not(debug_assertions))]
        {
            eprintln!("{}", panic_hook.panic_report(pi));
        }

        #[cfg(debug_assertions)]
        {
            better_panic::Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .create_panic_handler()(pi);
        }

        std::process::exit(1);
    }));

    Ok(())
}
