use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::{debug, info, warn};

use crate::state::AppState;

#[tracing::instrument(skip(app_state, addr, req, next), fields(ip = %addr.ip(), port = addr.port()))]
pub async fn admin_subnet_restricted(
    State(app_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let allowed_configuration_from = app_state.config.admin_subnet();

    debug!(
        %allowed_configuration_from,
        "Checking if admin request is allowed"
    );

    if !allowed_configuration_from.contains(&addr.ip()) {
        warn!(%addr, "Forbidden request");

        return ((StatusCode::FORBIDDEN, "Forbidden")).into_response();
    }

    info!(%addr, "Allowing admin request");
    next.run(req).await
}
