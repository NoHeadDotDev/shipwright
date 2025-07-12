// Example demonstrating the view! macro usage
// This would be used in a real LiveView application

use shipwright_liveview_macros::view;

// Example state structure
struct AppState {
    counter: i32,
    items: Vec<String>,
    show_message: bool,
    theme: String,
}

// Example component
struct Button {
    label: String,
    on_click: Box<dyn Fn()>,
}

// Example event handlers
fn increment_counter(state: &mut AppState) {
    state.counter += 1;
}

fn toggle_message(state: &mut AppState) {
    state.show_message = !state.show_message;
}

fn main() {
    let state = AppState {
        counter: 0,
        items: vec!["Item 1".to_string(), "Item 2".to_string(), "Item 3".to_string()],
        show_message: false,
        theme: "light".to_string(),
    };

    // Example 1: Basic HTML with dynamic content
    let basic_view = view! {
        <div class="container">
            <h1>{"Counter App"}</h1>
            <p>{"Current count: "}{state.counter}</p>
        </div>
    };

    // Example 2: Event handlers
    let interactive_view = view! {
        <div>
            <button on:click={increment_counter}>{"Increment"}</button>
            <button on:click={toggle_message}>{"Toggle Message"}</button>
        </div>
    };

    // Example 3: Conditional rendering
    let conditional_view = view! {
        <div>
            {if state.show_message {
                <div class="message">
                    <p>{"This is a conditional message!"}</p>
                </div>
            } else {
                <p>{"Click the button to show a message"}</p>
            }}
        </div>
    };

    // Example 4: Loops
    let list_view = view! {
        <ul>
            {for item in &state.items {
                <li>{item}</li>
            }}
        </ul>
    };

    // Example 5: Dynamic attributes and classes
    let dynamic_attrs_view = view! {
        <div class="app" class:dark={state.theme == "dark"}>
            <input type="text" value={state.counter.to_string()} />
            <span class:active={state.counter > 5}>{"High count!"}</span>
        </div>
    };

    // Example 6: Components
    let component_view = view! {
        <div>
            <h2>{"My App"}</h2>
            <Button 
                label={"Click me!"} 
                on_click={Box::new(|| println!("Button clicked!"))} 
            />
        </div>
    };

    // Example 7: Complex nested structure
    let complex_view = view! {
        <div class="app-container">
            <header>
                <h1>{"Todo App"}</h1>
                <Button label={"Add Todo"} on_click={Box::new(|| {})} />
            </header>
            
            <main>
                {if state.items.is_empty() {
                    <p class="empty-state">{"No items yet!"}</p>
                } else {
                    <div class="todo-list">
                        {for (idx, item) in state.items.iter().enumerate() {
                            <div class="todo-item" class:completed={idx % 2 == 0}>
                                <input type="checkbox" checked={idx % 2 == 0} />
                                <span>{item}</span>
                                <button on:click={move || {}}>{"Delete"}</button>
                            </div>
                        }}
                    </div>
                }}
            </main>
            
            <footer>
                <p>{"Total items: "}{state.items.len()}</p>
            </footer>
        </div>
    };
}

// Example helper module showing the expected output structure
mod shipwright_liveview {
    pub mod prelude {
        pub struct View {
            pub template_id: &'static str,
            pub static_template: &'static str,
            pub update_fn: Box<dyn Fn(&mut dyn std::any::Any, &mut Vec<Diff>)>,
            pub event_handlers: Vec<EventHandler>,
        }

        pub enum Diff {
            Text { id: usize, value: String },
            Attribute { id: usize, name: String, value: String },
            ClassToggle { id: usize, class: String, active: bool },
            Component { id: usize, component: Box<dyn std::any::Any> },
        }

        pub struct EventHandler {
            pub event_type: String,
            pub handler_id: usize,
            pub handler: Box<dyn Fn()>,
        }
    }
}