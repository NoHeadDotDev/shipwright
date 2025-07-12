//! Entity handling tests for HTML, SVG, and MathML namespaces

use shipwright_liveview_macros::html;

#[test]
fn test_html_entities() {
    let _ = html! {
        <div>
            <p>{"Common entities: &amp; &lt; &gt; &quot; &apos;"}</p>
            <p>{"Special chars: &nbsp; &copy; &reg; &trade;"}</p>
            <p>{"Symbols: &#8364; &#163; &#165;"}</p>
        </div>
    };
}

#[test]
fn test_svg_with_entities() {
    let _ = html! {
        <svg width="200" height="100">
            <text x="10" y="30" font-family="serif">
                {"Entities in SVG: &amp; &lt; &gt;"}
            </text>
            <text x="10" y="60" font-family="sans-serif">
                {"Unicode: &#8364; &#9733; &#9829;"}
            </text>
        </svg>
    };
}

#[test]
fn test_mathml_with_entities() {
    let _ = html! {
        <math>
            <mrow>
                <mi>{"&alpha;"}</mi>
                <mo>{"+"}</mo>
                <mi>{"&beta;"}</mi>
                <mo>{"="}</mo>
                <mi>{"&gamma;"}</mi>
            </mrow>
        </math>
    };
}

#[test]
fn test_mathml_mathematical_entities() {
    let _ = html! {
        <math>
            <mrow>
                <mo>{"&int;"}</mo>
                <mi>f</mi>
                <mo>(</mo>
                <mi>x</mi>
                <mo>)</mo>
                <mo>d</mo>
                <mi>x</mi>
                <mo>{"&ap;"}</mo>
                <mfrac>
                    <mi>{"&pi;"}</mi>
                    <mn>2</mn>
                </mfrac>
            </mrow>
        </math>
    };
}

#[test]
fn test_mathml_greek_letters() {
    let _ = html! {
        <math>
            <mrow>
                <mi>{"&alpha;"}</mi>
                <mo>,</mo>
                <mi>{"&beta;"}</mi>
                <mo>,</mo>
                <mi>{"&gamma;"}</mi>
                <mo>,</mo>
                <mi>{"&delta;"}</mi>
                <mo>,</mo>
                <mi>{"&epsilon;"}</mi>
                <mo>,</mo>
                <mi>{"&theta;"}</mi>
                <mo>,</mo>
                <mi>{"&pi;"}</mi>
                <mo>,</mo>
                <mi>{"&sigma;"}</mi>
                <mo>,</mo>
                <mi>{"&omega;"}</mi>
            </mrow>
        </math>
    };
}

#[test]
fn test_mathml_operators_and_symbols() {
    let _ = html! {
        <math>
            <mrow>
                <mi>x</mi>
                <mo>{"&isin;"}</mo>
                <mi>A</mi>
                <mo>{"&cap;"}</mo>
                <mi>B</mi>
                <mo>{"&sub;"}</mo>
                <mi>C</mi>
                <mo>{"&cup;"}</mo>
                <mi>D</mi>
            </mrow>
        </math>
    };
}

#[test]
fn test_mathml_advanced_symbols() {
    let _ = html! {
        <math>
            <mrow>
                <mo>{"&sum;"}</mo>
                <msubsup>
                    <mi>{"&empty;"}</mi>
                    <mrow>
                        <mi>n</mi>
                        <mo>=</mo>
                        <mn>1</mn>
                    </mrow>
                    <mi>{"&infin;"}</mi>
                </msubsup>
                <mfrac>
                    <mn>1</mn>
                    <msup>
                        <mi>n</mi>
                        <mn>2</mn>
                    </msup>
                </mfrac>
                <mo>=</mo>
                <mfrac>
                    <msup>
                        <mi>{"&pi;"}</mi>
                        <mn>2</mn>
                    </msup>
                    <mn>6</mn>
                </mfrac>
            </mrow>
        </math>
    };
}

#[test]
fn test_mixed_entities_in_complex_expression() {
    let _ = html! {
        <div>
            <h3>{"Schrödinger Equation"}</h3>
            <math display="block">
                <mrow>
                    <mi>i</mi>
                    <mi>&#8463;</mi>
                    <mfrac>
                        <mrow>
                            <mo>{"&part;"}</mo>
                            <mi>{"&psi;"}</mi>
                        </mrow>
                        <mrow>
                            <mo>{"&part;"}</mo>
                            <mi>t</mi>
                        </mrow>
                    </mfrac>
                    <mo>=</mo>
                    <mi>H</mi>
                    <mi>{"&psi;"}</mi>
                </mrow>
            </math>
            <p>{"Where &#8463; is the reduced Planck constant &amp; H is the Hamiltonian operator."}</p>
        </div>
    };
}

#[test]
fn test_svg_mathematical_notation() {
    let _ = html! {
        <svg width="300" height="150">
            <text x="20" y="30" font-size="20" font-family="serif">
                {"Mathematical symbols in SVG:"}
            </text>
            <text x="20" y="60" font-size="16">
                {"&#8747; &#8721; &#8719; &#8730; &#8734; &#8804; &#8805; &#8800;"}
            </text>
            <text x="20" y="90" font-size="16">
                {"Greek: &#945; &#946; &#947; &#948; &#960; &#963; &#969;"}
            </text>
            <text x="20" y="120" font-size="16">
                {"Arrows: &#8594; &#8592; &#8593; &#8595; &#8596; &#8656; &#8658;"}
            </text>
        </svg>
    };
}

