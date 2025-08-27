use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::{Extension, State},
    http::{HeaderMap},

    routing::{MethodFilter, get, on},
};
use juniper_axum::{extract::JuniperRequest, graphiql, playground, response::JuniperResponse};
use tokio::net::TcpListener;
use tracing::info;

use crate::state::AppState;
use crate::{setup::SetupResult, state::AppData};

mod auth;
mod config;
mod schema;
mod setup;
mod state;

async fn graphql(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(schema): Extension<Arc<schema::Schema<'static>>>,
    JuniperRequest(request): JuniperRequest,
) -> JuniperResponse {
    let user = state::extract_user_from_headers(&headers);
    let app_data = AppData::new(state.db.clone(), user);
    JuniperResponse(request.execute(&schema, &app_data).await)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let SetupResult { db } = setup::setup_all().await.expect("setup failed");

    let schema = schema::schema();

    let app_state = AppState::new(AppData::new(db, None));

    let app = Router::new()
        .route("/", on(MethodFilter::GET.or(MethodFilter::POST), graphql))
        .route("/graphiql", get(graphiql("/", "/subscriptions")))
        .route("/playground", get(playground("/", "/subscriptions")))
        .with_state(app_state)
        .layer(Extension(Arc::new(schema)));

    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");
    info!("Server running at http://{}", addr);
    axum::serve(listener, app).await.expect("Server failed");
}
