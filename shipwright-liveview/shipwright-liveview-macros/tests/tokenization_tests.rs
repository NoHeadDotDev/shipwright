//! Comprehensive test suite for HTML tokenization edge cases
//! Tests the tokenizer module for various HTML validation scenarios

use shipwright_liveview_macros::html;

#[test]
fn test_valid_html_elements() {
    // Test basic valid elements
    let _ = html! {
        <div></div>
    };
    
    let _ = html! {
        <p>Content</p>
    };
    
    let _ = html! {
        <span class="test">Text</span>
    };
}

#[test]
fn test_void_elements() {
    // Test valid void elements
    let _ = html! {
        <br />
    };
    
    let _ = html! {
        <img src="test.png" alt="test" />
    };
    
    let _ = html! {
        <input type="text" />
    };
    
    let _ = html! {
        <meta charset="utf-8" />
    };
    
    let _ = html! {
        <link rel="stylesheet" href="style.css" />
    };
}

#[test]
fn test_boolean_attributes() {
    // Test boolean attributes
    let _ = html! {
        <input checked />
    };
    
    let _ = html! {
        <input disabled />
    };
    
    let _ = html! {
        <select multiple>
            <option>Test</option>
        </select>
    };
    
    let _ = html! {
        <video controls autoplay muted>
            <source src="movie.mp4" type="video/mp4" />
        </video>
    };
}

#[test]
fn test_data_attributes() {
    // Test data attributes with valid naming
    let _ = html! {
        <div data-value="test"></div>
    };
    
    let _ = html! {
        <span data-toggle="modal" data-target="#myModal">Click</span>
    };
    
    let _ = html! {
        <button data-action="submit" data-form-id="contact">Submit</button>
    };
}

#[test]
fn test_custom_elements() {
    // Test custom elements with hyphens
    let _ = html! {
        <my-component></my-component>
    };
    
    let _ = html! {
        <custom-button type="primary">Click me</custom-button>
    };
}

#[test]
fn test_nested_structures() {
    // Test complex nested structures
    let _ = html! {
        <div class="container">
            <header>
                <h1>Title</h1>
                <nav>
                    <ul>
                        <li><a href="#home">Home</a></li>
                        <li><a href="#about">About</a></li>
                    </ul>
                </nav>
            </header>
            <main>
                <section>
                    <article>
                        <h2>Article Title</h2>
                        <p>Article content with <strong>emphasis</strong> and <em>italics</em>.</p>
                    </article>
                </section>
            </main>
            <footer>
                <p>Copyright 2023</p>
            </footer>
        </div>
    };
}

#[test]
fn test_html_entities() {
    // Test valid HTML entities
    let _ = html! {
        <p>"&amp; &lt; &gt; &quot; &apos;"</p>
    };
    
    let _ = html! {
        <p>"&nbsp; &copy; &reg; &trade;"</p>
    };
    
    let _ = html! {
        <p>"&hellip; &mdash; &ndash;"</p>
    };
}

#[test]
fn test_numeric_entities() {
    // Test numeric character references
    let _ = html! {
        <p>"&#65; &#66; &#67;"</p>
    };
    
    let _ = html! {
        <p>"&#x41; &#x42; &#x43;"</p>
    };
    
    let _ = html! {
        <p>"&#8364; &#x20AC;"</p>  // Euro symbol
    };
}

#[test]
fn test_attribute_variations() {
    // Test various attribute patterns
    let _ = html! {
        <div id="unique-id" class="class1 class2" data-value="123">
            <input type="email" name="email" required />
            <textarea rows="5" cols="40" placeholder="Enter text"></textarea>
        </div>
    };
}

#[test]
fn test_mixed_content() {
    // Test elements with mixed text and child elements
    let _ = html! {
        <p>
            "This is some text with "
            <strong>"bold"</strong>
            " and "
            <em>"italic"</em>
            " formatting."
        </p>
    };
}

