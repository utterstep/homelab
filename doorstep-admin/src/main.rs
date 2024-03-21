use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{ConnectInfo, Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use eyre::{Result, WrapErr};
use image::{imageops, io::Reader as ImageReader, Luma};
use tokio::{fs::File, net::TcpListener, sync::Mutex};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

mod config;
use config::Config;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    background: Arc<Mutex<Option<Vec<u8>>>>,
}

fn image_to_bitmap(image: &[u8], config: &Config) -> Result<Vec<u8>> {
    let img = ImageReader::new(std::io::Cursor::new(image))
        .with_guessed_format()
        .wrap_err("Failed to guess image format")?
        .decode()
        .wrap_err("Failed to decode image")?
        .into_luma8();

    let img = imageops::resize(
        &img,
        config.background_width(),
        config.background_height(),
        imageops::FilterType::Nearest,
    );

    let mut data: Vec<u8> = Vec::new();

    for y in 0..img.height() {
        let mut byte = 0u8;
        for x in 0..img.width() {
            let Luma([l]) = img.get_pixel(x, y);
            if x % 2 == 0 {
                byte = l >> 4;
            } else {
                byte |= l & 0xF0;
                data.push(byte);
            }
        }
        // For odd widths, push the last byte too
        if img.width() % 2 != 0 {
            data.push(byte);
        }
    }

    Ok(data)
}

async fn update_background(
    State(app_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut multipart: Multipart,
) -> Response {
    macro_rules! try_or_400 {
        ($expr:expr) => {
            match $expr {
                Ok(val) => val,
                Err(e) => {
                    return (StatusCode::BAD_REQUEST, format!("Bad request: {}", e)).into_response()
                }
            }
        };
    }

    if !app_state
        .config
        .allowed_configuration_from()
        .contains(&addr.ip())
    {
        return (StatusCode::FORBIDDEN, "Forbidden").into_response();
    }

    let mut name = None;
    let mut bytes = Vec::new();

    while let Some(field) = try_or_400!(multipart.next_field().await) {
        if field.name() != Some("background") {
            continue;
        }

        name = field.file_name().map(ToString::to_string);
        if name.is_none() {
            return (StatusCode::BAD_REQUEST, "No file name provided").into_response();
        }

        let mut stream = try_or_400!(field.bytes().await).to_vec();
        bytes.append(&mut stream);
    }

    let name = try_or_400!(name.ok_or_else(|| "No file name provided"));
    let file_path = app_state.config.backgrounds_dir().join(name);
    let mut file = try_or_400!(File::create(file_path).await);

    try_or_400!(tokio::io::copy(&mut &*bytes, &mut file).await);

    let bytes = try_or_400!(try_or_400!(
        tokio::task::spawn_blocking(move || image_to_bitmap(&bytes, &app_state.config)).await
    )
    .wrap_err("Failed to convert new background image"));

    let mut background = app_state.background.lock().await;
    *background = Some(bytes);

    (StatusCode::OK, "Background updated successfully").into_response()
}

async fn get_background(State(app_state): State<AppState>) -> Response {
    let background = app_state.background.lock().await;
    let background = match &*background {
        Some(bytes) => bytes.clone(),
        None => return (StatusCode::NOT_FOUND, "No background set").into_response(),
    };

    (StatusCode::OK, background).into_response()
}

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
                .with_thread_names(true)
                .with_thread_ids(true),
        )
        .init();

    let app_state = AppState {
        config: Arc::new(config),
        background: Arc::new(Mutex::new(None)),
    };

    // try listing all files in the backgrounds directory,
    // and load the latest one as the background
    let backgrounds_dir = app_state.config.backgrounds_dir();
    let mut latest_file = None;

    // (1) Create the backgrounds directory if it doesn't exist
    std::fs::create_dir_all(backgrounds_dir).wrap_err("Failed to create backgrounds directory")?; // (1

    // (2) Read the directory and find the latest file
    for entry in
        std::fs::read_dir(backgrounds_dir).wrap_err("Failed to read backgrounds directory")?
    {
        let entry = entry.wrap_err("Failed to read directory entry")?;
        let metadata = entry.metadata().wrap_err("Failed to read metadata")?;
        if metadata.is_file() {
            let modified = metadata
                .modified()
                .wrap_err("Failed to read modified time")?;
            if let Some((latest, _)) = latest_file {
                if modified > latest {
                    latest_file = Some((modified, entry.path()));
                }
            } else {
                latest_file = Some((modified, entry.path()));
            }
        }
    }

    // (3) Read the latest file and convert it to a bitmap
    if let Some((_, path)) = latest_file {
        let bytes = tokio::fs::read(path)
            .await
            .wrap_err("Failed to read background file")?;
        let bytes = image_to_bitmap(&bytes, &app_state.config)
            .wrap_err("Failed to convert stored background image")?;
        let mut background = app_state.background.lock().await;
        *background = Some(bytes);
    }

    let app = axum::Router::new()
        .route("/update", post(update_background))
        .route("/get", get(get_background))
        .with_state(app_state.clone());

    let addr = app_state.config.listen_addr().parse::<SocketAddr>()?;
    let lst = TcpListener::bind(addr).await?;

    axum::serve(lst, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}
