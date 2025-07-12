//! Mixed namespace example demonstrating HTML, SVG, and MathML together
//! 
//! This example shows how different namespaces can be used together seamlessly
//! in a single document with proper namespace detection and rendering.

use shipwright_liveview_macros::html;

fn main() {
    // Complete example with all three namespaces
    let mixed_document = html! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <title>{"Mixed Namespaces Example"}</title>
                <style>
                    {"
                        body { 
                            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; 
                            line-height: 1.6; 
                            margin: 40px;
                            background: #f8f9fa;
                        }
                        .container { 
                            max-width: 1200px; 
                            margin: 0 auto; 
                            background: white;
                            padding: 30px;
                            border-radius: 10px;
                            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                        }
                        .section { 
                            margin: 30px 0; 
                            padding: 20px;
                            border-left: 4px solid #007bff;
                            background: #f8f9fa;
                        }
                        .math-section {
                            border-left-color: #28a745;
                        }
                        .svg-section {
                            border-left-color: #ffc107;
                        }
                        h1, h2 { 
                            color: #333; 
                        }
                        svg { 
                            border: 1px solid #ddd; 
                            border-radius: 5px;
                            background: white;
                        }
                        math {
                            font-size: 1.2em;
                        }
                    "}
                </style>
            </head>
            <body>
                <div class="container">
                    <header>
                        <h1>{"Shipwright LiveView: Multi-Namespace Support"}</h1>
                        <p>{"This document demonstrates HTML, SVG, and MathML working together seamlessly."}</p>
                    </header>

                    <section class="section">
                        <h2>{"1. HTML Content"}</h2>
                        <p>{"This is regular HTML content with various elements:"}</p>
                        <ul>
                            <li>{"Text formatting: "}<strong>{"bold"}</strong>{", "}<em>{"italic"}</em>{", "}<code>{"code"}</code></li>
                            <li>{"Links: "}<a href="https://example.com">{"example link"}</a></li>
                            <li>{"Form elements below:"}</li>
                        </ul>
                        
                        <form>
                            <fieldset>
                                <legend>{"Sample Form"}</legend>
                                <div style="margin: 10px 0;">
                                    <label for="name">{"Name: "}</label>
                                    <input type="text" id="name" name="name" placeholder="Enter your name" />
                                </div>
                                <div style="margin: 10px 0;">
                                    <label for="email">{"Email: "}</label>
                                    <input type="email" id="email" name="email" placeholder="your@email.com" />
                                </div>
                                <div style="margin: 10px 0;">
                                    <input type="checkbox" id="subscribe" name="subscribe" />
                                    <label for="subscribe">{"Subscribe to newsletter"}</label>
                                </div>
                                <button type="submit">{"Submit"}</button>
                            </fieldset>
                        </form>
                    </section>

                    <section class="section svg-section">
                        <h2>{"2. SVG Graphics"}</h2>
                        <p>{"Scalable Vector Graphics embedded within HTML:"}</p>
                        
                        <div style="display: flex; gap: 20px; flex-wrap: wrap;">
                            <div>
                                <h3>{"Basic Shapes"}</h3>
                                <svg width="200" height="150" viewBox="0 0 200 150">
                                    <rect x="10" y="10" width="60" height="40" fill="#007bff" />
                                    <circle cx="130" cy="30" r="25" fill="#28a745" />
                                    <polygon points="50,70 80,70 90,100 40,100" fill="#ffc107" />
                                    <line x1="10" y1="120" x2="190" y2="120" stroke="#dc3545" stroke-width="3" />
                                    <text x="100" y="140" text-anchor="middle" font-size="12" fill="#333">
                                        {"SVG Shapes"}
                                    </text>
                                </svg>
                            </div>
                            
                            <div>
                                <h3>{"Data Visualization"}</h3>
                                <svg width="250" height="150" viewBox="0 0 250 150">
                                    <defs>
                                        <linearGradient id="barGradient" x1="0%" y1="0%" x2="0%" y2="100%">
                                            <stop offset="0%" stop-color="#007bff" />
                                            <stop offset="100%" stop-color="#0056b3" />
                                        </linearGradient>
                                    </defs>
                                    
                                    // Chart bars
                                    <rect x="20" y="80" width="30" height="50" fill="url(#barGradient)" />
                                    <rect x="60" y="60" width="30" height="70" fill="url(#barGradient)" />
                                    <rect x="100" y="40" width="30" height="90" fill="url(#barGradient)" />
                                    <rect x="140" y="70" width="30" height="60" fill="url(#barGradient)" />
                                    <rect x="180" y="50" width="30" height="80" fill="url(#barGradient)" />
                                    
                                    // Axes
                                    <line x1="15" y1="135" x2="225" y2="135" stroke="#333" stroke-width="2" />
                                    <line x1="15" y1="30" x2="15" y2="135" stroke="#333" stroke-width="2" />
                                    
                                    // Labels
                                    <text x="125" y="15" text-anchor="middle" font-size="12" font-weight="bold">
                                        {"Sales Data"}
                                    </text>
                                    <text x="35" y="145" text-anchor="middle" font-size="10">{"Q1"}</text>
                                    <text x="75" y="145" text-anchor="middle" font-size="10">{"Q2"}</text>
                                    <text x="115" y="145" text-anchor="middle" font-size="10">{"Q3"}</text>
                                    <text x="155" y="145" text-anchor="middle" font-size="10">{"Q4"}</text>
                                    <text x="195" y="145" text-anchor="middle" font-size="10">{"Q5"}</text>
                                </svg>
                            </div>
                        </div>
                        
                        <div style="margin-top: 20px;">
                            <h3>{"Interactive Elements"}</h3>
                            <svg width="300" height="100" viewBox="0 0 300 100">
                                <circle cx="50" cy="50" r="20" fill="#007bff">
                                    <animate attributeName="r" values="20;30;20" dur="2s" repeatCount="indefinite" />
                                </circle>
                                <rect x="100" y="30" width="40" height="40" fill="#28a745">
                                    <animateTransform 
                                        attributeName="transform" 
                                        type="rotate" 
                                        values="0 120 50;360 120 50" 
                                        dur="3s" 
                                        repeatCount="indefinite" />
                                </rect>
                                <polygon points="200,30 230,50 200,70 170,50" fill="#ffc107">
                                    <animate attributeName="fill" 
                                        values="#ffc107;#fd7e14;#ffc107" 
                                        dur="1.5s" 
                                        repeatCount="indefinite" />
                                </polygon>
                                <text x="150" y="90" text-anchor="middle" font-size="12">
                                    {"Animated SVG Elements"}
                                </text>
                            </svg>
                        </div>
                    </section>

                    <section class="section math-section">
                        <h2>{"3. Mathematical Notation (MathML)"}</h2>
                        <p>{"Mathematical expressions rendered with MathML:"}</p>
                        
                        <div style="display: grid; gap: 20px;">
                            <div>
                                <h3>{"Famous Equations"}</h3>
                                
                                <div style="margin: 15px 0;">
                                    <strong>{"Einstein's Mass-Energy Equivalence:"}</strong>
                                    <math display="block">
                                        <mrow>
                                            <mi>E</mi>
                                            <mo>=</mo>
                                            <mi>m</mi>
                                            <msup>
                                                <mi>c</mi>
                                                <mn>2</mn>
                                            </msup>
                                        </mrow>
                                    </math>
                                </div>
                                
                                <div style="margin: 15px 0;">
                                    <strong>{"Euler's Identity:"}</strong>
                                    <math display="block">
                                        <mrow>
                                            <msup>
                                                <mi>e</mi>
                                                <mrow>
                                                    <mi>i</mi>
                                                    <mi>π</mi>
                                                </mrow>
                                            </msup>
                                            <mo>+</mo>
                                            <mn>1</mn>
                                            <mo>=</mo>
                                            <mn>0</mn>
                                        </mrow>
                                    </math>
                                </div>
                                
                                <div style="margin: 15px 0;">
                                    <strong>{"Pythagorean Theorem:"}</strong>
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
                            </div>
                            
                            <div>
                                <h3>{"Calculus Examples"}</h3>
                                
                                <div style="margin: 15px 0;">
                                    <strong>{"Fundamental Theorem of Calculus:"}</strong>
                                    <math display="block">
                                        <mrow>
                                            <msubsup>
                                                <mo>∫</mo>
                                                <mi>a</mi>
                                                <mi>b</mi>
                                            </msubsup>
                                            <mi>f</mi>
                                            <mo>(</mo>
                                            <mi>x</mi>
                                            <mo>)</mo>
                                            <mo>d</mo>
                                            <mi>x</mi>
                                            <mo>=</mo>
                                            <mi>F</mi>
                                            <mo>(</mo>
                                            <mi>b</mi>
                                            <mo>)</mo>
                                            <mo>-</mo>
                                            <mi>F</mi>
                                            <mo>(</mo>
                                            <mi>a</mi>
                                            <mo>)</mo>
                                        </mrow>
                                    </math>
                                </div>
                                
                                <div style="margin: 15px 0;">
                                    <strong>{"Derivative Definition:"}</strong>
                                    <math display="block">
                                        <mrow>
                                            <msup>
                                                <mi>f</mi>
                                                <mo>′</mo>
                                            </msup>
                                            <mo>(</mo>
                                            <mi>x</mi>
                                            <mo>)</mo>
                                            <mo>=</mo>
                                            <munder>
                                                <mo>lim</mo>
                                                <mrow>
                                                    <mi>h</mi>
                                                    <mo>→</mo>
                                                    <mn>0</mn>
                                                </mrow>
                                            </munder>
                                            <mfrac>
                                                <mrow>
                                                    <mi>f</mi>
                                                    <mo>(</mo>
                                                    <mi>x</mi>
                                                    <mo>+</mo>
                                                    <mi>h</mi>
                                                    <mo>)</mo>
                                                    <mo>-</mo>
                                                    <mi>f</mi>
                                                    <mo>(</mo>
                                                    <mi>x</mi>
                                                    <mo>)</mo>
                                                </mrow>
                                                <mi>h</mi>
                                            </mfrac>
                                        </mrow>
                                    </math>
                                </div>
                            </div>
                            
                            <div>
                                <h3>{"Matrix Operations"}</h3>
                                <strong>{"Matrix Multiplication:"}</strong>
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
                                        <mo>×</mo>
                                        <mo>[</mo>
                                        <mtable>
                                            <mtr>
                                                <mtd><mi>e</mi></mtd>
                                                <mtd><mi>f</mi></mtd>
                                            </mtr>
                                            <mtr>
                                                <mtd><mi>g</mi></mtd>
                                                <mtd><mi>h</mi></mtd>
                                            </mtr>
                                        </mtable>
                                        <mo>]</mo>
                                        <mo>=</mo>
                                        <mo>[</mo>
                                        <mtable>
                                            <mtr>
                                                <mtd>
                                                    <mrow>
                                                        <mi>a</mi>
                                                        <mi>e</mi>
                                                        <mo>+</mo>
                                                        <mi>b</mi>
                                                        <mi>g</mi>
                                                    </mrow>
                                                </mtd>
                                                <mtd>
                                                    <mrow>
                                                        <mi>a</mi>
                                                        <mi>f</mi>
                                                        <mo>+</mo>
                                                        <mi>b</mi>
                                                        <mi>h</mi>
                                                    </mrow>
                                                </mtd>
                                            </mtr>
                                            <mtr>
                                                <mtd>
                                                    <mrow>
                                                        <mi>c</mi>
                                                        <mi>e</mi>
                                                        <mo>+</mo>
                                                        <mi>d</mi>
                                                        <mi>g</mi>
                                                    </mrow>
                                                </mtd>
                                                <mtd>
                                                    <mrow>
                                                        <mi>c</mi>
                                                        <mi>f</mi>
                                                        <mo>+</mo>
                                                        <mi>d</mi>
                                                        <mi>h</mi>
                                                    </mrow>
                                                </mtd>
                                            </mtr>
                                        </mtable>
                                        <mo>]</mo>
                                    </mrow>
                                </math>
                            </div>
                        </div>
                    </section>

                    <section class="section">
                        <h2>{"4. Interactive Mixed Content"}</h2>
                        <p>{"Combining all three namespaces with dynamic content:"}</p>
                        
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px; align-items: start;">
                            <div>
                                <h3>{"Mathematical Graph"}</h3>
                                <svg width="280" height="200" viewBox="0 0 280 200">
                                    // Coordinate system
                                    <line x1="40" y1="160" x2="260" y2="160" stroke="#333" stroke-width="1" />
                                    <line x1="40" y1="20" x2="40" y2="160" stroke="#333" stroke-width="1" />
                                    
                                    // Axis labels
                                    <text x="150" y="180" text-anchor="middle" font-size="12">{"x"}</text>
                                    <text x="25" y="95" text-anchor="middle" font-size="12">{"y"}</text>
                                    
                                    // Parabola y = x²
                                    <path d="M 40 160 Q 150 40 260 160" 
                                          stroke="#007bff" stroke-width="2" fill="none" />
                                    
                                    // Points
                                    <circle cx="40" cy="160" r="3" fill="#dc3545" />
                                    <circle cx="150" cy="40" r="3" fill="#dc3545" />
                                    <circle cx="260" cy="160" r="3" fill="#dc3545" />
                                    
                                    <text x="150" y="15" text-anchor="middle" font-size="14" font-weight="bold">
                                        {"Graph of y = x²"}
                                    </text>
                                </svg>
                                
                                <div style="margin-top: 10px;">
                                    <strong>{"Equation:"}</strong>
                                    <math>
                                        <mrow>
                                            <mi>y</mi>
                                            <mo>=</mo>
                                            <msup>
                                                <mi>x</mi>
                                                <mn>2</mn>
                                            </msup>
                                        </mrow>
                                    </math>
                                </div>
                            </div>
                            
                            <div>
                                <h3>{"Form with Validation"}</h3>
                                <form style="background: #f8f9fa; padding: 15px; border-radius: 5px;">
                                    <div style="margin-bottom: 15px;">
                                        <label for="equation-input" style="display: block; margin-bottom: 5px;">
                                            {"Enter coefficient for "}
                                            <math>
                                                <mrow>
                                                    <mi>a</mi>
                                                    <msup>
                                                        <mi>x</mi>
                                                        <mn>2</mn>
                                                    </msup>
                                                    <mo>+</mo>
                                                    <mi>b</mi>
                                                    <mi>x</mi>
                                                    <mo>+</mo>
                                                    <mi>c</mi>
                                                </mrow>
                                            </math>
                                            {":"}
                                        </label>
                                        <input type="number" id="equation-input" 
                                               placeholder="Enter value for 'a'" 
                                               style="width: 100%; padding: 8px; border: 1px solid #ddd; border-radius: 3px;" />
                                    </div>
                                    
                                    <div style="margin-bottom: 15px;">
                                        <button type="button" 
                                                style="background: #007bff; color: white; border: none; padding: 10px 15px; border-radius: 3px; cursor: pointer;">
                                            {"Calculate Discriminant"}
                                        </button>
                                    </div>
                                    
                                    <div style="font-size: 0.9em; color: #666;">
                                        {"Discriminant: "}
                                        <math>
                                            <mrow>
                                                <mi>Δ</mi>
                                                <mo>=</mo>
                                                <msup>
                                                    <mi>b</mi>
                                                    <mn>2</mn>
                                                </msup>
                                                <mo>-</mo>
                                                <mn>4</mn>
                                                <mi>a</mi>
                                                <mi>c</mi>
                                            </mrow>
                                        </math>
                                    </div>
                                </form>
                            </div>
                        </div>
                    </section>

                    <footer style="margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; text-align: center; color: #666;">
                        <p>
                            {"This example demonstrates the seamless integration of HTML, SVG, and MathML "}
                            {"within the Shipwright LiveView framework using namespace-aware parsing."}
                        </p>
                        <div style="margin-top: 15px;">
                            <svg width="30" height="30" viewBox="0 0 30 30" style="margin: 0 10px;">
                                <circle cx="15" cy="15" r="12" fill="#007bff" />
                                <text x="15" y="20" text-anchor="middle" fill="white" font-size="12" font-weight="bold">
                                    {"H"}
                                </text>
                            </svg>
                            <math style="margin: 0 10px;">
                                <mi>+</mi>
                            </math>
                            <svg width="30" height="30" viewBox="0 0 30 30" style="margin: 0 10px;">
                                <rect x="3" y="3" width="24" height="24" fill="#28a745" />
                                <text x="15" y="20" text-anchor="middle" fill="white" font-size="11" font-weight="bold">
                                    {"SVG"}
                                </text>
                            </svg>
                            <math style="margin: 0 10px;">
                                <mi>+</mi>
                            </math>
                            <svg width="40" height="30" viewBox="0 0 40 30" style="margin: 0 10px;">
                                <rect x="2" y="2" width="36" height="26" fill="#ffc107" />
                                <text x="20" y="20" text-anchor="middle" fill="black" font-size="10" font-weight="bold">
                                    {"MathML"}
                                </text>
                            </svg>
                            <math style="margin: 0 10px;">
                                <mo>=</mo>
                            </math>
                            <strong style="color: #007bff;">{"Powerful Web Documents"}</strong>
                        </div>
                    </footer>
                </div>
            </body>
        </html>
    };

    println!("Mixed Namespaces Example Generated Successfully!");
    println!("{}", mixed_document);
}