#[test]
fn test_semantic_html() {
    // Test semantic HTML5 elements
    let _ = html! {
        <article>
            <header>
                <h1>"Article Title"</h1>
                <time datetime="2023-01-01">"January 1, 2023"</time>
            </header>
            <section>
                <p>"Article content goes here."</p>
                <aside>
                    <p>"This is a sidebar note."</p>
                </aside>
            </section>
            <footer>
                <p>"Article footer"</p>
            </footer>
        </article>
    };
}

#[test]
fn test_form_elements() {
    // Test comprehensive form structure
    let _ = html! {
        <form action="/submit" method="post">
            <fieldset>
                <legend>"Personal Information"</legend>
                <div>
                    <label for="name">"Name:"</label>
                    <input type="text" id="name" name="name" required />
                </div>
                <div>
                    <label for="email">"Email:"</label>
                    <input type="email" id="email" name="email" required />
                </div>
                <div>
                    <label for="country">"Country:"</label>
                    <select id="country" name="country">
                        <option value="">"Select a country"</option>
                        <option value="us">"United States"</option>
                        <option value="ca">"Canada"</option>
                    </select>
                </div>
                <div>
                    <label>"
                        <input type="checkbox" name="newsletter" />
                        " Subscribe to newsletter"
                    </label>
                </div>
            </fieldset>
            <button type="submit">"Submit"</button>
        </form>
    };
}

#[test]
fn test_table_structure() {
    // Test table elements
    let _ = html! {
        <table>
            <caption>"Sales Report"</caption>
            <thead>
                <tr>
                    <th scope="col">"Product"</th>
                    <th scope="col">"Sales"</th>
                    <th scope="col">"Revenue"</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td>"Product A"</td>
                    <td>"100"</td>
                    <td>"$1,000"</td>
                </tr>
                <tr>
                    <td>"Product B"</td>
                    <td>"150"</td>
                    <td>"$2,250"</td>
                </tr>
            </tbody>
            <tfoot>
                <tr>
                    <td>"Total"</td>
                    <td>"250"</td>
                    <td>"$3,250"</td>
                </tr>
            </tfoot>
        </table>
    };
}

#[test]
fn test_media_elements() {
    // Test media elements
    let _ = html! {
        <div>
            <img src="image.jpg" alt="Description" width="300" height="200" />
            <figure>
                <img src="chart.png" alt="Sales Chart" />
                <figcaption>"Monthly sales chart"</figcaption>
            </figure>
            <video controls width="640" height="360">
                <source src="movie.mp4" type="video/mp4" />
                <source src="movie.ogg" type="video/ogg" />
                "Your browser does not support the video tag."
            </video>
            <audio controls>
                <source src="audio.mp3" type="audio/mpeg" />
                <source src="audio.ogg" type="audio/ogg" />
                "Your browser does not support the audio element."
            </audio>
        </div>
    };
}

// Test module for edge cases and error conditions
// These would be tested with trybuild for compile-time failures
#[cfg(test)]
mod edge_case_documentation {
    // The following would cause compile errors with our validation:
    
    // Invalid element names:
    // html! { <123invalid></123invalid> }
    // html! { <div@></div@> }
    // html! { <></> }
    
    // Invalid attributes:
    // html! { <div class name="test"></div> }  // space in attribute name
    // html! { <div class"="test"></div> }      // quote in attribute name
    // html! { <div =value></div> }             // missing attribute name
    
    // Void elements with children:
    // html! { <br>Content</br> }
    // html! { <img><span>Invalid</span></img> }
    // html! { <input>Text</input> }
    
    // Mismatched tags:
    // html! { <div><span></div></span> }
    // html! { <p><h1></p></h1> }
    
    // Unclosed tags:
    // html! { <div><p>Content</div> }
    // html! { <ul><li>Item</ul> }
    
