use std::net::SocketAddr;

use axum::{
    middleware,
    routing::{get, post},
};
use eyre::{Result, WrapErr};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

mod config;
use config::Config;

mod controllers;

mod error;

mod image;

mod middlewares;

mod routes;

mod state;
use state::AppStateBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let config = envy::from_env::<Config>()?;

    // enable tracing
    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(4)
                .with_targets(true)
                .with_indent_lines(true)
                .with_bracketed_fields(true)
                .with_thread_names(false)
                .with_thread_ids(true),
        )
        .init();

    let mut app_state = AppStateBuilder::new().with_config(config).build();
    app_state
        .try_init_background()
        .await
        .wrap_err("Failed to load background")?;

    let priviliged_router = axum::Router::new()
        .route("/background/update/", post(routes::update_background))
        .route("/background/list/", get(routes::backgrounds_list))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            middlewares::admin_subnet_restricted,
        ));

    let common_router = axum::Router::new().route("/background/", get(routes::get_background));

    let app = axum::Router::new()
        .nest("/admin", priviliged_router)
        .merge(common_router)
        .route_layer(TraceLayer::new_for_http())
        .with_state(app_state.clone());

    let addr = app_state.config.listen_addr().parse::<SocketAddr>()?;
    let lst = TcpListener::bind(addr).await?;

    info!("Listening on {}", addr);

    axum::serve(lst, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}
