
# LiveView Guide

LiveView enables you to build rich, interactive web applications with server-side rendered HTML that updates in real-time over WebSocket connections. This guide covers everything you need to know about building LiveView applications with hello-world-local.

## Table of Contents

1. [Introduction](#introduction)
2. [Basic Concepts](#basic-concepts)
3. [Creating Your First Component](#creating-your-first-component)
4. [Event Handling](#event-handling)
5. [State Management](#state-management)
6. [HTML Templates](#html-templates)
7. [Component Composition](#component-composition)
8. [Advanced Patterns](#advanced-patterns)
9. [Best Practices](#best-practices)
10. [Examples](#examples)

## Introduction

LiveView is a server-side rendering approach that maintains a persistent connection between the client and server. When the application state changes, only the parts of the DOM that need updating are sent to the client and patched in place.

### Benefits

- **No JavaScript Required**: Build interactive UIs with only server-side Rust
- **Real-time Updates**: Automatic UI updates as state changes
- **SEO Friendly**: Server-side rendered HTML
- **Type Safety**: Full Rust type checking for your UI logic
- **Simplified Architecture**: Single language for frontend and backend

## Basic Concepts

### LiveView Trait

Every LiveView component implements the `LiveView` trait:

```rust
use shipwright_liveview::{LiveView, Html, Updated, EventData};

pub trait LiveView {
    type Message;
    
    fn update(self, msg: Self::Message, data: Option<EventData>) -> Updated<Self>;
    fn render(&self) -> Html<Self::Message>;
}
```

### Component State

Components are structs that hold your application state:

```rust
#[derive(Clone, Default)]
struct Counter {
    count: i32,
}
```

### Messages

Messages represent events that can change your component's state:

```rust
#[derive(Serialize, Deserialize)]
enum CounterMessage {
    Increment,
    Decrement,
    Reset,
}
```

## Creating Your First Component

Let's build a simple counter component:

```rust
use shipwright_liveview::{LiveView, Html, Updated, EventData};
use shipwright_liveview_macros::{html, LiveView};
use serde::{Serialize, Deserialize};

#[derive(LiveView, Clone, Default)]
struct Counter {
    count: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum CounterMessage {
    Increment,
    Decrement,
    Reset,
}

impl LiveView for Counter {
    type Message = CounterMessage;

    fn update(mut self, msg: Self::Message, _data: Option<EventData>) -> Updated<Self> {
        match msg {
            CounterMessage::Increment => self.count += 1,
            CounterMessage::Decrement => self.count -= 1,
            CounterMessage::Reset => self.count = 0,
        }
        Updated::new(self)
    }

    fn render(&self) -> Html<Self::Message> {
        html! {
            <div class="counter">
                <h1>{ self.count }</h1>
                <div class="buttons">
                    <button axm-click={ CounterMessage::Decrement }>"-"</button>
                    <button axm-click={ CounterMessage::Reset }>"Reset"</button>
                    <button axm-click={ CounterMessage::Increment }>"+"</button>
                </div>
            </div>
        }
    }
}
```

### Using the Component

```rust
use axum::response::IntoResponse;
use shipwright_liveview::LiveViewUpgrade;

async fn counter_page(live: LiveViewUpgrade) -> impl IntoResponse {
    let counter = Counter::default();
    
    live.response(move |embed| {
        html! {
            <!DOCTYPE html>
            <html>
                <head>
                    <title>"Counter"</title>
                </head>
                <body>
                    { embed.embed(counter) }
                    <script src="/assets/liveview.js"></script>
                </body>
            </html>
        }
    })
}
```

## Event Handling

LiveView supports various DOM events through special attributes:

### Click Events

```rust
html! {
    <button axm-click={ MyMessage::ButtonClicked }>"Click me"</button>
}
```

### Form Events

```rust
html! {
    <form axm-submit={ MyMessage::FormSubmitted }>
        <input 
            type="text" 
            value={ &self.input_value }
            axm-input={ MyMessage::InputChanged }
        />
        <button type="submit">"Submit"</button>
    </form>
}
```

### Keyboard Events

```rust
html! {
    <input 
        axm-keydown={ MyMessage::KeyPressed }
        axm-keyup={ MyMessage::KeyReleased }
    />
}
```

### Event Data

Access event data in your update function:

```rust
impl LiveView for MyComponent {
    type Message = MyMessage;
    
    fn update(mut self, msg: Self::Message, data: Option<EventData>) -> Updated<Self> {
        match msg {
            MyMessage::InputChanged => {
                if let Some(event_data) = data {
                    if let Some(value) = event_data.value() {
                        self.input_value = value.to_string();
                    }
                }
            }
            MyMessage::KeyPressed => {
                if let Some(event_data) = data {
                    if let Some(key) = event_data.key() {
                        println!("Key pressed: {}", key);
                    }
                }
            }
        }
        Updated::new(self)
    }
}
```

## State Management

### Component State

Each component manages its own state:

```rust
#[derive(Clone)]
struct TodoApp {
    todos: Vec<Todo>,
    input: String,
    filter: Filter,
}

#[derive(Clone)]
struct Todo {
    id: Uuid,
    text: String,
    completed: bool,
}

#[derive(Clone)]
enum Filter {
    All,
    Active,
    Completed,
}
```

### State Updates

The `update` method is where you modify state:

```rust
fn update(mut self, msg: Self::Message, data: Option<EventData>) -> Updated<Self> {
    match msg {
        TodoMessage::AddTodo => {
            if !self.input.trim().is_empty() {
                let todo = Todo {
                    id: Uuid::new_v4(),
                    text: self.input.trim().to_string(),
                    completed: false,
                };
                self.todos.push(todo);
                self.input.clear();
            }
        }
        TodoMessage::ToggleTodo(id) => {
            if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
                todo.completed = !todo.completed;
            }
        }
        TodoMessage::UpdateInput => {
            if let Some(data) = data {
                if let Some(value) = data.value() {
                    self.input = value.to_string();
                }
            }
        }
    }
    Updated::new(self)
}
```

### Conditional Rendering

Render different content based on state:

```rust
fn render(&self) -> Html<Self::Message> {
    let filtered_todos: Vec<&Todo> = match self.filter {
        Filter::All => self.todos.iter().collect(),
        Filter::Active => self.todos.iter().filter(|t| !t.completed).collect(),
        Filter::Completed => self.todos.iter().filter(|t| t.completed).collect(),
    };

    html! {
        <div class="todo-app">
            <h1>"Todo App"</h1>
            
            <div class="input-section">
                <input 
                    type="text"
                    value={ &self.input }
                    placeholder="What needs to be done?"
                    axm-input={ TodoMessage::UpdateInput }
                    axm-keydown={ TodoMessage::HandleKeydown }
                />
            </div>
            
            <div class="todos">
                { for filtered_todos.iter().map(|todo| self.render_todo(todo)) }
            </div>
            
            <div class="filters">
                { self.render_filter_button(Filter::All, "All") }
                { self.render_filter_button(Filter::Active, "Active") }
                { self.render_filter_button(Filter::Completed, "Completed") }
            </div>
        </div>
    }
}
```

## HTML Templates

### Basic Syntax

The `html!` macro provides a JSX-like syntax for writing HTML:

```rust
html! {
    <div class="container">
        <h1>"Hello, World!"</h1>
        <p>"This is a paragraph with " <strong>"bold text"</strong> "."</p>
    </div>
}
```

### Dynamic Content

Insert dynamic values using curly braces:

```rust
html! {
    <div>
        <h1>{ &self.title }</h1>
        <p>"Count: " { self.count }</p>
        <p>"Percentage: " { format!("{:.2}%", self.percentage) }</p>
    </div>
}
```

### Conditional Rendering

Use Rust's conditional expressions:

```rust
html! {
    <div>
        { if self.is_logged_in {
            html! { <p>"Welcome back!"</p> }
        } else {
            html! { <p>"Please log in."</p> }
        }}
        
        { match self.status {
            Status::Loading => html! { <div class="spinner"></div> },
            Status::Success => html! { <div class="success">"Success!"</div> },
            Status::Error(ref msg) => html! { <div class="error">{ msg }</div> },
        }}
    </div>
}
```

### Lists and Iteration

Render lists using iterators:

```rust
html! {
    <ul>
        { for self.items.iter().map(|item| {
            html! {
                <li key={ item.id }>
                    <span>{ &item.name }</span>
                    <button axm-click={ MyMessage::DeleteItem(item.id) }>
                        "Delete"
                    </button>
                </li>
            }
        })}
    </ul>
}
```

### Styling

Add CSS classes and inline styles:

```rust
html! {
    <div 
        class="card"
        class:active={ self.is_active }
        style="background-color: blue; color: white;"
    >
        <h2 class={ if self.is_important { "important" } else { "normal" } }>
            { &self.title }
        </h2>
    </div>
}
```

## Component Composition

### Child Components

Embed one component inside another:

```rust
#[derive(Clone)]
struct App {
    counter: Counter,
    todo_list: TodoList,
    current_tab: Tab,
}

impl LiveView for App {
    type Message = AppMessage;
    
    fn render(&self) -> Html<Self::Message> {
        html! {
            <div class="app">
                <nav>
                    <button 
                        axm-click={ AppMessage::SwitchTab(Tab::Counter) }
                        class:active={ matches!(self.current_tab, Tab::Counter) }
                    >
                        "Counter"
                    </button>
                    <button 
                        axm-click={ AppMessage::SwitchTab(Tab::Todos) }
                        class:active={ matches!(self.current_tab, Tab::Todos) }
                    >
                        "Todos"
                    </button>
                </nav>
                
                <main>
                    { match self.current_tab {
                        Tab::Counter => self.counter.render().map(AppMessage::CounterMessage),
                        Tab::Todos => self.todo_list.render().map(AppMessage::TodoMessage),
                    }}
                </main>
            </div>
        }
    }
}
```

### Message Mapping

Handle child component messages:

```rust
#[derive(Serialize, Deserialize)]
enum AppMessage {
    CounterMessage(CounterMessage),
    TodoMessage(TodoMessage),
    SwitchTab(Tab),
}

impl LiveView for App {
    type Message = AppMessage;
    
    fn update(mut self, msg: Self::Message, data: Option<EventData>) -> Updated<Self> {
        match msg {
            AppMessage::CounterMessage(counter_msg) => {
                self.counter = self.counter.update(counter_msg, data).view;
            }
            AppMessage::TodoMessage(todo_msg) => {
                self.todo_list = self.todo_list.update(todo_msg, data).view;
            }
            AppMessage::SwitchTab(tab) => {
                self.current_tab = tab;
            }
        }
        Updated::new(self)
    }
}
```

## Advanced Patterns

### Async Operations

Handle async operations with effects:

```rust
use shipwright_liveview::Updated;

impl LiveView for DataLoader {
    type Message = DataMessage;
    
    fn update(mut self, msg: Self::Message, _data: Option<EventData>) -> Updated<Self> {
        match msg {
            DataMessage::LoadData => {
                self.loading = true;
                
                Updated::new(self).with_effect(|tx| {
                    Box::pin(async move {
                        match load_data_from_api().await {
                            Ok(data) => {
                                let _ = tx.send(DataMessage::DataLoaded(data)).await;
                            }
                            Err(err) => {
                                let _ = tx.send(DataMessage::LoadError(err.to_string())).await;
                            }
                        }
                    })
                })
            }
            DataMessage::DataLoaded(data) => {
                self.loading = false;
                self.data = Some(data);
                Updated::new(self)
            }
            DataMessage::LoadError(error) => {
                self.loading = false;
                self.error = Some(error);
                Updated::new(self)
            }
        }
    }
}
```

### Form Handling

Build complex forms with validation:

```rust
#[derive(Clone)]
struct UserForm {
    name: String,
    email: String,
    age: String,
    errors: HashMap<String, String>,
    submitting: bool,
}

impl UserForm {
    fn validate(&self) -> HashMap<String, String> {
        let mut errors = HashMap::new();
        
        if self.name.trim().is_empty() {
            errors.insert("name".to_string(), "Name is required".to_string());
        }
        
        if self.email.trim().is_empty() {
            errors.insert("email".to_string(), "Email is required".to_string());
        } else if !self.email.contains('@') {
            errors.insert("email".to_string(), "Invalid email format".to_string());
        }
        
        if let Ok(age) = self.age.parse::<u8>() {
            if age < 13 {
                errors.insert("age".to_string(), "Must be at least 13 years old".to_string());
            }
        } else if !self.age.is_empty() {
            errors.insert("age".to_string(), "Age must be a number".to_string());
        }
        
        errors
    }
}

impl LiveView for UserForm {
    type Message = FormMessage;
    
    fn update(mut self, msg: Self::Message, data: Option<EventData>) -> Updated<Self> {
        match msg {
            FormMessage::UpdateName => {
                if let Some(data) = data {
                    if let Some(value) = data.value() {
                        self.name = value.to_string();
                        self.errors.remove("name");
                    }
                }
            }
            FormMessage::Submit => {
                self.errors = self.validate();
                if self.errors.is_empty() {
                    self.submitting = true;
                    // Handle form submission
                }
            }
        }
        Updated::new(self)
    }
    
    fn render(&self) -> Html<Self::Message> {
        html! {
            <form axm-submit={ FormMessage::Submit }>
                <div class="field">
                    <label for="name">"Name"</label>
                    <input 
                        type="text"
                        id="name"
                        value={ &self.name }
                        axm-input={ FormMessage::UpdateName }
                        class:error={ self.errors.contains_key("name") }
                    />
                    { if let Some(error) = self.errors.get("name") {
                        html! { <div class="error-message">{ error }</div> }
                    } else {
                        html! { <></> }
                    }}
                </div>
                
                <button type="submit" disabled={ self.submitting }>
                    { if self.submitting { "Submitting..." } else { "Submit" } }
                </button>
            </form>
        }
    }
}
```

## Best Practices

### 1. Keep Components Small and Focused

```rust
// Good: Focused component
#[derive(Clone)]
struct SearchBox {
    query: String,
    suggestions: Vec<String>,
}

// Better: Break down complex components
#[derive(Clone)]
struct Dashboard {
    search: SearchBox,
    results: ResultsList,
    filters: FilterPanel,
}
```

### 2. Use Clear Message Names

```rust
// Good: Descriptive message names
enum UserMessage {
    UpdateUsername(String),
    SubmitRegistration,
    ToggleProfileVisibility,
    DeleteAccount,
}
```

### 3. Handle All Possible States

```rust
#[derive(Clone)]
enum LoadingState<T> {
    NotStarted,
    Loading,
    Loaded(T),
    Error(String),
}

fn render_data<T>(&self, state: &LoadingState<T>) -> Html<Self::Message> 
where 
    T: Display 
{
    match state {
        LoadingState::NotStarted => html! { <div>"Click to load"</div> },
        LoadingState::Loading => html! { <div class="spinner">"Loading..."</div> },
        LoadingState::Loaded(data) => html! { <div>{ data }</div> },
        LoadingState::Error(err) => html! { <div class="error">{ err }</div> },
    }
}
```

### 4. Use Type-Safe IDs

```rust
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct TodoId(Uuid);

#[derive(Clone)]
struct Todo {
    id: TodoId,
    text: String,
    completed: bool,
}

enum TodoMessage {
    Toggle(TodoId),
    Delete(TodoId),
    Edit(TodoId, String),
}
```

### 5. Organize CSS with Components

```rust
impl MyComponent {
    fn render(&self) -> Html<Self::Message> {
        html! {
            <div class="my-component">
                { self.render_header() }
                { self.render_content() }
                { self.render_footer() }
                
                <style>
                    ".my-component {
                        display: flex;
                        flex-direction: column;
                        gap: 16px;
                        padding: 20px;
                        border: 1px solid #ddd;
                        border-radius: 8px;
                    }
                    
                    .my-component .header {
                        font-size: 1.2em;
                        font-weight: bold;
                    }"
                </style>
            </div>
        }
    }
}
```

## Examples

Check out these complete examples in the hello-world-local codebase:

- [Counter](../hello-world-local-liveview/src/pages/counter.rs) - Basic state management
- [Chat](../hello-world-local-liveview/src/pages/chat.rs) - Real-time messaging
- [Components Demo](../hello-world-local-liveview/src/pages/components.rs) - UI components showcase
- [Form Example](../examples/form/) - Complex form handling

## Next Steps

- Read the [API Reference](./api-reference.md) for detailed documentation
- Explore the [Architecture Guide](./architecture.md) to understand the bigger picture
- Check out [Advanced Patterns](./advanced-patterns.md) for complex use cases

Happy building with LiveView! 🚀