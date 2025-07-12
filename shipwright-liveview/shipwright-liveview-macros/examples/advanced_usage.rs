// Advanced example showing complex view! macro usage patterns

use shipwright_liveview_macros::view;

// Example data structures
#[derive(Clone)]
struct Todo {
    id: usize,
    text: String,
    completed: bool,
}

struct TodoApp {
    todos: Vec<Todo>,
    filter: Filter,
    new_todo_text: String,
}

#[derive(PartialEq)]
enum Filter {
    All,
    Active,
    Completed,
}

// Example components
struct TodoItem {
    todo: Todo,
    on_toggle: Box<dyn Fn(usize)>,
    on_delete: Box<dyn Fn(usize)>,
}

struct FilterButton {
    filter: Filter,
    current_filter: Filter,
    on_click: Box<dyn Fn(Filter)>,
}

// Event handlers
fn add_todo(app: &mut TodoApp) {
    if !app.new_todo_text.is_empty() {
        app.todos.push(Todo {
            id: app.todos.len(),
            text: app.new_todo_text.clone(),
            completed: false,
        });
        app.new_todo_text.clear();
    }
}

fn toggle_todo(app: &mut TodoApp, id: usize) {
    if let Some(todo) = app.todos.iter_mut().find(|t| t.id == id) {
        todo.completed = !todo.completed;
    }
}

fn delete_todo(app: &mut TodoApp, id: usize) {
    app.todos.retain(|t| t.id != id);
}

fn set_filter(app: &mut TodoApp, filter: Filter) {
    app.filter = filter;
}

fn clear_completed(app: &mut TodoApp) {
    app.todos.retain(|t| !t.completed);
}

fn main() {
    let app = TodoApp {
        todos: vec![
            Todo { id: 0, text: "Learn Rust".to_string(), completed: true },
            Todo { id: 1, text: "Build LiveView".to_string(), completed: false },
            Todo { id: 2, text: "Ship it!".to_string(), completed: false },
        ],
        filter: Filter::All,
        new_todo_text: String::new(),
    };

    // Complex todo application view
    let todo_app_view = view! {
        <div class="todoapp">
            <header class="header">
                <h1>{"todos"}</h1>
                <input 
                    class="new-todo"
                    placeholder="What needs to be done?"
                    value={app.new_todo_text.clone()}
                    on:change={|e| app.new_todo_text = e.target.value}
                    on:keydown={|e| if e.key == "Enter" { add_todo(&mut app) }}
                />
            </header>

            {if !app.todos.is_empty() {
                <section class="main">
                    <input 
                        id="toggle-all"
                        class="toggle-all"
                        type="checkbox"
                        checked={app.todos.iter().all(|t| t.completed)}
                        on:change={|_| {
                            let all_completed = app.todos.iter().all(|t| t.completed);
                            for todo in &mut app.todos {
                                todo.completed = !all_completed;
                            }
                        }}
                    />
                    <label for="toggle-all">{"Mark all as complete"}</label>

                    <ul class="todo-list">
                        {for todo in filtered_todos(&app.todos, &app.filter) {
                            <TodoItem 
                                todo={todo.clone()}
                                on_toggle={Box::new(move |id| toggle_todo(&mut app, id))}
                                on_delete={Box::new(move |id| delete_todo(&mut app, id))}
                            />
                        }}
                    </ul>
                </section>

                <footer class="footer">
                    <span class="todo-count">
                        <strong>{active_count(&app.todos)}</strong>
                        {if active_count(&app.todos) == 1 {
                            " item left"
                        } else {
                            " items left"
                        }}
                    </span>

                    <ul class="filters">
                        <li>
                            <FilterButton 
                                filter={Filter::All}
                                current_filter={app.filter}
                                on_click={Box::new(|f| set_filter(&mut app, f))}
                            />
                        </li>
                        <li>
                            <FilterButton 
                                filter={Filter::Active}
                                current_filter={app.filter}
                                on_click={Box::new(|f| set_filter(&mut app, f))}
                            />
                        </li>
                        <li>
                            <FilterButton 
                                filter={Filter::Completed}
                                current_filter={app.filter}
                                on_click={Box::new(|f| set_filter(&mut app, f))}
                            />
                        </li>
                    </ul>

                    {if completed_count(&app.todos) > 0 {
                        <button 
                            class="clear-completed"
                            on:click={|_| clear_completed(&mut app)}
                        >
                            {"Clear completed"}
                        </button>
                    }}
                </footer>
            }}
        </div>
    };

    // TodoItem component implementation
    let todo_item_view = |todo: &Todo, on_toggle: &dyn Fn(usize), on_delete: &dyn Fn(usize)| {
        view! {
            <li class:completed={todo.completed}>
                <div class="view">
                    <input 
                        class="toggle"
                        type="checkbox"
                        checked={todo.completed}
                        on:change={move |_| on_toggle(todo.id)}
                    />
                    <label>{&todo.text}</label>
                    <button 
                        class="destroy"
                        on:click={move |_| on_delete(todo.id)}
                    />
                </div>
            </li>
        }
    };

    // FilterButton component implementation
    let filter_button_view = |filter: &Filter, current: &Filter, label: &str, on_click: &dyn Fn(Filter)| {
        view! {
            <a 
                href="#"
                class:selected={filter == current}
                on:click={move |e| {
                    e.prevent_default();
                    on_click(filter.clone());
                }}
            >
                {label}
            </a>
        }
    };

    // Example with nested conditionals and loops
    let complex_nested_view = view! {
        <div class="dashboard">
            {for (section_idx, section) in app.sections.iter().enumerate() {
                <div class="section" class:expanded={section.expanded}>
                    <h2 on:click={move |_| toggle_section(section_idx)}>
                        {&section.title}
                        <span class="icon">
                            {if section.expanded { "▼" } else { "▶" }}
                        </span>
                    </h2>
                    
                    {if section.expanded {
                        <div class="content">
                            {if section.items.is_empty() {
                                <p class="empty">{"No items in this section"}</p>
                            } else {
                                <ul>
                                    {for item in &section.items {
                                        <li class:highlighted={item.important}>
                                            {if item.important {
                                                <strong>{&item.text}</strong>
                                            } else {
                                                <span>{&item.text}</span>
                                            }}
                                        </li>
                                    }}
                                </ul>
                            }}
                        </div>
                    }}
                </div>
            }}
        </div>
    };
}

// Helper functions
fn filtered_todos(todos: &[Todo], filter: &Filter) -> Vec<&Todo> {
    todos.iter().filter(|todo| {
        match filter {
            Filter::All => true,
            Filter::Active => !todo.completed,
            Filter::Completed => todo.completed,
        }
    }).collect()
}

fn active_count(todos: &[Todo]) -> usize {
    todos.iter().filter(|t| !t.completed).count()
}

fn completed_count(todos: &[Todo]) -> usize {
    todos.iter().filter(|t| t.completed).count()
}