// Dynamic content generation combining all namespaces
fn dynamic_mixed_content() -> String {
    let data_points = vec![(1, 1), (2, 4), (3, 9), (4, 16), (5, 25)];
    let show_formula = true;
    
    let dynamic_content = html! {
        <div class="dynamic-mixed">
            <h2>{"Dynamic Content with Mixed Namespaces"}</h2>
            
            <div style="display: flex; gap: 20px;">
                <div>
                    <h3>{"Data Points"}</h3>
                    <svg width="250" height="200" viewBox="0 0 250 200">
                        // Axes
                        <line x1="30" y1="170" x2="220" y2="170" stroke="#333" />
                        <line x1="30" y1="30" x2="30" y2="170" stroke="#333" />
                        
                        // Plot points
                        {for (x, y) in data_points.iter() {
                            <circle 
                                cx={30 + x * 35} 
                                cy={170 - y * 5} 
                                r="4" 
                                fill="#007bff" 
                            />
                        }}
                        
                        // Connect points
                        <polyline 
                            points={data_points.iter()
                                .map(|(x, y)| format!("{},{}", 30 + x * 35, 170 - y * 5))
                                .collect::<Vec<_>>()
                                .join(" ")
                            }
                            stroke="#007bff" 
                            stroke-width="2" 
                            fill="none" 
                        />
                        
                        <text x="125" y="20" text-anchor="middle" font-size="14" font-weight="bold">
                            {"Function Plot"}
                        </text>
                    </svg>
                </div>
                
                <div>
                    <h3>{"Pattern Analysis"}</h3>
                    <table style="border-collapse: collapse;">
                        <thead>
                            <tr>
                                <th style="border: 1px solid #ddd; padding: 8px;">{"x"}</th>
                                <th style="border: 1px solid #ddd; padding: 8px;">{"y"}</th>
                                <th style="border: 1px solid #ddd; padding: 8px;">{"x²"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {for (x, y) in data_points.iter() {
                                <tr>
                                    <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">{x}</td>
                                    <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">{y}</td>
                                    <td style="border: 1px solid #ddd; padding: 8px; text-align: center;">{x * x}</td>
                                </tr>
                            }}
                        </tbody>
                    </table>
                    
                    {if show_formula {
                        <div style="margin-top: 15px;">
                            <strong>{"Pattern:"}</strong>
                            <math display="block">
                                <mrow>
                                    <mi>y</mi>
                                    <mo>=</mo>
                                    <msup>
                                        <mi>x</mi>
                                        <mn>2</mn>
                                    </msup>
                                </mrow>
                            </math>
                        </div>
                    }}
                </div>
            </div>
            
            <p style="margin-top: 20px;">
                {format!("Generated {} data points demonstrating quadratic relationship.", data_points.len())}
            </p>
        </div>
    };

    dynamic_content.to_string()
}