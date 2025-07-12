// Demonstration of the view! macro capabilities
// This file shows how the macro would be used in a real application

use shipwright_liveview_macros::view;

fn demo_basic_html() {
    // Basic HTML structure
    let html = view! {
        <div class="container">
            <h1>{"Welcome to Shipwright LiveView"}</h1>
            <p>{"This is a reactive template system"}</p>
        </div>
    };
    
    println!("Generated template ID: {}", html.template_id);
    println!("Static HTML: {}", html.static_template);
}

fn demo_dynamic_content() {
    let count = 42;
    let message = "Hello, World!";
    
    // Dynamic content with Rust expressions
    let html = view! {
        <div>
            <h2>{message}</h2>
            <p>{"The count is: "}{count}</p>
            <p>{"Double the count: "}{count * 2}</p>
        </div>
    };
}

fn demo_event_handlers() {
    // Event handlers with type-safe registration
    let html = view! {
        <div>
            <button on:click={|| println!("Button clicked!")}>
                {"Click me"}
            </button>
            <input 
                type="text"
                on:change={|e| println!("Input changed")}
                on:keydown={|e| println!("Key pressed")}
            />
        </div>
    };
    
    println!("Registered {} event handlers", html.event_handlers.len());
}

fn demo_conditionals() {
    let is_logged_in = true;
    let username = "Alice";
    
    // Conditional rendering
    let html = view! {
        <div>
            {if is_logged_in {
                <div class="welcome">
                    <h1>{"Welcome back, "}{username}{"!"}</h1>
                    <button on:click={|| println!("Logout")}>{"Logout"}</button>
                </div>
            } else {
                <div class="login">
                    <h1>{"Please log in"}</h1>
                    <button on:click={|| println!("Login")}>{"Login"}</button>
                </div>
            }}
        </div>
    };
}

fn demo_loops() {
    let items = vec!["Rust", "Axum", "LiveView"];
    
    // Loop rendering
    let html = view! {
        <div>
            <h2>{"Technologies"}</h2>
            <ul>
                {for item in &items {
                    <li>
                        <span class="tech-item">{item}</span>
                    </li>
                }}
            </ul>
        </div>
    };
}

fn demo_dynamic_attributes() {
    let is_active = true;
    let is_disabled = false;
    let theme = "dark";
    
    // Dynamic attributes and conditional classes
    let html = view! {
        <div 
            class="app"
            class:active={is_active}
            class:disabled={is_disabled}
        >
            <input 
                type="text"
                value={theme}
                disabled={is_disabled}
            />
            <span class:highlight={theme == "dark"}>
                {"Current theme: "}{theme}
            </span>
        </div>
    };
}

fn demo_components() {
    // Component usage (components start with uppercase)
    let html = view! {
        <div class="app">
            <Header title={"My App"} />
            <MainContent>
                <Card title={"Feature 1"} active={true} />
                <Card title={"Feature 2"} active={false} />
            </MainContent>
            <Footer year={2025} />
        </div>
    };
}

fn demo_complex_example() {
    let todos = vec![
        ("Learn Rust", false),
        ("Build with Axum", true),
        ("Master LiveView", false),
    ];
    let show_completed = true;
    
    // Complex nested structure
    let html = view! {
        <div class="todo-app">
            <header>
                <h1>{"Todo List"}</h1>
                <button on:click={|| println!("Add todo")}>
                    {"Add New"}
                </button>
            </header>
            
            <main>
                <div class="filters">
                    <label>
                        <input 
                            type="checkbox"
                            checked={show_completed}
                            on:change={|_| println!("Toggle filter")}
                        />
                        {"Show completed"}
                    </label>
                </div>
                
                <ul class="todo-list">
                    {for (index, (text, completed)) in todos.iter().enumerate() {
                        {if show_completed || !completed {
                            <li 
                                class="todo-item"
                                class:completed={*completed}
                            >
                                <input 
                                    type="checkbox"
                                    checked={*completed}
                                    on:change={move |_| println!("Toggle todo {}", index)}
                                />
                                <span>{text}</span>
                                <button 
                                    class="delete"
                                    on:click={move |_| println!("Delete todo {}", index)}
                                >
                                    {"×"}
                                </button>
                            </li>
                        }}
                    }}
                </ul>
            </main>
            
            <footer>
                <p>
                    {todos.iter().filter(|(_, done)| !done).count()}
                    {" items left"}
                </p>
            </footer>
        </div>
    };
}

// Mock structures for demonstration
struct Header { title: String }
struct MainContent;
struct Card { title: String, active: bool }
struct Footer { year: i32 }

// Mock View structure that would be provided by the main crate
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

fn main() {
    println!("Shipwright LiveView Macro Demonstrations:");
    println!("=========================================");
    
    demo_basic_html();
    demo_dynamic_content();
    demo_event_handlers();
    demo_conditionals();
    demo_loops();
    demo_dynamic_attributes();
    demo_components();
    demo_complex_example();
}