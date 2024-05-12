use axum::{extract::State, http::StatusCode, response::IntoResponse};
use eyre::WrapErr;
use maud::html;

use crate::{controllers, error::DoorstepError, state::AppState};

fn head(title: &str) -> maud::Markup {
    html! {
        head {
            link rel="stylesheet" href="https://unpkg.com/mvp.css";
            link rel="stylesheet" href="/admin/static/css/main.css";
            title { (title) }
            script type="module" {
                "import * as Turbo from 'https://esm.sh/@hotwired/turbo@8.0.4';"
            }
            script type="module" src="/admin/static/js/main.js" {}
            script type="module" src="/admin/static/js/controllers.js" {}
        }
    }
}

/// Page where admin users can:
///
/// - List existing backgrounds
/// - Upload new backgrounds
/// - Draw a new background on a canvas, based or not on an existing background
pub async fn background_admin_page(State(state): State<AppState>) -> impl IntoResponse {
    let backgrounds = controllers::list_backgrounds(&state)
        .await
        .wrap_err("Failed to list existing backgrounds")?;
    let current_background_name = state
        .background
        .read()
        .await
        .as_ref()
        .map(|b| b.name().to_owned())
        .unwrap_or_default();

    let width = state.config.background_width();
    let height = state.config.background_height();

    let markup = html! {
        html {
            (head("Homelab – Doorstep Admin – Backgrounds"))
            body {
                header {
                    h1 { "Homelab – Doorstep Admin – Backgrounds" }
                }
                main {
                    div class="main-container" {
                        // left panel – list of clickable backgrounds
                        div {
                            div
                                data-controller="background-preview"
                                data-background-preview-width-value={(width)}
                                data-background-preview-height-value={(height)}
                                class="background-preview"
                                style="display: none; position: fixed; z-index: 1000;"
                            {
                                    img
                                        class="background-preview-image"
                                        data-background-preview-target="preview";
                            }

                            h2 { "Background history" }
                            ul class="background-list" {
                                @for background in backgrounds {
                                    @let current = background.name() == current_background_name;
                                    li class="background-list-item" {
                                        a
                                            href="#"
                                            data-controller="previous-background"
                                            data-previous-background-is-current-value={(current)}
                                            data-previous-background-filename-value={(background.name())}
                                            data-previous-background-background-canvas-outlet=".background-canvas-container"
                                            data-previous-background-background-preview-outlet=".background-preview"
                                            data-action="click->previous-background#loadToCanvas mousemove->previous-background#preview mouseleave->previous-background#hidePreview"
                                        { (background.name()) }
                                    }
                                }
                            }
                        }
                        // right panel – blank canvas + form to upload new background
                        div {
                            div {
                                h2 { "Update" }
                                h3 { "Draw on existing background" }
                                div
                                    class="background-canvas-container"
                                    data-controller="background-canvas"
                                    data-background-canvas-width-value={(width)}
                                    data-background-canvas-height-value={(height)}
                                {
                                    canvas
                                        class="background-canvas"
                                        data-background-canvas-target="canvas"
                                    {}
                                    label for="line-width" {
                                        "Line width: "
                                        span data-background-canvas-target="lineWidth" { "8px" }
                                    }
                                    input
                                        id="line-width"
                                        type="range"
                                        min="1"
                                        max="32"
                                        value="8"
                                        data-action="input->background-canvas#changeLineWidth"
                                    {}

                                    label for="background-name" { "Name: " }
                                    input
                                        id="background-name"
                                        type="text"
                                        name="background-name"
                                        data-background-canvas-target="nameInput"
                                    {}

                                    div {
                                        // two buttons – save and clear
                                        button
                                            class="button-primary"
                                            data-action="click->background-canvas#save"
                                        { "Save" }
                                        button
                                            class="button-secondary"
                                            data-action="click->background-canvas#clearCanvas"
                                        { "Clear" }
                                    }
                                }

                                hr {}
                                h3 { "Or – upload one from your device" }
                                form method="post" action="./update/" enctype="multipart/form-data" {
                                    input type="file" name="background" accept="image/*";
                                    input type="submit" value="Upload";
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    Ok::<_, DoorstepError>((StatusCode::OK, markup))
}
