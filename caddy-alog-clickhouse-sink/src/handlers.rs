use std::{net::SocketAddr, sync::Arc};

use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{debug, error, info, Instrument};

use crate::{
    app_state::AppState,
    log::{db::DbAccessLogEntry, AccessLogEntry},
};

pub async fn handle_stream(app_state: Arc<AppState>, socket: TcpStream, peer: SocketAddr) {
    let mut framed = Framed::new(socket, LinesCodec::new());

    while let Some(line) = framed.next().await {
        let frame_uuid = uuid::Uuid::now_v7();
        let span = tracing::info_span!("frame", peer_addr = %peer, frame_uuid = %frame_uuid);
        let app_state = Arc::clone(&app_state);

        // running everything inside the async block to correctly instrument it
        // (see documentation for the Span::enter method from the tracing crate for more details)
        async move {
            debug!("Next frame");
            match line {
                Ok(line) => {
                    debug!(frame_len = line.len(), "Received line");
                    let access_log_entry: AccessLogEntry = serde_json::from_str(&line).unwrap();
                    debug!("Parsed line");

                    let db_access_log_entry = DbAccessLogEntry::new(
                        frame_uuid,
                        app_state.config().service_name(),
                        app_state.config().environment(),
                        access_log_entry,
                    );

                    let client = match app_state.ch_pool().get().await {
                        Ok(client) => client,
                        Err(e) => {
                            error!("Failed to get CH client from pool: {}", e);
                            return;
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
                            info!("Inserted log entry");
                        }
                        Err(e) => {
                            error!("Failed to insert log entry: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read line: {}", e);
                }
            }
        }.instrument(span)
        .await
    }
}
