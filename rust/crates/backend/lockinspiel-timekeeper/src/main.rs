use axum::{Json, extract::FromRef, routing::get};
use clap::Parser;
use color_eyre::eyre::{self, Context};
use lockinspiel_backend_common::{ApiState, ServiceConfig, shutdown_signal};
use serde::Deserialize;
use tokio::net::TcpListener;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};

pub mod routes;

#[derive(Parser, Deserialize, Default)]
pub struct TimekeeperConfig {}

#[derive(Clone, FromRef)]
pub struct TimekeeperApiState {
    api_state: ApiState,
}

macro_rules! app_routes {
    ($state:ty) => {{
        let (router, mut api) = [
            routes!(
                routes::timer::post_timer,
                routes::timer::get_timers,
                routes::timer::modify_timer
            ),
            routes!(
                routes::tag::create_tag,
                routes::tag::get_tags,
                routes::tag::modify_tag,
                routes::tag::delete_tag,
            ),
            routes!(
                routes::time_split::create_time_split,
                routes::time_split::get_time_splits,
                routes::time_split::modify_time_split,
                routes::time_split::delete_time_split,
            ),
        ]
        .into_iter()
        .fold(OpenApiRouter::<$state>::new(), |router, routes| {
            router.routes(routes)
        })
        .split_for_parts();

        lockinspiel_backend_common::fill_in_openapi(&mut api);

        (router, api)
    }};
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let service_config = ServiceConfig::new("timekeeper");
    let (init_state, api_state) = lockinspiel_backend_common::init::<TimekeeperConfig>(
        service_config,
        lockinspiel_timekeeper_schema::MIGRATIONS,
    )
    .await
    .wrap_err("Failed to initialize API state")?;

    let (router, api) = app_routes!(TimekeeperApiState);

    let app = router
        .route("/", get(|| async { "up" }))
        .merge(Scalar::with_url("/timekeeper/openapi", api))
        .route(
            "/timekeeper/openapi/json",
            get(async move || Json(app_routes!(TimekeeperApiState).1)),
        )
        .layer(lockinspiel_backend_common::layer())
        .with_state(TimekeeperApiState { api_state });

    let listener = TcpListener::bind(init_state.addr)
        .await
        .wrap_err_with(|| format!("Failed to open listener on {}", init_state.addr))?;
    tracing::info!("Listening on {}", init_state.addr);

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .wrap_err("Failed to serve make service")?;

    init_state
        .shutdown()
        .wrap_err("Failed to shutdown OpenTelemetry services")
}
