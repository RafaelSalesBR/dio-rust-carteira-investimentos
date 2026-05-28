use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use askama::Template;
use axum::{
    extract::{Form, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use dio_rust_carteira_investimentos::{build_positions, format_money, Asset, Position, Purchase, User};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    users: Arc<RwLock<HashMap<String, User>>>,
    assets: Arc<RwLock<HashMap<String, Asset>>>,
    purchases: Arc<RwLock<Vec<Purchase>>>,
    jwt_secret: String,
    admin_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    name: String,
    exp: usize,
}

#[derive(Debug, Deserialize)]
struct RegisterForm {
    name: String,
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct PurchaseForm {
    symbol: String,
    quantity: f64,
    paid_price_cents: i64,
}

#[derive(Debug, Deserialize)]
struct UpsertAsset {
    symbol: String,
    name: String,
    current_price_cents: i64,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate<'a> {
    message: &'a str,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    user_name: String,
    assets: Vec<AssetView>,
    positions: Vec<PositionView>,
}

#[derive(Debug)]
struct AssetView {
    symbol: String,
    name: String,
    current_price: String,
}

#[derive(Debug)]
struct PositionView {
    symbol: String,
    quantity: String,
    invested: String,
    current_value: String,
    profit: String,
}

impl From<&Asset> for AssetView {
    fn from(asset: &Asset) -> Self {
        Self {
            symbol: asset.symbol.clone(),
            name: asset.name.clone(),
            current_price: format_money(asset.current_price_cents),
        }
    }
}

impl From<&Position> for PositionView {
    fn from(position: &Position) -> Self {
        Self {
            symbol: position.symbol.clone(),
            quantity: format!("{:.4}", position.quantity),
            invested: format_money(position.invested_cents),
            current_value: format_money(position.current_value_cents),
            profit: format_money(position.profit_cents),
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("dio_rust_carteira_investimentos=debug,tower_http=debug")
        .init();

    let state = seed_state();
    let app = router(state);
    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    println!("Servidor iniciado em http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(login_page))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/dashboard", get(dashboard))
        .route("/purchases", post(record_purchase))
        .route("/api/assets", get(list_assets).post(upsert_asset))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn seed_state() -> AppState {
    let user = User::new("Rafael", "rafael@example.com", "123456");
    let btc = Asset::new("BTC", "Bitcoin", 350_000_00);
    let usd = Asset::new("USD", "Dólar comercial", 520);
    let purchases = vec![
        Purchase::new(user.id, "BTC", 0.01, 300_000_00),
        Purchase::new(user.id, "USD", 100.0, 500),
    ];

    AppState {
        users: Arc::new(RwLock::new(HashMap::from([(user.email.clone(), user)]))),
        assets: Arc::new(RwLock::new(HashMap::from([(btc.symbol.clone(), btc), (usd.symbol.clone(), usd)]))),
        purchases: Arc::new(RwLock::new(purchases)),
        jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "dio-rust-secret".to_string()),
        admin_secret: std::env::var("ADMIN_SECRET").unwrap_or_else(|_| "admin".to_string()),
    }
}

async fn login_page() -> impl IntoResponse {
    render(LoginTemplate { message: "Use rafael@example.com / 123456 ou cadastre um novo usuário." })
}

async fn register(State(state): State<AppState>, Form(form): Form<RegisterForm>) -> impl IntoResponse {
    let user = User::new(form.name, form.email, form.password);
    state.users.write().await.insert(user.email.clone(), user);
    Redirect::to("/")
}

async fn login(State(state): State<AppState>, Form(form): Form<LoginForm>) -> Response {
    let users = state.users.read().await;
    let Some(user) = users.get(&form.email.to_lowercase()) else {
        return render(LoginTemplate { message: "Usuário não encontrado." }).into_response();
    };

    if user.password != form.password {
        return render(LoginTemplate { message: "Senha inválida." }).into_response();
    }

    let token = issue_token(&state.jwt_secret, user);
    let cookie = format!("session={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age=86400");
    let mut response = Redirect::to("/dashboard").into_response();
    response.headers_mut().insert(header::SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
    response
}

async fn logout() -> impl IntoResponse {
    let mut response = Redirect::to("/").into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static("session=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0"),
    );
    response
}

async fn dashboard(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let Some(user_id) = authenticated_user_id(&state, &headers) else {
        return Redirect::to("/").into_response();
    };

    let users = state.users.read().await;
    let user_name = users
        .values()
        .find(|user| user.id == user_id)
        .map(|user| user.name.clone())
        .unwrap_or_else(|| "Investidor".to_string());
    drop(users);

    let assets: Vec<Asset> = state.assets.read().await.values().cloned().collect();
    let purchases = state.purchases.read().await.clone();
    let positions = build_positions(&assets, &purchases, user_id);

    let template = DashboardTemplate {
        user_name,
        assets: assets.iter().map(AssetView::from).collect(),
        positions: positions.iter().map(PositionView::from).collect(),
    };

    render(template).into_response()
}

async fn record_purchase(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<PurchaseForm>,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&state, &headers) else {
        return Redirect::to("/");
    };

    let purchase = Purchase::new(user_id, form.symbol, form.quantity, form.paid_price_cents);
    state.purchases.write().await.push(purchase);
    Redirect::to("/dashboard")
}

async fn list_assets(State(state): State<AppState>) -> Json<Vec<Asset>> {
    Json(state.assets.read().await.values().cloned().collect())
}

async fn upsert_asset(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpsertAsset>,
) -> impl IntoResponse {
    let is_admin = headers
        .get("x-admin-secret")
        .and_then(|value| value.to_str().ok())
        .map(|value| value == state.admin_secret)
        .unwrap_or(false);

    if !is_admin {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({ "error": "admin secret inválida" })));
    }

    let asset = Asset::new(payload.symbol, payload.name, payload.current_price_cents);
    state.assets.write().await.insert(asset.symbol.clone(), asset.clone());
    (StatusCode::OK, Json(serde_json::json!(asset)))
}

fn render<T: Template>(template: T) -> Html<String> {
    Html(template.render().unwrap_or_else(|error| format!("Erro ao renderizar template: {error}")))
}

fn issue_token(secret: &str, user: &User) -> String {
    let claims = Claims {
        sub: user.id.to_string(),
        name: user.name.clone(),
        exp: 4_102_444_800,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).unwrap()
}

fn authenticated_user_id(state: &AppState, headers: &HeaderMap) -> Option<Uuid> {
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    let token = cookie
        .split(';')
        .map(str::trim)
        .find_map(|part| part.strip_prefix("session="))?;
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    ).ok()?;
    Uuid::parse_str(&decoded.claims.sub).ok()
}
