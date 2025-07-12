//! Demo of enhanced parsing capabilities

use shipwright_liveview_macros::html;

fn main() {
    // Set environment variable to enable enhanced parsing
    std::env::set_var("SHIPWRIGHT_HOT_RELOAD", "1");

    println!("Enhanced parsing demo");
    
    let title = "Demo Title";
    let items = vec!["Apple", "Banana", "Cherry"];
    let show_details = true;

    let _result = html! {
        <div class="demo-container">
            <header>
                <h1>{title}</h1>
            </header>
            <main>
                <ul>
                    for item in items {
                        <li class={format!("item-{}", item.to_lowercase())}>
                            <span>{item}</span>
                            if show_details {
                                <small>" - Fresh fruit"</small>
                            }
                        </li>
                    }
                </ul>
            </main>
        </div>
    };

    println!("Enhanced parsing completed successfully!");
}