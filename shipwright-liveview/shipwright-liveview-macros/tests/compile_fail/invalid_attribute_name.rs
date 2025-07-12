use shipwright_liveview_macros::html;

fn main() {
    // This should fail: attribute names cannot contain quotes
    let _ = html! {
        <div class"name"="value">Content</div>
    };
}