    // Invalid boolean attributes:
    // html! { <input checked="true" /> }       // boolean attr with wrong value
    // html! { <select multiple="false"></select> }
    
    // Invalid data attributes:
    // html! { <div data-="value"></div> }      // empty data attribute name
    // html! { <div data-Value="test"></div> }  // uppercase in data attribute
    
    // Invalid entities:
    // html! { <p>"&invalidEntity;"</p> }
    // html! { <p>"&amp"</p> }                 // missing semicolon
    // html! { <p>"&#xyz;"</p> }                // invalid numeric entity
    
    // Invalid nesting:
    // html! { <p><div>Block in inline</div></p> }
    // html! { <button><button>Nested button</button></button> }
}

/// Integration tests that combine multiple validation features
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_document_structure() {
        let _ = html! {
            <html lang="en">
                <head>
                    <meta charset="UTF-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                    <title>"Test Document"</title>
                    <link rel="stylesheet" href="styles.css" />
                </head>
                <body>
                    <header role="banner">
                        <h1>"Welcome to Our Site"</h1>
                        <nav role="navigation">
                            <ul>
                                <li><a href="#section1">"Section 1"</a></li>
                                <li><a href="#section2">"Section 2"</a></li>
                            </ul>
                        </nav>
                    </header>
                    
                    <main role="main">
                        <section id="section1">
                            <h2>"First Section"</h2>
                            <p>"This section contains important information."</p>
                            <img src="diagram.png" alt="Important diagram" />
                        </section>
                        
                        <section id="section2">
                            <h2>"Second Section"</h2>
                            <form action="/contact" method="post">
                                <div>
                                    <label for="message">"Message:"</label>
                                    <textarea id="message" name="message" required></textarea>
                                </div>
                                <button type="submit">"Send Message"</button>
                            </form>
                        </section>
                    </main>
                    
                    <footer role="contentinfo">
                        <p>"&copy; 2023 Our Company. All rights reserved."</p>
                    </footer>
                </body>
            </html>
        };
    }

    #[test]
    fn test_complex_interactive_elements() {
        let _ = html! {
            <div class="app">
                <div class="modal" data-modal="true" role="dialog" aria-labelledby="modal-title">
                    <div class="modal-content">
                        <header class="modal-header">
                            <h2 id="modal-title">"Confirm Action"</h2>
                            <button type="button" class="close" aria-label="Close">
                                <span aria-hidden="true">"&times;"</span>
                            </button>
                        </header>
                        <div class="modal-body">
                            <p>"Are you sure you want to proceed?"</p>
                        </div>
                        <footer class="modal-footer">
                            <button type="button" class="btn btn-secondary">"Cancel"</button>
                            <button type="button" class="btn btn-primary">"Confirm"</button>
                        </footer>
                    </div>
                </div>
            </div>
        };
    }

    #[test]
    fn test_accessibility_features() {
        let _ = html! {
            <div>
                <nav aria-label="Main navigation">
                    <ul role="menubar">
                        <li role="none">
                            <a href="#" role="menuitem">"Home"</a>
                        </li>
                        <li role="none">
                            <a href="#" role="menuitem" aria-haspopup="true">"Products"</a>
                        </li>
                    </ul>
                </nav>
                
                <main>
                    <h1>"Product Catalog"</h1>
                    <div role="region" aria-labelledby="filter-heading">
                        <h2 id="filter-heading">"Filters"</h2>
                        <form>
                            <fieldset>
                                <legend>"Price Range"</legend>
                                <label>
                                    <input type="radio" name="price" value="low" />
                                    " Under $50"
                                </label>
                                <label>
                                    <input type="radio" name="price" value="medium" />
                                    " $50 - $100"
                                </label>
                                <label>
                                    <input type="radio" name="price" value="high" />
                                    " Over $100"
                                </label>
                            </fieldset>
                        </form>
                    </div>
                </main>
            </div>
        };
    }
}