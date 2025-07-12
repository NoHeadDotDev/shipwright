// Compile-time tests for the html! macro

use shipwright_liveview_macros::html;

#[test]
fn test_basic_html() {
    let _ = html! {
        <div class="test">
            <p>{"Hello"}</p>
        </div>
    };
}

#[test]
fn test_dynamic_content() {
    let value = 42;
    let _ = html! {
        <div>
            <span>{value}</span>
            <p>{"Count: "}{value + 1}</p>
        </div>
    };
}

#[test]
fn test_event_handlers() {
    let handler = || println!("clicked");
    let _ = html! {
        <button on:click={handler}>{"Click me"}</button>
    };
}

#[test]
fn test_conditionals() {
    let show = true;
    let _ = html! {
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
    let _ = html! {
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
    let _ = html! {
        <div class:active={active}>
            <input value={value} />
        </div>
    };
}

#[test]
fn test_self_closing_tags() {
    let _ = html! {
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
    
    let _ = html! {
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

#[test]
fn test_void_elements_self_closing() {
    // Void elements should work with self-closing syntax
    let _ = html! {
        <div>
            <br />
            <hr />
            <img src="test.png" alt="test" />
            <input type="text" />
            <meta charset="utf-8" />
            <link rel="stylesheet" href="style.css" />
        </div>
    };
}

#[test]
fn test_valid_element_nesting() {
    // Test valid nesting patterns
    let _ = html! {
        <div>
            <p><span>{"text"}</span></p>
            <ul>
                <li>{"item 1"}</li>
                <li>{"item 2"}</li>
            </ul>
            <table>
                <tr>
                    <td>{"cell"}</td>
                </tr>
            </table>
        </div>
    };
}

#[test]
fn test_form_elements() {
    // Test form-related elements
    let _ = html! {
        <form>
            <label>{"Name: "}<input type="text" /></label>
            <button type="submit">{"Submit"}</button>
        </form>
    };
}

#[test]
fn test_sectioning_content() {
    // Test sectioning elements
    let _ = html! {
        <article>
            <header>
                <h1>{"Article Title"}</h1>
            </header>
            <section>
                <h2>{"Section Title"}</h2>
                <p>{"Content"}</p>
            </section>
            <aside>
                <p>{"Sidebar content"}</p>
            </aside>
        </article>
    };
}

#[cfg(test)]
mod compile_fail_tests {
    // These tests would use trybuild to test compile failures
    // For now, they're documented here
    
    // Should fail: Mismatched closing tag
    // html! {
    //     <div>
    //         <span>{"test"}</div>
    //     </span>
    // }
    
    // Should fail: Invalid attribute syntax  
    // html! {
    //     <div class=invalid>{"test"}</div>
    // }
    
    // Should fail: Unclosed tag
    // html! {
    //     <div>
    //         <p>{"test"}
    //     </div>
    // }
    
    // Should fail: Void element with children
    // html! {
    //     <br>{"text"}</br>
    // }
    
    // Should fail: Non-void element with self-closing syntax
    // html! {
    //     <div />
    // }
    
    // Should fail: Invalid nesting (interactive in interactive)
    // html! {
    //     <button>
    //         <a href="#">{"link"}</a>
    //     </button>
    // }
    
    // Should fail: Invalid nesting (p containing div)
    // html! {
    //     <p>
    //         <div>{"block content"}</div>
    //     </p>
    // }
    
    // Should fail: Invalid nesting (ul containing div)
    // html! {
    //     <ul>
    //         <div>{"not an li"}</div>
    //     </ul>
    // }
    
    // Should fail: Invalid nesting (nested forms)
    // html! {
    //     <form>
    //         <form>
    //             <input type="text" />
    //         </form>
    //     </form>
    // }
}