use axum::{response::IntoResponse, routing::get, Router};
use serde::{Deserialize, Serialize};
use shipwright_liveview::{
    event_data::EventData, html, live_view::Updated, Html, LiveView, LiveViewUpgrade,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Start hot reload server in development
    #[cfg(debug_assertions)]
    {
        tokio::spawn(async {
            let addr = "127.0.0.1:3001".parse().unwrap();
            let watch_paths = vec![std::path::PathBuf::from("src")];
            let hot_reload_server =
                shipwright_liveview_hotreload::HotReloadServer::new(addr, watch_paths);
            if let Err(e) = hot_reload_server.start().await {
                eprintln!("Hot reload server failed to start: {}", e);
            }
        });
        println!("🔥 Hot reload server started on ws://localhost:3001");
    }

    let app = Router::new()
        .route("/", get(root))
        .route("/bundle.js", shipwright_liveview::precompiled_js());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 Server listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn root(live: LiveViewUpgrade) -> impl IntoResponse {
    let view = Counter::default();

    live.response(move |embed| {
        html! {
            <!DOCTYPE html>
            <html>
                <head>
                    <title>"Counter Example - Shipwright LiveView"</title>
                </head>
                <body>
                    { embed.embed(view) }
                    <script src="/bundle.js"></script>
                    if cfg!(debug_assertions) {
                        <script>
                            "// Hot Reload Client with Throttling
                            (function() {
                                const ws = new WebSocket('ws://localhost:3001/ws');
                                let lastReloadTime = 0;
                                const RELOAD_THROTTLE_MS = 2000; // 2 seconds
                                
                                function throttledReload() {
                                    const now = Date.now();
                                    if (now - lastReloadTime < RELOAD_THROTTLE_MS) {
                                        console.log('🔥 Hot reload: Throttled - ignoring rapid reload request');
                                        return;
                                    }
                                    
                                    lastReloadTime = now;
                                    console.log('🔥 Hot reload: Template updated - waiting for compilation...');
                                    
                                    // Close WebSocket before reload to prevent reconnection issues
                                    ws.close();
                                    
                                    // Show loading indicator
                                    document.body.style.opacity = '0.7';
                                    document.body.style.pointerEvents = 'none';
                                    
                                    // Wait for compilation by polling the server
                                    const checkCompilation = () => {
                                        fetch('/')
                                            .then(response => {
                                                if (response.ok) {
                                                    console.log('🔥 Compilation complete - reloading');
                                                    location.reload();
                                                } else {
                                                    // Still compiling, check again
                                                    setTimeout(checkCompilation, 200);
                                                }
                                            })
                                            .catch(() => {
                                                // Server not ready, check again
                                                setTimeout(checkCompilation, 200);
                                            });
                                    };
                                    
                                    // Start checking after initial delay
                                    setTimeout(checkCompilation, 500);
                                }
                                
                                ws.onmessage = function(event) {
                                    const msg = JSON.parse(event.data);
                                    console.log('🔍 Received message:', msg);
                                    if (msg.type === 'template_updated' || msg.type === 'batch_update') {
                                        throttledReload();
                                    }
                                };
                                ws.onopen = function() {
                                    console.log('🔥 Hot reload connected');
                                };
                                ws.onclose = function() {
                                    console.log('🔥 Hot reload disconnected');
                                    setTimeout(() => location.reload(), 1000);
                                };
                            })();"
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
            <div style="background-color: green; padding: 20px; font-family: Arial, sans-serif;">
                <h1>"🚀 Shipwright LiveView Counter"</h1>
                <div style="margin: 20px 0;">
                    <button
                        axm-click={ Msg::Decr }
                        style="padding: 10px 20px; margin: 5px; background: #ff6b6b; color: white; border: none; border-radius: 5px; cursor: pointer;"
                    >"-"</button>
                    <span style="margin: 0 20px; font-size: 24px; font-weight: bold;">
                        { self.count }
                    </span>
                    <button
                        axm-click={ Msg::Incr }
                        style="padding: 10px 20px; margin: 5px; background: #51cf66; color: white; border: none; border-radius: 5px; cursor: pointer;"
                    >"+"</button>
                </div>
                <p style="color: #666;">
                    "🔥 Try editing this fart the server is running!!!!"
                </p>
            </div>
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Msg {
    Incr,
    Decr,
}
