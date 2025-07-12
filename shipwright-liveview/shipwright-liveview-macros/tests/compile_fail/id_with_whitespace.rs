use shipwright_liveview_macros::html;

fn main() {
    // This should fail: ID attribute cannot contain whitespace
    let _ = html! {
        <div id="my id">Content</div>
    };
}