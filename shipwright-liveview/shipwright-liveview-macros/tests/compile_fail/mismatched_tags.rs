use shipwright_liveview_macros::html;

fn main() {
    // This should fail: mismatched opening and closing tags
    let _ = html! {
        <div>
            <span>Content</div>
        </span>
    };
}