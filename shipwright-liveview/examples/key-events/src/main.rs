use axum::{http::header, response::IntoResponse, routing::get, Router};
use shipwright_liveview::{
    event_data::EventData, html, live_view::Updated, Html, LiveView, LiveViewUpgrade,
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/bundle.js", shipwright_liveview::precompiled_js())
        .route("/hot-reload-client.js", get(serve_hot_reload_client));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn root(live: LiveViewUpgrade) -> impl IntoResponse {
    let view = View::default();

    live.response(move |embed| {
        html! {
            <!DOCTYPE html>
            <html>
                <head>
                </head>
                <body>
                    { embed.embed(view) }
                    <script src="/bundle.js"></script>
                    if cfg!(debug_assertions) {
                        <script src="/hot-reload-client.js"></script>
                        <script>
                            "if (typeof initHotReload !== 'undefined') {
                                const client = initHotReload('ws://localhost:3001/ws', {
                                    toastEnabled: true,
                                    showIndicator: true,
                                    enableDebugShortcuts: true
                                });
                            }"
                        </script>
                    }
                </body>
            </html>
        }
    })
}

#[derive(Default, Clone)]
struct View {
    count: u64,
    prev: Option<Msg>,
}

impl LiveView for View {
    type Message = Msg;

    fn update(mut self, msg: Msg, _data: Option<EventData>) -> Updated<Self> {
        self.count += 1;
        self.prev = Some(msg);
        Updated::new(self)
    }

    fn render(&self) -> Html<Self::Message> {
        html! {
            <div axm-window-keyup={ Msg::Key("window-keyup".to_owned()) } axm-key="escape" >
                <div>
                    "Keydown"
                    <br />
                    <input type="text" axm-keydown={ Msg::Key("keydown".to_owned()) } />
                </div>

                <div>
                    "Keydown (w debounce)"
                    <br />
                    <input
                        type="text"
                        axm-keydown={ Msg::Key("keydown-w-debounce".to_owned()) }
                        axm-debounce="500"
                    />
                </div>

                <div>
                    "Keyup"
                    <br />
                    <input type="text" axm-keyup={ Msg::Key("keyup".to_owned()) }/>
                </div>

                <hr />

                if let Some(event) = &self.prev {
                    <div>"Event count: " { self.count }</div>
                    <pre>
                        <code>
                            { format!("{:#?}", event) }
                        </code>
                    </pre>
                } else {
                    <div>
                        "No keys pressed yet"
                    </div>
                }
            </div>
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
enum Msg {
    Key(String),
}

async fn serve_hot_reload_client() -> impl IntoResponse {
    let client_js =
        include_str!("../../../shipwright-liveview-hotreload/client/hot-reload-client.js");

    (
        [(header::CONTENT_TYPE, "application/javascript")],
        client_js,
    )
}
