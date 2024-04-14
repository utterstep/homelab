use std::{net::SocketAddr, sync::Arc};

use eyre::WrapErr;
use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{debug, error, info};

use crate::{
    app_state::AppState,
    log::{db::DbAccessLogEntry, AccessLogEntry},
};

pub async fn handle_stream(app_state: Arc<AppState>, socket: TcpStream, peer: SocketAddr) {
    let mut framed = Framed::new(socket, LinesCodec::new());

    while let Some(line) = framed.next().await {
        let frame_uuid = uuid::Uuid::now_v7();
        let span = tracing::info_span!("frame", peer_addr = %peer, frame_uuid = %frame_uuid);
        let _enter = span.enter();

        debug!(parent: &span, "Next frame");
        match line {
            Ok(line) => {
                debug!(parent: &span, frame_len = line.len(), "Received line");
                let access_log_entry: AccessLogEntry = serde_json::from_str(&line).unwrap();
                debug!(parent: &span, "Parsed line");

                let db_access_log_entry = DbAccessLogEntry::new(
                    frame_uuid,
                    app_state.config().service_name(),
                    app_state.config().environment(),
                    access_log_entry,
                );

                let client = match app_state
                    .ch_pool()
                    .get()
                    .await
                    .wrap_err("Failed to get CH client from pool")
                {
                    Ok(client) => client,
                    Err(e) => {
                        error!(parent: &span, "Failed to get CH client from pool: {}", e);
                        continue;
                    }
                };

                match client
                    .insert_native_block(
                        "INSERT INTO access_log SETTINGS async_insert=1, wait_for_async_insert=0 FORMAT NATIVE",
                        vec![db_access_log_entry],
                    )
                    .await
                {
                    Ok(_) => {
                        info!(parent: &span, "Inserted log entry");
                    }
                    Err(e) => {
                        error!(parent: &span, "Failed to insert log entry: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to read line: {}", e);
            }
        }
    }
}
