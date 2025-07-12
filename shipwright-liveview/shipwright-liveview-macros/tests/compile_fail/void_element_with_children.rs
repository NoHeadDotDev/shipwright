use shipwright_liveview_macros::html;

fn main() {
    // This should fail: void elements cannot have children
    let _ = html! {
        <br>Content</br>
    };
}