#[test]
fn test_dynamic_entities() {
    let symbol = "&alpha;";
    let operator = "&plus;";
    let result = "&beta;";
    
    let _ = html! {
        <math>
            <mrow>
                <mi>{symbol}</mi>
                <mo>{operator}</mo>
                <mi>x</mi>
                <mo>=</mo>
                <mi>{result}</mi>
            </mrow>
        </math>
    };
}

#[test]
fn test_entities_in_attributes() {
    let _ = html! {
        <div>
            <svg width="200" height="100">
                <text x="10" y="30" font-family="Times New Roman" title="Contains &amp; symbol">
                    {"Text with title containing entity"}
                </text>
                <rect x="10" y="40" width="50" height="20" fill="blue" title="Width &gt; Height" />
            </svg>
            <p title="Quote: &quot;Hello World&quot;">{"Paragraph with quoted title"}</p>
        </div>
    };
}

#[test]
fn test_complex_entity_combinations() {
    let _ = html! {
        <article>
            <h2>{"Mathematical Formulas &amp; Equations"}</h2>
            <math display="block">
                <mrow>
                    <mo>{"&forall;"}</mo>
                    <mi>x</mi>
                    <mo>{"&isin;"}</mo>
                    <mi>&#8477;</mi>
                    <mo>:</mo>
                    <mi>e</mi>
                    <msup>
                        <mi>i</mi>
                        <mrow>
                            <mi>{"&pi;"}</mi>
                            <mi>x</mi>
                        </mrow>
                    </msup>
                    <mo>=</mo>
                    <mo>cos</mo>
                    <mo>(</mo>
                    <mi>{"&pi;"}</mi>
                    <mi>x</mi>
                    <mo>)</mo>
                    <mo>+</mo>
                    <mi>i</mi>
                    <mo>sin</mo>
                    <mo>(</mo>
                    <mi>{"&pi;"}</mi>
                    <mi>x</mi>
                    <mo>)</mo>
                </mrow>
            </math>
            <p>{"This demonstrates Euler's formula using entities for ∀, ∈, π, and other symbols."}</p>
        </article>
    };
}

#[test]
fn test_entity_edge_cases() {
    let _ = html! {
        <div>
            <p>{"Literal ampersand: &amp;amp; becomes &amp;"}</p>
            <p>{"Mixed: &lt;tag&gt; and &#60;numeric&#62;"}</p>
            <math>
                <mrow>
                    <mi>{"Unknown entity should be literal: &unknown;"}</mi>
                </mrow>
            </math>
        </div>
    };
}

// Tests for namespace-specific entity validation
#[cfg(test)]
mod entity_validation_tests {
    use shipwright_liveview_macros_internal::namespaces::{EntityHandler, Namespace};
    
    #[test]
    fn test_html_entity_validation() {
        let handler = EntityHandler::new();
        
        // Standard HTML entities
        assert!(handler.is_valid_entity("amp", Namespace::Html));
        assert!(handler.is_valid_entity("lt", Namespace::Html));
        assert!(handler.is_valid_entity("gt", Namespace::Html));
        assert!(handler.is_valid_entity("quot", Namespace::Html));
        assert!(handler.is_valid_entity("nbsp", Namespace::Html));
        
        // Mathematical entities should not be valid in HTML
        assert!(!handler.is_valid_entity("alpha", Namespace::Html));
        assert!(!handler.is_valid_entity("sum", Namespace::Html));
    }
    
    #[test]
    fn test_svg_entity_validation() {
        let handler = EntityHandler::new();
        
        // SVG should support HTML entities
        assert!(handler.is_valid_entity("amp", Namespace::Svg));
        assert!(handler.is_valid_entity("lt", Namespace::Svg));
        
        // Mathematical entities should not be valid in SVG
        assert!(!handler.is_valid_entity("alpha", Namespace::Svg));
        assert!(!handler.is_valid_entity("integral", Namespace::Svg));
    }
    
    #[test]
    fn test_mathml_entity_validation() {
        let handler = EntityHandler::new();
        
        // MathML should support both HTML and mathematical entities
        assert!(handler.is_valid_entity("amp", Namespace::MathML));
        assert!(handler.is_valid_entity("lt", Namespace::MathML));
        assert!(handler.is_valid_entity("alpha", Namespace::MathML));
        assert!(handler.is_valid_entity("beta", Namespace::MathML));
        assert!(handler.is_valid_entity("sum", Namespace::MathML));
        assert!(handler.is_valid_entity("integral", Namespace::MathML));
        
        // Unknown entities should not be valid
        assert!(!handler.is_valid_entity("unknown", Namespace::MathML));
        assert!(!handler.is_valid_entity("fake", Namespace::MathML));
    }
    
    #[test]
    fn test_entity_availability() {
        let handler = EntityHandler::new();
        
        let html_entities = handler.get_available_entities(Namespace::Html);
        let svg_entities = handler.get_available_entities(Namespace::Svg);
        let mathml_entities = handler.get_available_entities(Namespace::MathML);
        
        // HTML should have the smallest set
        assert!(html_entities.len() > 0);
        
        // SVG should have at least as many as HTML
        assert!(svg_entities.len() >= html_entities.len());
        
        // MathML should have the most entities
        assert!(mathml_entities.len() >= svg_entities.len());
        
        // Verify specific entities are in the right sets
        assert!(html_entities.contains(&"amp"));
        assert!(svg_entities.contains(&"amp"));
        assert!(mathml_entities.contains(&"amp"));
        assert!(mathml_entities.contains(&"alpha"));
        assert!(!html_entities.contains(&"alpha"));
    }
}