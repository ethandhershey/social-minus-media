mod config;

use anyhow::{Context, Result};
use api::{
    router::create_router,
    state::{AppServices, AppState},
};
use clap::{Parser, Subcommand};
use infra::{
    auth::jwt_validator::JwtTokenValidator,
    entitlment::ConfigEntitlementService,
    mail::ResendMailClient,
    postgres::{
        event_repo::PgEventRepository, product_repo::PgProductRepository,
        rsvp_repo::PgRsvpRepository, user_interests_repo::PgUserInterestsRepository,
        user_repo::PgUserRepository,
    },
    stripe::StripeClient,
};
use secrecy::ExposeSecret;
use std::path::Path;
use tokio::net::TcpListener;

use crate::config::AppConfig;

pub const BUILD_ID: &str = concat!(env!("CARGO_PKG_VERSION"), "-", env!("GIT_HASH"));

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(Command::Init) = args.command {
        write_default_config()?;
        return Ok(());
    }

    let config = AppConfig::new(&args.configs).with_context(|| {
        let files = args
            .configs
            .iter()
            .map(|name| format!("  config/{name}.toml"))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "failed to load config. Expected files:\n\n{files}\n\nRun `{} init` to generate a default config.",
            env!("CARGO_PKG_NAME")
        )
    })?;

    if let Some(Command::PrintConfig) = args.command {
        println!("{:#?}", config);
        return Ok(());
    }

    setup_tracing(&config.server.logs_dir)?;

    tracing::info!("starting {} on {}", BUILD_ID, config.server.address);

    // Bind early so we fail fast if the address is unavailable
    let listener = TcpListener::bind(config.server.address).await?;

    // ── Database ─────────────────────────────────────────────────────────────
    let pool = infra::postgres::create(
        &config.database.url.expose_secret(),
        config.database.max_connections,
    )
    .await?;
    infra::postgres::migrate(&pool).await?;

    // ── Infra ────────────────────────────────────────────────────────────────
    let auth_client = infra::http::AuthClient::new()?;
    let http_client = infra::http::HttpClient::new()?;

    let validator = JwtTokenValidator::new(
        &config.zitadel.issuer,
        &config.zitadel.client_id,
        &config.zitadel.jwks_url,
        auth_client,
    )
    .await
    .context("failed to initialise JWT validator")?;

    #[cfg(not(feature = "fake-ai"))]
    let ai = infra::llm::SimpleLlmClient::new(
        http_client.clone(),
        config.llm.providers,
        config.llm.models,
        config.llm.embed_models,
    );
    #[cfg(feature = "fake-ai")]
    let ai = domain::test_utils::fake_ai_service::FakeAiService::new();

    let user_repo = PgUserRepository::new(pool.clone());
    let product_repo = PgProductRepository::new(pool.clone());
    let event_repo = PgEventRepository::new(pool.clone());
    let rsvp_repo = PgRsvpRepository::new(pool.clone());
    let user_interests_repo = PgUserInterestsRepository::new(
        pool.clone(),
        config.llm.interests_summary_model,
        config.llm.interests_embed_model,
    );

    let stripe = StripeClient::new(
        http_client.clone(),
        config.stripe.secret_key,
        config.stripe.webhook_secret,
        config.stripe.essential_price_id,
        config.stripe.pro_price_id,
        config.stripe.success_url,
        config.stripe.cancel_url,
        config.stripe.portal_return_url,
    );

    let mail = ResendMailClient::new(
        http_client,
        config.resend.api_key,
        config.resend.sender_email,
        config.resend.sender_name,
    );

    let entitlment = ConfigEntitlementService::new(config.tiers.into());

    let public_config = api::state::PublicConfig {
        version: BUILD_ID,
        auth_client_id: config.zitadel.client_id,
        auth_issuer: config.zitadel.issuer,
    };

    // ── State ────────────────────────────────────────────────────────────────
    #[derive(Clone)]
    struct Services;

    impl AppServices for Services {
        type Auth = JwtTokenValidator;
        type UserRepo = PgUserRepository;
        type ProductRepo = PgProductRepository;
        type EventRepo = PgEventRepository;
        type RsvpRepo = PgRsvpRepository;
        type UserInterestsRepo = PgUserInterestsRepository;
        #[cfg(not(feature = "fake-ai"))]
        type Llm = infra::llm::SimpleLlmClient;
        #[cfg(feature = "fake-ai")]
        type Llm = domain::test_utils::fake_ai_service::FakeAiService;
        type Billing = StripeClient;
        type Mail = ResendMailClient;
        type Entitlement = ConfigEntitlementService;
    }

    let state = AppState::<Services>::new(
        public_config,
        validator,
        user_repo,
        product_repo,
        event_repo,
        rsvp_repo,
        user_interests_repo,
        ai,
        stripe,
        mail,
        entitlment,
    );

    // ── Router ───────────────────────────────────────────────────────────────
    let app = create_router(
        state,
        config.server.frontend_dir,
        config.server.allowed_origins,
        config.server.max_upload_size,
    );

    // ── Serve ────────────────────────────────────────────────────────────────
    tracing::info!("listening on http://{}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[derive(Parser, Debug)]
pub struct Args {
    /// Configuration file names (without extensions) in order of least to most priority
    #[arg(
        long,
        short = 'c',
        env = "APP_CONFIGS",
        value_delimiter = ',',
        default_value = "default,local"
    )]
    configs: Vec<String>,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Generate a default and example config file
    Init,
    /// Print the resolved config and exit
    PrintConfig,
}

pub fn write_default_config() -> Result<()> {
    const DEFAULT_PATH: &str = "config/default.toml";
    const EXAMPLE_PATH: &str = "config/example.toml";

    std::fs::create_dir_all("config")?;
    std::fs::write(DEFAULT_PATH, include_str!("../../config/default.toml"))?;
    std::fs::write(EXAMPLE_PATH, include_str!("../../config/example.toml"))?;

    println!(
        "Created \"{DEFAULT_PATH}\" and \"{EXAMPLE_PATH}\".\n\
         Copy example.toml to local.toml, fill in your secrets, then start the app.\n\
         Never commit local.toml."
    );

    Ok(())
}

fn setup_tracing(logs_dir: &Path) -> Result<()> {
    use tracing_subscriber::{EnvFilter, Registry, fmt, fmt::format::FmtSpan, prelude::*};

    const LOG_NAME_PREFIX: &str = "app-logs";

    let stdout_layer = fmt::layer()
        .with_target(false)
        .with_file(false)
        .with_line_number(false)
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(std::io::stdout)
        .with_ansi(true);

    std::fs::create_dir_all(logs_dir)?;
    let file_appender = tracing_appender::rolling::daily(logs_dir, LOG_NAME_PREFIX);

    let file_layer = fmt::layer()
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_writer(file_appender)
        .with_ansi(false);

    let stdout_filter = EnvFilter::try_from_env("RUST_LOG_TERM").unwrap_or_else(|_| {
        format!(
            "{}=debug,infra=debug,api=debug,tower_http=error,axum=error",
            env!("CARGO_CRATE_NAME")
        )
        .into()
    });

    let file_filter = EnvFilter::try_from_env("RUST_LOG_FILE").unwrap_or_else(|_| {
        format!(
            "{}=debug,infra=debug,api=debug,tower_http=error,axum=error",
            env!("CARGO_CRATE_NAME")
        )
        .into()
    });

    let registry = Registry::default()
        .with(stdout_filter.and_then(stdout_layer))
        .with(file_filter.and_then(file_layer));

    #[cfg(feature = "tokio-console")]
    let registry = registry.with(console_subscriber::spawn());

    registry.init();
    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
