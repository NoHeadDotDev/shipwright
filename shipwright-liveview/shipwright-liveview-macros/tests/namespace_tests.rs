//! Comprehensive namespace and entity test suite for SVG and MathML support

use shipwright_liveview_macros::html;

#[test]
fn test_basic_svg_elements() {
    let _ = html! {
        <svg width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="blue" />
            <circle cx="50" cy="50" r="30" fill="red" />
            <line x1="0" y1="0" x2="100" y2="100" stroke="black" />
        </svg>
    };
}

#[test]
fn test_basic_mathml_elements() {
    let _ = html! {
        <math>
            <mrow>
                <mi>x</mi>
                <mo>=</mo>
                <mfrac>
                    <mi>a</mi>
                    <mi>b</mi>
                </mfrac>
            </mrow>
        </math>
    };
}

#[test]
fn test_nested_svg_elements() {
    let _ = html! {
        <svg width="200" height="200">
            <g transform="translate(10,10)">
                <rect width="50" height="50" fill="blue" />
                <text x="25" y="25" text-anchor="middle">
                    <tspan>Hello</tspan>
                </text>
            </g>
            <defs>
                <linearGradient id="grad1">
                    <stop offset="0%" stop-color="red" />
                    <stop offset="100%" stop-color="blue" />
                </linearGradient>
            </defs>
        </svg>
    };
}

#[test]
fn test_complex_mathml_expressions() {
    let _ = html! {
        <math>
            <mrow>
                <msup>
                    <mi>x</mi>
                    <mn>2</mn>
                </msup>
                <mo>+</mo>
                <msup>
                    <mi>y</mi>
                    <mn>2</mn>
                </msup>
                <mo>=</mo>
                <msup>
                    <mi>z</mi>
                    <mn>2</mn>
                </msup>
            </mrow>
        </math>
    };
}

#[test]
fn test_svg_with_animations() {
    let _ = html! {
        <svg width="100" height="100">
            <circle cx="50" cy="50" r="20" fill="red">
                <animate attributeName="r" from="20" to="40" dur="1s" repeatCount="indefinite" />
                <animateTransform attributeName="transform" type="rotate" 
                    from="0 50 50" to="360 50 50" dur="2s" repeatCount="indefinite" />
            </circle>
        </svg>
    };
}

#[test]
fn test_mathml_with_tables() {
    let _ = html! {
        <math>
            <mtable>
                <mtr>
                    <mtd><mi>a</mi></mtd>
                    <mtd><mi>b</mi></mtd>
                </mtr>
                <mtr>
                    <mtd><mi>c</mi></mtd>
                    <mtd><mi>d</mi></mtd>
                </mtr>
            </mtable>
        </math>
    };
}

#[test]
fn test_svg_filters_and_effects() {
    let _ = html! {
        <svg width="200" height="200">
            <defs>
                <filter id="blur">
                    <feGaussianBlur stdDeviation="3" />
                </filter>
                <filter id="shadow">
                    <feOffset dx="2" dy="2" />
                    <feColorMatrix type="matrix" values="0 0 0 0.5 0 0 0 0 0.5 0 0 0 0 0.5 0 0 0 0 1 0" />
                </filter>
            </defs>
            <rect x="10" y="10" width="100" height="100" fill="blue" filter="url(#blur)" />
            <text x="50" y="150" text-anchor="middle" filter="url(#shadow)">{"Filtered Text"}</text>
        </svg>
    };
}

#[test]
fn test_mathml_advanced_notation() {
    let _ = html! {
        <math>
            <mrow>
                <msubsup>
                    <mo>∫</mo>
                    <mn>0</mn>
                    <mi>∞</mi>
                </msubsup>
                <msup>
                    <mi>e</mi>
                    <mrow>
                        <mo>-</mo>
                        <msup>
                            <mi>x</mi>
                            <mn>2</mn>
                        </msup>
                    </mrow>
                </msup>
                <mo>d</mo>
                <mi>x</mi>
                <mo>=</mo>
                <mfrac>
                    <msqrt>
                        <mi>π</mi>
                    </msqrt>
                    <mn>2</mn>
                </mfrac>
            </mrow>
        </math>
    };
}

