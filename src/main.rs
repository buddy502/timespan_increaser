/*
    Use TCP for getting the website in the grid
*/

use std::io;
use axum::{Form, Router, routing::get, routing::post};

struct SOptions {
    reels: u8,
    shorts: u8,
    tiktok: u8,
}

async fn choose_brainrot() {

}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/scroll_choice", get(choose_brainrot));
}
