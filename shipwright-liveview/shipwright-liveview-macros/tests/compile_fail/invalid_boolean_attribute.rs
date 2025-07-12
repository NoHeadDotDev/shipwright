use shipwright_liveview_macros::html;

fn main() {
    // This should fail: boolean attributes should not have non-matching values
    let _ = html! {
        <input checked="true" />
    };
}