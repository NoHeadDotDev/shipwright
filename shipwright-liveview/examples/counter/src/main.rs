use axum::{response::IntoResponse, routing::get, Router};
use serde::{Deserialize, Serialize};
use shipwright_liveview::{
    event_data::EventData, html, live_view::Updated, Html, LiveView, LiveViewUpgrade,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Initialize hot reload system in development
    #[cfg(debug_assertions)]
    {
        shipwright_liveview_hotreload::init_hot_reload();
        println!("🔥 Enhanced hot reload system initialized!");
    }

    let app = Router::new()
        .route("/", get(root))
        .route("/bundle.js", shipwright_liveview::precompiled_js())
        .route("/hot-reload-client.js", get(hot_reload_client_js));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Server listening on {}", listener.local_addr().unwrap());
    println!("📝 Use 'shipwright dev' for enhanced hot reload!");
    axum::serve(listener, app).await.unwrap();
}

async fn hot_reload_client_js() -> impl IntoResponse {
    // Serve the enhanced hot reload client
    let client_js = include_str!("../../../shipwright-liveview-hotreload/client/hot-reload-client.js");
    ([("content-type", "application/javascript")], client_js)
}

async fn root(live: LiveViewUpgrade) -> impl IntoResponse {
    let view = Counter::default();

    live.response(move |embed| {
        html! {
            <!DOCTYPE html>
            <html>
                <head>
                    <title>"🚀 Enhanced Counter - Shipwright LiveView"</title>
                    <meta charset="utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1" />
                    <style>
                        "body { margin: 0; font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); min-height: 100vh; }"
                        ".container { display: flex; justify-content: center; align-items: center; min-height: 100vh; padding: 20px; }"
                        ".card { background: white; border-radius: 20px; padding: 40px; box-shadow: 0 20px 40px rgba(0,0,0,0.1); text-align: center; max-width: 500px; }"
                        ".title { color: #4a5568; margin-bottom: 30px; font-size: 2.5rem; font-weight: 700; }"
                        ".counter-display { font-size: 4rem; font-weight: 800; color: #2d3748; margin: 30px 0; text-shadow: 2px 2px 4px rgba(0,0,0,0.1); }"
                        ".button-group { display: flex; gap: 20px; justify-content: center; margin: 30px 0; }"
                        ".btn { padding: 15px 30px; font-size: 1.5rem; font-weight: 600; border: none; border-radius: 12px; cursor: pointer; transition: all 0.3s ease; color: white; }"
                        ".btn:hover { transform: translateY(-2px); box-shadow: 0 8px 16px rgba(0,0,0,0.2); }"
                        ".btn-decr { background: linear-gradient(135deg, #ff6b6b, #ee5a52); }"
                        ".btn-incr { background: linear-gradient(135deg, #51cf66, #40c057); }"
                        ".description { color: #718096; font-size: 1.1rem; line-height: 1.6; margin-top: 30px; }"
                        ".hot-reload-info { background: #f7fafc; border-left: 4px solid #4299e1; padding: 15px; margin-top: 20px; border-radius: 8px; }"
                        ".emoji { font-size: 1.5em; }"
                    </style>
                </head>
                <body>
                    <div class="container">
                        { embed.embed(view) }
                    </div>
                    
                    <script src="/bundle.js"></script>
                    
                    if cfg!(debug_assertions) {
                        <script src="/hot-reload-client.js"></script>
                        <script>
                            "// Initialize enhanced hot reload client
                            if (typeof initHotReload !== 'undefined') {
                                const client = initHotReload('ws://localhost:3001/ws', {
                                    toastEnabled: true,
                                    showIndicator: true,
                                    enableDebugShortcuts: true
                                });
                                console.log('🔥 Enhanced hot reload client initialized!');
                                console.log('📝 Try editing the template and save to see instant updates!');
                                console.log('⌨️  Debug shortcuts: Ctrl+Shift+R (reconnect), Ctrl+Shift+H (stats), Ctrl+Shift+T (toggle toasts)');
                            } else {
                                console.warn('Hot reload client not available');
                            }"
                        </script>
                    }
                </body>
            </html>
        }
    })
}

#[derive(Default, Clone)]
struct Counter {
    count: u64,
}

impl LiveView for Counter {
    type Message = Msg;

    fn update(mut self, msg: Msg, _data: Option<EventData>) -> Updated<Self> {
        match msg {
            Msg::Incr => self.count += 1,
            Msg::Decr => {
                if self.count > 0 {
                    self.count -= 1;
                }
            }
        }

        Updated::new(self)
    }

    fn render(&self) -> Html<Self::Message> {
        html! {
            <div class="card">
                <h1 class="title">
                    <span class="emoji">"🚀"</span>
                    " Shipwright LiveView "
                    <span class="emoji">"🔥"</span>
                </h1>
                <div class="counter-display">
                    { self.count }
                </div>
                <div class="button-group">
                    <button
                        axm-click={ Msg::Decr }
                        class="btn btn-decr"
                    >
                        <span class="emoji">"➖"</span>
                        " Decrement"
                    </button>
                    <button
                        axm-click={ Msg::Incr }
                        class="btn btn-incr"
                    >
                        <span class="emoji">"➕"</span>
                        " Increment"
                    </button>
                </div>
                <div class="description">
                    <p>"🎯 This is a "<strong>"reactive counter"</strong>" built with Shipwright LiveView!"</p>
                    <p>"✨ Click the buttons to see instant state updates."</p>
                </div>
                <div class="hot-reload-info">
                    <p>
                        <strong>"🔥 Enhanced Hot Reload Active"</strong>
                    </p>
                    <p>"Try editing this template and saving - you'll see instant updates with state preservation!"</p>
                    <p>
                        <small>"💡 Features: DOM patching • Toast notifications • State preservation • Connection resilience"</small>
                    </p>
                </div>
            </div>
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Msg {
    Incr,
    Decr,
}