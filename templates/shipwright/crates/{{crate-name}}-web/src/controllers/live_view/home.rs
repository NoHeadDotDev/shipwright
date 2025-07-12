use axum::{extract::State, response::IntoResponse};
use shipwright_liveview::{html, Html, LiveView, LiveViewUpgrade};

use crate::state::AppState;

/// Home page with LiveView
pub async fn home_page(
    State(state): State<AppState>,
    live: LiveViewUpgrade,
) -> impl IntoResponse {
    let view = HomePage::new();

    live.response(move |embed| {
        html! {
            <!DOCTYPE html>
            <html>
                <head>
                    <title>"{{crate_name}} - Home"</title>
                    <meta charset="utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1" />
                    <link rel="stylesheet" href="/assets/css/styles.css" />
                </head>
                <body>
                    <div class="container">
                        { embed.embed(view) }
                    </div>
                    <script src="/bundle.js"></script>
                    
                    // Hot reload client for development
                    if cfg!(debug_assertions) {
                        <script>
                            "(function() {
                                const ws = new WebSocket('ws://localhost:3001/ws');
                                let lastReloadTime = 0;
                                const RELOAD_THROTTLE_MS = 2000;
                                
                                function throttledReload() {
                                    const now = Date.now();
                                    if (now - lastReloadTime < RELOAD_THROTTLE_MS) {
                                        console.log('🔥 Hot reload: Throttled - ignoring rapid reload request');
                                        return;
                                    }
                                    
                                    lastReloadTime = now;
                                    console.log('🔥 Hot reload: Template updated - waiting for compilation...');
                                    
                                    ws.close();
                                    document.body.style.opacity = '0.7';
                                    document.body.style.pointerEvents = 'none';
                                    
                                    const checkCompilation = () => {
                                        fetch('/')
                                            .then(response => {
                                                if (response.ok) {
                                                    console.log('🔥 Compilation complete - reloading');
                                                    location.reload();
                                                } else {
                                                    setTimeout(checkCompilation, 200);
                                                }
                                            })
                                            .catch(() => {
                                                setTimeout(checkCompilation, 200);
                                            });
                                    };
                                    
                                    setTimeout(checkCompilation, 500);
                                }
                                
                                ws.onmessage = function(event) {
                                    const msg = JSON.parse(event.data);
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

#[derive(Clone)]
struct HomePage {
    message: String,
}

impl HomePage {
    fn new() -> Self {
        Self {
            message: "Welcome to {{crate_name}}!".to_string(),
        }
    }
}

impl LiveView for HomePage {
    type Message = HomeMessage;

    fn update(
        mut self,
        msg: HomeMessage,
        _data: Option<shipwright_liveview::event_data::EventData>,
    ) -> shipwright_liveview::live_view::Updated<Self> {
        match msg {
            HomeMessage::UpdateMessage(new_message) => {
                self.message = new_message;
            }
        }
        
        shipwright_liveview::live_view::Updated::new(self)
    }

    fn render(&self) -> Html<Self::Message> {
        html! {
            <div class="home-page">
                <header class="header">
                    <h1>"🚀 {{ self.message }}"</h1>
                    <p>"A modern web application built with Shipwright"</p>
                </header>
                
                <main class="main-content">
                    <div class="features">
                        <div class="feature-card">
                            <h3>"⚡ Fast Development"</h3>
                            <p>"Hot reload and LiveView for rapid development"</p>
                        </div>
                        
                        <div class="feature-card">
                            <h3>"🔐 Type Safe"</h3>
                            <p>"Built with Rust for maximum safety and performance"</p>
                        </div>
                        
                        <div class="feature-card">
                            <h3>"🌐 Modern Web"</h3>
                            <p>"Axum-based architecture with real-time capabilities"</p>
                        </div>
                    </div>
                    
                    <div class="actions">
                        <a href="/live/counter" class="btn btn-primary">"Try Counter Example"</a>
                        <a href="/live/dashboard" class="btn btn-secondary">"View Dashboard"</a>
                    </div>
                </main>
            </div>
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
enum HomeMessage {
    UpdateMessage(String),
}