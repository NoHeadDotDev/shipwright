use axum::{extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};
use shipwright_liveview::{
    event_data::EventData, html, live_view::Updated, Html, LiveView, LiveViewUpgrade,
};

use crate::state::AppState;

/// Counter page with LiveView
pub async fn counter_page(
    State(state): State<AppState>,
    live: LiveViewUpgrade,
) -> impl IntoResponse {
    let view = Counter::default();

    live.response(move |embed| {
        html! {
            <!DOCTYPE html>
            <html>
                <head>
                    <title>"{{crate_name}} - Counter"</title>
                    <meta charset="utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1" />
                    <link rel="stylesheet" href="/assets/css/styles.css" />
                </head>
                <body>
                    <div class="container">
                        <nav class="nav">
                            <a href="/live/">"← Back to Home"</a>
                        </nav>
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

#[derive(Default, Clone)]
struct Counter {
    count: i64,
}

impl LiveView for Counter {
    type Message = CounterMessage;

    fn update(mut self, msg: CounterMessage, _data: Option<EventData>) -> Updated<Self> {
        match msg {
            CounterMessage::Increment => self.count += 1,
            CounterMessage::Decrement => self.count -= 1,
            CounterMessage::Reset => self.count = 0,
        }

        Updated::new(self)
    }

    fn render(&self) -> Html<Self::Message> {
        html! {
            <div class="counter-page">
                <h1>"🔢 Interactive Counter"</h1>
                <div class="counter-display">
                    <span class="counter-value">{ self.count }</span>
                </div>
                <div class="counter-controls">
                    <button 
                        class="btn btn-danger"
                        axm-click={ CounterMessage::Decrement }
                    >
                        "−"
                    </button>
                    <button 
                        class="btn btn-secondary"
                        axm-click={ CounterMessage::Reset }
                    >
                        "Reset"
                    </button>
                    <button 
                        class="btn btn-success"
                        axm-click={ CounterMessage::Increment }
                    >
                        "+"
                    </button>
                </div>
                <p class="counter-description">
                    "This counter updates in real-time using Shipwright LiveView. "
                    "Try clicking the buttons to see the magic!"
                </p>
            </div>
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum CounterMessage {
    Increment,
    Decrement,
    Reset,
}