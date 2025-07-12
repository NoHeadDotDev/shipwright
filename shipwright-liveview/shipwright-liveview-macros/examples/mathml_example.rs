//! MathML namespace support example for shipwright-liveview
//! 
//! This example demonstrates how to use MathML elements within the html! macro
//! with proper namespace detection and rendering for mathematical expressions.

use shipwright_liveview_macros::html;

fn main() {
    // Basic mathematical expressions
    let basic_math = html! {
        <div class="math-container">
            <h2>{"Basic Mathematical Expressions"}</h2>
            
            <p>{"Simple addition:"}</p>
            <math display="block">
                <mrow>
                    <mi>a</mi>
                    <mo>+</mo>
                    <mi>b</mi>
                    <mo>=</mo>
                    <mi>c</mi>
                </mrow>
            </math>
            
            <p>{"With numbers:"}</p>
            <math display="block">
                <mrow>
                    <mn>2</mn>
                    <mo>+</mo>
                    <mn>3</mn>
                    <mo>=</mo>
                    <mn>5</mn>
                </mrow>
            </math>
            
            <p>{"Mixed variables and numbers:"}</p>
            <math display="block">
                <mrow>
                    <mn>2</mn>
                    <mi>x</mi>
                    <mo>+</mo>
                    <mn>5</mn>
                    <mo>=</mo>
                    <mn>13</mn>
                </mrow>
            </math>
        </div>
    };

    // Fractions and complex expressions
    let fractions = html! {
        <div class="fractions-container">
            <h2>{"Fractions and Complex Expressions"}</h2>
            
            <p>{"Simple fraction:"}</p>
            <math display="block">
                <mfrac>
                    <mi>a</mi>
                    <mi>b</mi>
                </mfrac>
            </math>
            
            <p>{"Complex fraction:"}</p>
            <math display="block">
                <mfrac>
                    <mrow>
                        <mi>x</mi>
                        <mo>+</mo>
                        <mi>y</mi>
                    </mrow>
                    <mrow>
                        <mi>a</mi>
                        <mo>-</mo>
                        <mi>b</mi>
                    </mrow>
                </mfrac>
            </math>
            
            <p>{"Nested fractions:"}</p>
            <math display="block">
                <mfrac>
                    <mrow>
                        <mn>1</mn>
                        <mo>+</mo>
                        <mfrac>
                            <mi>x</mi>
                            <mi>y</mi>
                        </mfrac>
                    </mrow>
                    <mrow>
                        <mn>2</mn>
                        <mo>-</mo>
                        <mfrac>
                            <mi>a</mi>
                            <mi>b</mi>
                        </mfrac>
                    </mrow>
                </mfrac>
            </math>
        </div>
    };

    // Exponents and subscripts
    let superscripts = html! {
        <div class="superscripts-container">
            <h2>{"Exponents, Subscripts, and Powers"}</h2>
            
            <p>{"Simple exponent:"}</p>
            <math display="block">
                <msup>
                    <mi>x</mi>
                    <mn>2</mn>
                </msup>
            </math>
            
            <p>{"Subscript:"}</p>
            <math display="block">
                <msub>
                    <mi>a</mi>
                    <mn>1</mn>
                </msub>
            </math>
            
            <p>{"Both subscript and superscript:"}</p>
            <math display="block">
                <msubsup>
                    <mi>x</mi>
                    <mi>i</mi>
                    <mn>2</mn>
                </msubsup>
            </math>
            
            <p>{"Complex exponents:"}</p>
            <math display="block">
                <mrow>
                    <msup>
                        <mi>e</mi>
                        <mrow>
                            <mi>i</mi>
                            <mi>π</mi>
                        </mrow>
                    </msup>
                    <mo>=</mo>
                    <mo>-</mo>
                    <mn>1</mn>
                </mrow>
            </math>
            
            <p>{"Pythagorean theorem:"}</p>
            <math display="block">
                <mrow>
                    <msup>
                        <mi>a</mi>
                        <mn>2</mn>
                    </msup>
                    <mo>+</mo>
                    <msup>
                        <mi>b</mi>
                        <mn>2</mn>
                    </msup>
                    <mo>=</mo>
                    <msup>
                        <mi>c</mi>
                        <mn>2</mn>
                    </msup>
                </mrow>
            </math>
        </div>
    };

    // Square roots and radicals
    let radicals = html! {
        <div class="radicals-container">
            <h2>{"Square Roots and Radicals"}</h2>
            
            <p>{"Simple square root:"}</p>
            <math display="block">
                <msqrt>
                    <mi>x</mi>
                </msqrt>
            </math>
            
            <p>{"Square root of expression:"}</p>
            <math display="block">
                <msqrt>
                    <mrow>
                        <msup>
                            <mi>a</mi>
                            <mn>2</mn>
                        </msup>
                        <mo>+</mo>
                        <msup>
                            <mi>b</mi>
                            <mn>2</mn>
                        </msup>
                    </mrow>
                </msqrt>
            </math>
            
            <p>{"Nth root:"}</p>
            <math display="block">
                <mroot>
                    <mi>x</mi>
                    <mn>3</mn>
                </mroot>
            </math>
            
            <p>{"Complex radical:"}</p>
            <math display="block">
                <mroot>
                    <mrow>
                        <mi>x</mi>
                        <mo>+</mo>
                        <mi>y</mi>
                    </mrow>
                    <mrow>
                        <mi>n</mi>
                        <mo>+</mo>
                        <mn>1</mn>
                    </mrow>
                </mroot>
            </math>
        </div>
    };

    // Advanced mathematical notation
    let advanced_math = html! {
        <div class="advanced-container">
            <h2>{"Advanced Mathematical Notation"}</h2>
            
            <p>{"Quadratic formula:"}</p>
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
            
            <p>{"Integral with limits:"}</p>
            <math display="block">
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
            
            <p>{"Summation:"}</p>
            <math display="block">
                <mrow>
                    <munderover>
                        <mo>∑</mo>
                        <mrow>
                            <mi>n</mi>
                            <mo>=</mo>
                            <mn>1</mn>
                        </mrow>
                        <mi>∞</mi>
                    </munderover>
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
                            <mi>π</mi>
                            <mn>2</mn>
                        </msup>
                        <mn>6</mn>
                    </mfrac>
                </mrow>
            </math>
            
            <p>{"Matrix notation:"}</p>
            <math display="block">
                <mrow>
                    <mo>[</mo>
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
                    <mo>]</mo>
                </mrow>
            </math>
        </div>
    };

    // Mathematical entities and symbols
    let entities_demo = html! {
        <div class="entities-container">
            <h2>{"Mathematical Entities and Symbols"}</h2>
            
            <p>{"Greek letters:"}</p>
            <math display="block">
                <mrow>
                    <mi>α</mi>
                    <mo>,</mo>
                    <mi>β</mi>
                    <mo>,</mo>
                    <mi>γ</mi>
                    <mo>,</mo>
                    <mi>δ</mi>
                    <mo>,</mo>
                    <mi>π</mi>
                    <mo>,</mo>
                    <mi>σ</mi>
                    <mo>,</mo>
                    <mi>ω</mi>
                </mrow>
            </math>
            
            <p>{"Set theory symbols:"}</p>
            <math display="block">
                <mrow>
                    <mi>A</mi>
                    <mo>∩</mo>
                    <mi>B</mi>
                    <mo>⊂</mo>
                    <mi>C</mi>
                    <mo>∪</mo>
                    <mi>D</mi>
                    <mo>∈</mo>
                    <mi>E</mi>
                </mrow>
            </math>
            
            <p>{"Calculus operators:"}</p>
            <math display="block">
                <mrow>
                    <mo>∫</mo>
                    <mo>∑</mo>
                    <mo>∏</mo>
                    <mo>∂</mo>
                    <mo>∇</mo>
                    <mo>∞</mo>
                </mrow>
            </math>
            
            <p>{"Comparison operators:"}</p>
            <math display="block">
                <mrow>
                    <mi>x</mi>
                    <mo>≤</mo>
                    <mi>y</mi>
                    <mo>≥</mo>
                    <mi>z</mi>
                    <mo>≠</mo>
                    <mi>w</mi>
                    <mo>≈</mo>
                    <mi>v</mi>
                </mrow>
            </math>
        </div>
    };

    // Dynamic mathematical content
    let dynamic_math = create_dynamic_math();

    println!("MathML Examples Generated Successfully!");
    println!("Basic Math: {}", basic_math);
    println!("Fractions: {}", fractions);
    println!("Superscripts: {}", superscripts);
    println!("Radicals: {}", radicals);
    println!("Advanced Math: {}", advanced_math);
    println!("Entities Demo: {}", entities_demo);
    println!("Dynamic Math: {}", dynamic_math);
}