#[test]
fn test_svg_paths_and_shapes() {
    let _ = html! {
        <svg width="300" height="200">
            <path d="M 10 10 L 100 10 L 100 100 Z" fill="green" />
            <polygon points="150,10 180,50 120,50" fill="orange" />
            <polyline points="200,10 250,50 200,100 250,150" stroke="purple" fill="none" />
            <ellipse cx="250" cy="100" rx="30" ry="20" fill="pink" />
        </svg>
    };
}

#[test]
fn test_mathml_semantic_annotations() {
    let _ = html! {
        <math>
            <semantics>
                <mrow>
                    <msup>
                        <mi>x</mi>
                        <mn>2</mn>
                    </msup>
                    <mo>+</mo>
                    <msup>
                        <mi>y</mi>
                        <mn>2</mn>
                    </msup>
                </mrow>
                <annotation encoding="application/x-tex">x^2 + y^2</annotation>
                <annotation-xml encoding="MathML-Content">
                    <apply>
                        <plus />
                        <apply>
                            <power />
                            <ci>x</ci>
                            <cn>2</cn>
                        </apply>
                        <apply>
                            <power />
                            <ci>y</ci>
                            <cn>2</cn>
                        </apply>
                    </apply>
                </annotation-xml>
            </semantics>
        </math>
    };
}

#[test]
fn test_mixed_html_svg_content() {
    let _ = html! {
        <div class="container">
            <h1>{"SVG Example"}</h1>
            <svg width="100" height="100">
                <rect x="10" y="10" width="80" height="80" fill="blue" />
                <foreignObject x="20" y="20" width="60" height="60">
                    <div style="background: white; padding: 5px;">
                        <p>{"HTML inside SVG"}</p>
                    </div>
                </foreignObject>
            </svg>
            <p>{"Regular HTML content"}</p>
        </div>
    };
}

#[test]
fn test_mixed_html_mathml_content() {
    let _ = html! {
        <article>
            <h2>{"Mathematical Formula"}</h2>
            <p>{"The quadratic formula is:"}</p>
            <math display="block">
                <mrow>
                    <mi>x</mi>
                    <mo>=</mo>
                    <mfrac>
                        <mrow>
                            <mo>-</mo>
                            <mi>b</mi>
                            <mo>±</mo>
                            <msqrt>
                                <mrow>
                                    <msup>
                                        <mi>b</mi>
                                        <mn>2</mn>
                                    </msup>
                                    <mo>-</mo>
                                    <mn>4</mn>
                                    <mi>a</mi>
                                    <mi>c</mi>
                                </mrow>
                            </msqrt>
                        </mrow>
                        <mrow>
                            <mn>2</mn>
                            <mi>a</mi>
                        </mrow>
                    </mfrac>
                </mrow>
            </math>
            <p>{"where a, b, and c are coefficients."}</p>
        </article>
    };
}

#[test]
fn test_dynamic_svg_content() {
    let width = 150;
    let height = 150;
    let color = "purple";
    
    let _ = html! {
        <svg width={width} height={height}>
            <rect x="10" y="10" width={width - 20} height={height - 20} fill={color} />
            <text x={width / 2} y={height / 2} text-anchor="middle" fill="white">
                {"Dynamic SVG"}
            </text>
        </svg>
    };
}

#[test]
fn test_dynamic_mathml_content() {
    let variable = "x";
    let exponent = 3;
    let coefficient = 2;
    
    let _ = html! {
        <math>
            <mrow>
                <mi>y</mi>
                <mo>=</mo>
                <mn>{coefficient}</mn>
                <msup>
                    <mi>{variable}</mi>
                    <mn>{exponent}</mn>
                </msup>
            </mrow>
        </math>
    };
}

