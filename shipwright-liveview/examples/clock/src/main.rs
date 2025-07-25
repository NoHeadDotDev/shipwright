use axum::{http::header, response::IntoResponse, routing::get, Router};
use shipwright_liveview::{
    event_data::EventData, html, live_view::Updated, Html, LiveView, LiveViewUpgrade,
};

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
    let format =
        time::format_description::parse("[hour]:[minute]:[second].[subsecond digits:6]").unwrap();

    let view = Clock { format };

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

#[derive(Clone)]
struct Clock {
    format: Vec<time::format_description::FormatItem<'static>>,
}

impl LiveView for Clock {
    type Message = ();

    fn mount(
        &mut self,
        _uri: axum::http::Uri,
        _request_headers: &axum::http::HeaderMap,
        handle: shipwright_liveview::live_view::ViewHandle<Self::Message>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(1));
            loop {
                interval.tick().await;
                if handle.send(()).await.is_err() {
                    return;
                }
            }
        });
    }

    fn update(self, _msg: Self::Message, _data: Option<EventData>) -> Updated<Self> {
        Updated::new(self)
    }

    fn render(&self) -> Html<Self::Message> {
        let now = time::OffsetDateTime::now_utc();

        html! {
            "Current time:" { now.format(&self.format).unwrap() }
        }
    }
}

async fn serve_hot_reload_client() -> impl IntoResponse {
    let client_js =
        include_str!("../../../shipwright-liveview-hotreload/client/hot-reload-client.js");

    (
        [(header::CONTENT_TYPE, "application/javascript")],
        client_js,
    )
}