fn create_dynamic_math() -> String {
    let coefficients = vec![2, -3, 1];
    let variables = vec!["x", "y", "z"];
    let powers = vec![2, 1, 3];
    
    let polynomial = html! {
        <div class="dynamic-math-container">
            <h2>{"Dynamic Mathematical Content"}</h2>
            
            <p>{"Dynamically generated polynomial:"}</p>
            <math display="block">
                <mrow>
                    <mi>f</mi>
                    <mo>(</mo>
                    <mi>x</mi>
                    <mo>)</mo>
                    <mo>=</mo>
                    {for (i, ((coeff, var), power)) in coefficients.iter().zip(variables.iter()).zip(powers.iter()).enumerate() {
                        <mrow>
                            {if i > 0 && *coeff > 0 {
                                <mo>+</mo>
                            }}
                            {if *coeff != 1 || *power == 0 {
                                <mn>{coeff}</mn>
                            }}
                            {if *power > 0 {
                                {if *power == 1 {
                                    <mi>{var}</mi>
                                } else {
                                    <msup>
                                        <mi>{var}</mi>
                                        <mn>{power}</mn>
                                    </msup>
                                }}
                            }}
                        </mrow>
                    }}
                </mrow>
            </math>
            
            <p>{"Parametric equations:"}</p>
            <math display="block">
                <mrow>
                    <mi>x</mi>
                    <mo>=</mo>
                    <mi>r</mi>
                    <mo>cos</mo>
                    <mo>(</mo>
                    <mi>t</mi>
                    <mo>)</mo>
                    <mo>,</mo>
                    <mspace width="1em" />
                    <mi>y</mi>
                    <mo>=</mo>
                    <mi>r</mi>
                    <mo>sin</mo>
                    <mo>(</mo>
                    <mi>t</mi>
                    <mo>)</mo>
                </mrow>
            </math>
            
            <p>{format!("Generated with {} terms", coefficients.len())}</p>
        </div>
    };

    polynomial.to_string()
}

