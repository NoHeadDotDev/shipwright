use axum::{extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};
use shipwright_liveview::{
    event_data::EventData, html, live_view::Updated, Html, LiveView, LiveViewUpgrade,
};

use crate::state::AppState;

/// Dashboard page with LiveView
pub async fn dashboard_page(
    State(state): State<AppState>,
    live: LiveViewUpgrade,
) -> impl IntoResponse {
    let view = Dashboard::new();

    live.response(move |embed| {
        html! {
            <!DOCTYPE html>
            <html>
                <head>
                    <title>"{{crate_name}} - Dashboard"</title>
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

#[derive(Clone)]
struct Dashboard {
    active_users: u32,
    total_requests: u64,
    current_time: String,
}

impl Dashboard {
    fn new() -> Self {
        Self {
            active_users: 42,
            total_requests: 1337,
            current_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        }
    }
}

impl LiveView for Dashboard {
    type Message = DashboardMessage;

    fn update(mut self, msg: DashboardMessage, _data: Option<EventData>) -> Updated<Self> {
        match msg {
            DashboardMessage::RefreshStats => {
                // Simulate updating stats
                self.active_users = fastrand::u32(10..=100);
                self.total_requests += fastrand::u64(1..=50);
                self.current_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
            }
        }

        Updated::new(self)
    }

    fn render(&self) -> Html<Self::Message> {
        html! {
            <div class="dashboard-page">
                <header class="dashboard-header">
                    <h1>"📊 Application Dashboard"</h1>
                    <button 
                        class="btn btn-primary"
                        axm-click={ DashboardMessage::RefreshStats }
                    >
                        "🔄 Refresh Stats"
                    </button>
                </header>
                
                <div class="dashboard-grid">
                    <div class="stat-card">
                        <h3>"👥 Active Users"</h3>
                        <span class="stat-value">{ self.active_users }</span>
                    </div>
                    
                    <div class="stat-card">
                        <h3>"📈 Total Requests"</h3>
                        <span class="stat-value">{ self.total_requests }</span>
                    </div>
                    
                    <div class="stat-card">
                        <h3>"⏰ Current Time"</h3>
                        <span class="stat-value time">{ &self.current_time }</span>
                    </div>
                    
                    <div class="stat-card">
                        <h3>"⚡ System Status"</h3>
                        <span class="stat-value status">"🟢 Healthy"</span>
                    </div>
                </div>
                
                <div class="dashboard-section">
                    <h2>"Recent Activity"</h2>
                    <div class="activity-list">
                        <div class="activity-item">
                            <span class="activity-icon">"✅"</span>
                            <span class="activity-text">"New user registered"</span>
                            <span class="activity-time">"2 minutes ago"</span>
                        </div>
                        <div class="activity-item">
                            <span class="activity-icon">"📝"</span>
                            <span class="activity-text">"System backup completed"</span>
                            <span class="activity-time">"15 minutes ago"</span>
                        </div>
                        <div class="activity-item">
                            <span class="activity-icon">"🔄"</span>
                            <span class="activity-text">"Database migration successful"</span>
                            <span class="activity-time">"1 hour ago"</span>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum DashboardMessage {
    RefreshStats,
}