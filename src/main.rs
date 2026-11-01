use serde::Deserialize;
use tower_http::services::ServeDir;
use rand::RngExt;

use axum::response::Html;
use std::env;
use axum::{extract::State, Form, Router, routing::post, routing::get};
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Choice {
    Reels,
    Shorts,
    Tiktok,
}

#[derive(Deserialize)]
struct ChoiceForm {
    choice: Choice,
}

struct SOptions {
    reels: u8,
    shorts: u8,
    tiktok: u8,
}

const SERVERPAGE: &str = include_str!("../website/create_server.html");

// use openssl random for pincode :)
async fn create_pincode() -> String {
    let mut randomcode = rand::rng();
    (0..6)
        .map(|_| randomcode.random_range(32u8..=126) as char)
        .collect()
}

async fn show_pincode() -> Html<String> {
    let coder = create_pincode().await;
    Html(SERVERPAGE.replace("{{pincode}}", &coder))
}

async fn server_page(State(state): State<SharedOptions>) -> Html<String> {
    let opts = state.lock().unwrap();

    let choices = format!(
        r#"<form method="post" action="/scroll_choice">
             <button type="submit" name="choice" value="reels" class="scrollchoice">instagram reels ({})</button>
             <button type="submit" name="choice" value="tiktok" class="scrollchoice">tiktok ({})</button>
             <button type="submit" name="choice" value="shorts" class="scrollchoice">youtube shorts ({})</button>
           </form>"#,
           opts.reels, opts.tiktok, opts.shorts
    );

    Html(SERVERPAGE.replace("{{scrollchoice}}", &choices))
}

type SharedOptions = Arc<Mutex<SOptions>>;

async fn choose_brainrot(
    State(state): State<SharedOptions>,
    Form(form): Form<ChoiceForm>,
) -> String {
    let mut opts = state.lock().unwrap();

    match form.choice {
        Choice::Reels => opts.reels = opts.reels.saturating_add(1),
        Choice::Shorts => opts.shorts = opts.shorts.saturating_add(1),
        Choice::Tiktok => opts.tiktok = opts.tiktok.saturating_add(1),
    }

    format!("Reels: {}, Shorts: {}, Tiktok: {}", opts.reels, opts.shorts, opts.tiktok)
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let port_browser: u16 = args.get(2)
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    // states give the ability to outlive handlers
    let state: SharedOptions = Arc::new(Mutex::new(SOptions { reels: 0, shorts: 0, tiktok: 0 })); 

    let app = Router::new()
        .route("/server", get(server_page).post(show_pincode))
        .route("/scroll_choice", post(choose_brainrot))
        .with_state(state)
        .fallback_service(ServeDir::new("../website"));

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port_browser))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