// Example of conditional mathematical expressions
fn conditional_math_example() -> String {
    let show_solution = true;
    let equation_type = "quadratic";
    
    let conditional_math = html! {
        <div class="conditional-math">
            <h3>{"Conditional Mathematical Content"}</h3>
            
            <p>{format!("Solving a {} equation:", equation_type)}</p>
            
            <math display="block">
                <mrow>
                    <msup>
                        <mi>x</mi>
                        <mn>2</mn>
                    </msup>
                    <mo>+</mo>
                    <mn>2</mn>
                    <mi>x</mi>
                    <mo>-</mo>
                    <mn>3</mn>
                    <mo>=</mo>
                    <mn>0</mn>
                </mrow>
            </math>
            
            {if show_solution {
                <div>
                    <p>{"Solution:"}</p>
                    <math display="block">
                        <mrow>
                            <mi>x</mi>
                            <mo>=</mo>
                            <mfrac>
                                <mrow>
                                    <mo>-</mo>
                                    <mn>2</mn>
                                    <mo>±</mo>
                                    <msqrt>
                                        <mrow>
                                            <mn>4</mn>
                                            <mo>+</mo>
                                            <mn>12</mn>
                                        </mrow>
                                    </msqrt>
                                </mrow>
                                <mn>2</mn>
                            </mfrac>
                            <mo>=</mo>
                            <mrow>
                                <mn>1</mn>
                                <mo>,</mo>
                                <mo>-</mo>
                                <mn>3</mn>
                            </mrow>
                        </mrow>
                    </math>
                </div>
            } else {
                <p>{"Solution hidden. Set show_solution = true to reveal."}</p>
            }}
        </div>
    };

    conditional_math.to_string()
}