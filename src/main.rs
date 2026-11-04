use axum::{
    extract::State,
    response::{Html, Redirect},
    routing::get,
    Form, Router,
    http::StatusCode
};
use rand::RngExt;
use serde::Deserialize;
use std::collections::HashSet;
use std::env;
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Action {
    Reels,
    Shorts,
    Tiktok,
    Pincode,
}

#[derive(Deserialize)]
struct ActionForm {
    action: Action,
}

struct SOptions {
    reels: u8,
    shorts: u8,
    tiktok: u8,
    pincodes: HashSet<String>,

    latest_choice: Option<Action>,
    latest_pincode: String,
}
type SharedOptions = Arc<Mutex<SOptions>>;

const SERVERPAGE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/website/create_server.html"));

const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

async fn _enter_pincode() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}

// return pincodes struct then show pincode by getting
// the string when created
fn create_pincode(state: &SharedOptions) -> String {
    let mut rng = rand::rng();
    let mut opts = state.lock().unwrap();
    // has copy trait because it's going into return val & PinCodes.codes
    // no symbols
    let code: String = (0..6)
        .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
        .collect();

    opts.latest_pincode = code.clone();
    return code;
}

async fn delete_pincode(State(curstate): State<SharedOptions>) -> Redirect {
    // if every user presses the Leave button and there are 0 users destroy pincode
    // too many users would use all the pincodes
    // if leave is pressed and there are 0 counts in all buttons, remove code from hash

    let mut opts = curstate.lock().unwrap();

    // if there are no users using pincode, check if all buttons are , delete it
    if (opts.reels == 0) && (opts.shorts == 0) && (opts.tiktok == 0) {
        let pincodeborrow = opts.latest_pincode.clone();
        opts.pincodes.remove(&pincodeborrow);
    }

    Redirect::to("/")
}

async fn delete_button_count(State(curstate): State<SharedOptions>) -> Redirect {
    // if a user presses leave button, remove the count from the button they pressed

    let mut opts = curstate.lock().unwrap();

    match opts.latest_choice {
        Some(Action::Reels) => opts.reels = opts.reels.saturating_sub(1),
        Some(Action::Shorts) => opts.shorts = opts.shorts.saturating_sub(1),
        Some(Action::Tiktok) => opts.tiktok = opts.tiktok.saturating_sub(1),
        _ => {}
    }
    
    Redirect::to("/")
}

async fn choice_form(State(state): State<SharedOptions>) -> Html<String> {
    {
        let opts = state.lock().unwrap();
        if opts.latest_pincode.is_empty() {
            drop(opts);
            create_pincode(&state);
        }
    }
    let opts = state.lock().unwrap();

    let choices = if opts.latest_choice.is_some() {
        String::new()
    } else {
        format!(
            r#"<form method="post" action="/server">
             <button type="submit" name="action" value="reels" class="scrollchoice">instagram reels ({})</button>
             <button type="submit" name="action" value="tiktok" class="scrollchoice">tiktok ({})</button>
             <button type="submit" name="action" value="shorts" class="scrollchoice">youtube shorts ({})</button>
           </form>"#,
           opts.reels, opts.tiktok, opts.shorts
        )
    };

        let window = match opts.latest_choice {
        Some(Action::Reels) => r#"<iframe src="https://instagram.com" width="600" height="400"></iframe>"#,
        Some(Action::Shorts) => r#"<a href="https://www.youtube.com/shorts" target="_blank" rel="noopener"></a>"#,
        Some(Action::Tiktok) => r#"<iframe src="https://tiktok.com" width="600" height="400"></iframe>"#,
        _ => "",
    };
    let code = format!(r#"<span id="codetext">{}</span>"#, opts.latest_pincode);

    Html(
        SERVERPAGE
            .replace("{{scrollchoice}}", &choices)
            .replace("{{pincode}}", &code)
            .replace("{{choicewindows}}", window)
    )
}

async fn create_server_form(
    State(state): State<SharedOptions>,
    Form(form): Form<ActionForm>,
) -> Redirect {
    match form.action {
        Action::Reels => {
            let mut opts = state.lock().unwrap();
            opts.reels = opts.reels.saturating_add(1);
            opts.latest_choice = Some(Action::Reels);
        }
        Action::Shorts => {
            let mut opts = state.lock().unwrap();
            opts.shorts = opts.shorts.saturating_add(1);
            opts.latest_choice = Some(Action::Shorts);
        }
        Action::Tiktok => {
            let mut opts = state.lock().unwrap();
            opts.tiktok = opts.tiktok.saturating_add(1);
            opts.latest_choice = Some(Action::Tiktok);
        }
        _ => {
        }
    }
    return Redirect::to("/server")
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let port_browser: u16 = args.get(1)
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    // states give the ability to outlive handlers
    // ! the state only has 1 server so only 1 state exists
    // ! another pincode won't work because the state is saved across the whole server
    // !! fix is probably to create something like a state hashset
    let state: SharedOptions = Arc::new(Mutex::new(SOptions {
        reels: 0,
        shorts: 0,
        tiktok: 0,
        pincodes: HashSet::new(),
        latest_pincode: String::new(),
        latest_choice: None,
    }));
    // CARGO_MANIFEST_DIR is the dir where cargo.toml lives
    const WEBSITE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/website");

    let app = Router::new()
        .route("/server", get(choice_form).post(create_server_form))
        .route("/delete_choice_count", get(delete_button_count))
        .route("/delete_pincode", get(delete_pincode))
        .with_state(state)
        .fallback_service(ServeDir::new(WEBSITE));

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port_browser))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
