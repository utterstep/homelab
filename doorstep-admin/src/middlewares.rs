use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{AppendHeaders, IntoResponse, Response},
    RequestExt,
};
use axum_auth::AuthBasic;
use tracing::{info, warn};

use crate::state::AppState;

#[tracing::instrument(skip(app_state, addr, req, next), fields(ip = %addr.ip(), port = addr.port()))]
pub async fn admin_basic_auth(
    State(app_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut req: Request,
    next: Next,
) -> Response {
    let auth = req.extract_parts::<AuthBasic>().await;
    let headers = AppendHeaders([("WWW-Authenticate", "Basic realm=\"Rezo doorstep admin\"")]);
    let admin_user = app_state.config.admin_user();
    let admin_pass = app_state.config.admin_password();

    match auth {
        Ok(AuthBasic((user, pass))) => {
            if let Some(pass) = pass {
                if user == admin_user && pass == admin_pass {
                    info!(%addr, "Authenticated");
                    return next.run(req).await;
                }
            }
            warn!(%addr, "Unauthorized");

            return (StatusCode::UNAUTHORIZED, headers, "Unauthorized").into_response();
        }
        Err(e) => {
            warn!(%addr, ?e, "Failed to extract basic auth");

            return (StatusCode::UNAUTHORIZED, headers, e.1).into_response();
        }
    }
}
