//! Basic example of hot reload integration

use shipwright_liveview_hotreload::{
    protocol::{HotReloadMessage, TemplateId},
    runtime::{init_hot_reload, RegisteredTemplate, TemplateRegistry},
};
use std::path::PathBuf;

// Simulate a view! macro that registers itself
macro_rules! view {
    ($content:tt) => {{
        // Register this template with the runtime
        let template = RegisteredTemplate {
            id: TemplateId::new(
                PathBuf::from(file!()),
                line!(),
                column!(),
            ),
            component_type: "ExampleComponent".to_string(),
            function_name: "render".to_string(),
        };
        
        let registry = TemplateRegistry::global();
        tokio::spawn(async move {
            let mut reg = registry.write().await;
            reg.register(template);
        });
        
        // Return the template content
        stringify!($content)
    }};
}

struct ExampleComponent {
    count: u32,
}

impl ExampleComponent {
    fn render(&self) -> String {
        view! {
            <div>
                <h1>Hot Reload Example</h1>
                <p>Count: {self.count}</p>
                <button>Increment</button>
            </div>
        }
        .to_string()
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Initialize hot reload in development
    #[cfg(debug_assertions)]
    {
        println!("Initializing hot reload client...");
        tokio::spawn(async {
            if let Err(e) = init_hot_reload("ws://localhost:3001/ws".to_string()).await {
                eprintln!("Failed to initialize hot reload: {}", e);
            }
        });
    }
    
    // Simulate component rendering
    let component = ExampleComponent { count: 0 };
    println!("Initial render: {}", component.render());
    
    // Keep the application running
    println!("Application running. Edit the view! macro content to see hot reload in action.");
    println!("Press Ctrl+C to exit.");
    
    // In a real application, this would be your server loop
    tokio::signal::ctrl_c().await.unwrap();
    
    println!("Shutting down...");
}