#[test]
fn test_svg_with_conditional_content() {
    let show_circle = true;
    let show_text = false;
    
    let _ = html! {
        <svg width="100" height="100">
            <rect x="0" y="0" width="100" height="100" fill="lightgray" />
            {if show_circle {
                <circle cx="50" cy="50" r="20" fill="blue" />
            }}
            {if show_text {
                <text x="50" y="80" text-anchor="middle">{"Optional Text"}</text>
            }}
        </svg>
    };
}

#[test]
fn test_mathml_with_conditional_content() {
    let show_fraction = true;
    let show_exponent = false;
    
    let _ = html! {
        <math>
            <mrow>
                <mi>f</mi>
                <mo>(</mo>
                <mi>x</mi>
                <mo>)</mo>
                <mo>=</mo>
                {if show_fraction {
                    <mfrac>
                        <mi>a</mi>
                        <mi>b</mi>
                    </mfrac>
                } else {
                    <mi>c</mi>
                }}
                {if show_exponent {
                    <msup>
                        <mi>x</mi>
                        <mn>2</mn>
                    </msup>
                }}
            </mrow>
        </math>
    };
}

#[test]
fn test_svg_with_loops() {
    let points = vec![(10, 10), (50, 30), (90, 10), (70, 60), (30, 60)];
    
    let _ = html! {
        <svg width="100" height="100">
            {for (x, y) in points {
                <circle cx={x} cy={y} r="3" fill="red" />
            }}
        </svg>
    };
}

#[test]
fn test_mathml_with_loops() {
    let variables = vec!["a", "b", "c"];
    
    let _ = html! {
        <math>
            <mrow>
                {for (i, var) in variables.iter().enumerate() {
                    <mrow>
                        <mi>{var}</mi>
                        {if i < variables.len() - 1 {
                            <mo>+</mo>
                        }}
                    </mrow>
                }}
            </mrow>
        </math>
    };
}

#[test]
fn test_svg_use_and_symbols() {
    let _ = html! {
        <svg width="200" height="100">
            <defs>
                <symbol id="star" viewBox="0 0 10 10">
                    <path d="M 5 0 L 6 3 L 10 3 L 7 5 L 8 8 L 5 6 L 2 8 L 3 5 L 0 3 L 4 3 Z" fill="gold" />
                </symbol>
            </defs>
            <use href="#star" x="10" y="10" width="20" height="20" />
            <use href="#star" x="50" y="10" width="30" height="30" />
            <use href="#star" x="100" y="10" width="25" height="25" />
        </svg>
    };
}

#[test] 
fn test_mathml_matrices_and_determinants() {
    let _ = html! {
        <math>
            <mrow>
                <mo>det</mo>
                <mo>(</mo>
                <mtable>
                    <mtr>
                        <mtd><mi>a</mi></mtd>
                        <mtd><mi>b</mi></mtd>
                    </mtr>
                    <mtr>
                        <mtd><mi>c</mi></mtd>
                        <mtd><mi>d</mi></mtd>
                    </mtr>
                </mtable>
                <mo>)</mo>
                <mo>=</mo>
                <mi>a</mi>
                <mi>d</mi>
                <mo>-</mo>
                <mi>b</mi>
                <mi>c</mi>
            </mrow>
        </math>
    };
}

#[test]
fn test_namespace_edge_cases() {
    // Test that HTML elements work normally
    let _ = html! {
        <div>
            <p>{"Regular HTML"}</p>
            <span>{"More HTML"}</span>
        </div>
    };
    
    // Test SVG without explicit namespace
    let _ = html! {
        <svg>
            <rect width="50" height="50" />
        </svg>
    };
    
    // Test MathML without explicit namespace  
    let _ = html! {
        <math>
            <mi>x</mi>
        </math>
    };
}

// Note: These are compile-time tests to ensure the macro syntax works correctly.
// The actual namespace detection and rendering would be tested at runtime
// when integrated with the main shipwright-liveview library.