// Compile-time tests for the view! macro

use shipwright_liveview_macros::view;

#[test]
fn test_basic_html() {
    let _ = view! {
        <div class="test">
            <p>{"Hello"}</p>
        </div>
    };
}

#[test]
fn test_dynamic_content() {
    let value = 42;
    let _ = view! {
        <div>
            <span>{value}</span>
            <p>{"Count: "}{value + 1}</p>
        </div>
    };
}

#[test]
fn test_event_handlers() {
    let handler = || println!("clicked");
    let _ = view! {
        <button on:click={handler}>{"Click me"}</button>
    };
}

#[test]
fn test_conditionals() {
    let show = true;
    let _ = view! {
        <div>
            {if show {
                <p>{"Visible"}</p>
            } else {
                <p>{"Hidden"}</p>
            }}
        </div>
    };
}

#[test]
fn test_loops() {
    let items = vec!["a", "b", "c"];
    let _ = view! {
        <ul>
            {for item in items {
                <li>{item}</li>
            }}
        </ul>
    };
}

#[test]
fn test_dynamic_attributes() {
    let active = true;
    let value = "test";
    let _ = view! {
        <div class:active={active}>
            <input value={value} />
        </div>
    };
}

#[test]
fn test_self_closing_tags() {
    let _ = view! {
        <div>
            <br />
            <img src="test.png" alt="test" />
            <input type="text" />
        </div>
    };
}

#[test]
fn test_nested_structures() {
    let items = vec![1, 2, 3];
    let show_list = true;
    
    let _ = view! {
        <div class="container">
            <header>
                <h1>{"My App"}</h1>
            </header>
            {if show_list {
                <ul>
                    {for item in items {
                        <li>
                            <span>{"Item: "}</span>
                            <strong>{item}</strong>
                        </li>
                    }}
                </ul>
            }}
        </div>
    };
}

#[cfg(test)]
mod compile_fail_tests {
    // These tests would use trybuild to test compile failures
    // For now, they're documented here
    
    // Should fail: Mismatched closing tag
    // view! {
    //     <div>
    //         <span>{"test"}</div>
    //     </span>
    // }
    
    // Should fail: Invalid attribute syntax  
    // view! {
    //     <div class=invalid>{"test"}</div>
    // }
    
    // Should fail: Unclosed tag
    // view! {
    //     <div>
    //         <p>{"test"}
    //     </div>
    // }
}