use shipwright_liveview_macros::html;

fn main() {
    // This should fail: element name cannot start with a number
    let _ = html! {
        <123invalid></123invalid>
    };
}