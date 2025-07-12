use shipwright_liveview_macros::html;

fn main() {
    // This should fail: ID attribute cannot be empty
    let _ = html! {
        <div id="">Content</div>
    };
}