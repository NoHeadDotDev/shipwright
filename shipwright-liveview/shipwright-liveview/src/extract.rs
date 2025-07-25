//! Extractor for embedding live views in HTML templates.

use crate::{html::Html, life_cycle::run_view, LiveView};
use axum::{
    extract::{
        ws::{self, WebSocket, WebSocketUpgrade},
        FromRequestParts,
    },
    http::{HeaderMap, Uri},
    response::{IntoResponse, Response},
};
use futures_util::{
    sink::SinkExt,
    stream::{StreamExt, TryStreamExt},
};
use http::request::Parts;
use std::{convert::Infallible, fmt::Debug};

pub use crate::life_cycle::EmbedLiveView;

/// Extractor for embedding live views in HTML templates.
#[derive(Debug)]
pub struct LiveViewUpgrade {
    inner: LiveViewUpgradeInner,
}

#[derive(Debug)]
enum LiveViewUpgradeInner {
    Http,
    Ws(Box<(WebSocketUpgrade, Uri, HeaderMap)>),
}

impl<S> FromRequestParts<S> for LiveViewUpgrade
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Ok(ws) = WebSocketUpgrade::from_request_parts(parts, state).await {
            let uri = parts.uri.clone();
            let headers = parts.headers.clone();

            Ok(Self {
                inner: LiveViewUpgradeInner::Ws(Box::new((ws, uri, headers))),
            })
        } else {
            Ok(Self {
                inner: LiveViewUpgradeInner::Http,
            })
        }
    }
}

impl LiveViewUpgrade {
    /// Return a response that contains an embedded live view.
    ///
    /// # Example
    ///
    /// ```rust
    /// use axum::response::IntoResponse;
    /// use axum_live_view::{
    ///     event_data::EventData, html, live_view::Updated, Html, LiveView, LiveViewUpgrade,
    /// };
    /// use serde::{Deserialize, Serialize};
    /// use std::convert::Infallible;
    ///
    /// async fn handler(live: LiveViewUpgrade) -> impl IntoResponse {
    ///     let view = MyView::default();
    ///
    ///     live.response(|embed_live_view| {
    ///         html! {
    ///           { embed_live_view.embed(view) }
    ///
    ///           // Load the JavaScript. This will automatically initialize live view
    ///           // connections.
    ///           <script src="/assets/live-view.js"></script>
    ///         }
    ///     })
    /// }
    ///
    /// #[derive(Default)]
    /// struct MyView;
    ///
    /// impl LiveView for MyView {
    ///     // ...
    ///     # type Message = Msg;
    ///     # fn update(
    ///     #     mut self,
    ///     #     msg: Msg,
    ///     #     data: Option<EventData>,
    ///     # ) -> Updated<Self> {
    ///     #     todo!()
    ///     # }
    ///     # fn render(&self) -> Html<Self::Message> {
    ///     #     todo!()
    ///     # }
    /// }
    ///
    /// #[derive(Serialize, Deserialize, Debug, PartialEq)]
    /// enum Msg {}
    /// ```
    ///
    /// See the [root module docs](crate) for a more complete example.
    pub fn response<F, L>(self, gather_view: F) -> Response
    where
        L: LiveView,
        F: FnOnce(EmbedLiveView<'_, L>) -> Html<L::Message>,
    {
        match self.inner {
            LiveViewUpgradeInner::Http => {
                let embed = EmbedLiveView::noop();
                gather_view(embed).into_response()
            }
            LiveViewUpgradeInner::Ws(data) => {
                let (ws, uri, headers) = *data;
                let mut view = None;

                let embed = EmbedLiveView::new(&mut view);

                gather_view(embed);

                if let Some(view) = view {
                    ws.on_upgrade(|socket| run_view_on_socket(socket, view, uri, headers))
                        .into_response()
                } else {
                    ws.on_upgrade(|_| async {}).into_response()
                }
            }
        }
    }
}

async fn run_view_on_socket<L>(socket: WebSocket, view: L, uri: Uri, headers: HeaderMap)
where
    L: LiveView,
{
    let (write, read) = socket.split();

    let write = write.with(|msg| async move {
        let encoded_msg = ws::Message::Text(serde_json::to_string(&msg)?.into());
        Ok::<_, anyhow::Error>(encoded_msg)
    });
    futures_util::pin_mut!(write);

    let read = read
        .map_err(anyhow::Error::from)
        .and_then(|msg| async move {
            if let ws::Message::Text(text) = msg {
                serde_json::from_str(&text).map_err(Into::into)
            } else {
                anyhow::bail!("received message from socket that wasn't text")
            }
        });
    futures_util::pin_mut!(read);

    if let Err(err) = run_view(write, read, view, uri, headers).await {
        tracing::error!(%err, "encountered while processing socket");
    }
}
