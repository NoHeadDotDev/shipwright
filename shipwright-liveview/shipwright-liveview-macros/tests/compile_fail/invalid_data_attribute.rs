use shipwright_liveview_macros::html;

fn main() {
    // This should fail: data attribute names must be lowercase
    let _ = html! {
        <div data-Value="test">Content</div>
    };
}