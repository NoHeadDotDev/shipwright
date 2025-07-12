//! SVG namespace support example for shipwright-liveview
//! 
//! This example demonstrates how to use SVG elements within the html! macro
//! with proper namespace detection and rendering.

use shipwright_liveview_macros::html;

fn main() {
    // Basic SVG with shapes
    let basic_svg = html! {
        <div class="svg-container">
            <h2>{"Basic SVG Shapes"}</h2>
            <svg width="300" height="200" viewBox="0 0 300 200">
                <rect x="10" y="10" width="100" height="80" fill="#3498db" stroke="#2980b9" stroke-width="2" />
                <circle cx="200" cy="50" r="40" fill="#e74c3c" stroke="#c0392b" stroke-width="2" />
                <ellipse cx="250" cy="150" rx="30" ry="20" fill="#f39c12" stroke="#e67e22" stroke-width="2" />
                <line x1="50" y1="120" x2="150" y2="180" stroke="#9b59b6" stroke-width="3" />
                <polygon points="180,120 220,120 230,160 170,160" fill="#1abc9c" stroke="#16a085" stroke-width="2" />
            </svg>
        </div>
    };

    // SVG with paths and complex shapes
    let path_svg = html! {
        <div class="path-container">
            <h2>{"SVG Paths and Complex Shapes"}</h2>
            <svg width="400" height="300" viewBox="0 0 400 300">
                <defs>
                    <linearGradient id="gradient1" x1="0%" y1="0%" x2="100%" y2="100%">
                        <stop offset="0%" stop-color="#ff7675" />
                        <stop offset="50%" stop-color="#fd79a8" />
                        <stop offset="100%" stop-color="#e84393" />
                    </linearGradient>
                    <pattern id="pattern1" x="0" y="0" width="20" height="20" patternUnits="userSpaceOnUse">
                        <circle cx="10" cy="10" r="5" fill="#00b894" />
                    </pattern>
                </defs>
                
                <path d="M 50 150 Q 100 50 150 150 T 250 150" 
                      stroke="#2d3436" stroke-width="3" fill="none" />
                
                <path d="M 300 50 L 350 100 L 325 150 L 275 150 L 250 100 Z" 
                      fill="url(#gradient1)" stroke="#2d3436" stroke-width="2" />
                
                <rect x="50" y="200" width="100" height="60" fill="url(#pattern1)" />
                
                <polyline points="200,200 250,220 300,200 350,240 380,200" 
                          stroke="#0984e3" stroke-width="4" fill="none" stroke-linecap="round" />
            </svg>
        </div>
    };

    // SVG with text and transformations
    let text_svg = html! {
        <div class="text-container">
            <h2>{"SVG Text and Transformations"}</h2>
            <svg width="400" height="250" viewBox="0 0 400 250">
                <g transform="translate(50, 50)">
                    <text x="0" y="0" font-family="Arial" font-size="24" fill="#2d3436">
                        {"SVG Text Example"}
                    </text>
                    <text x="0" y="30" font-family="serif" font-size="16" fill="#636e72">
                        <tspan x="0" dy="0">{"This is a"}</tspan>
                        <tspan x="0" dy="20" font-weight="bold" fill="#e17055">{"multi-line"}</tspan>
                        <tspan x="0" dy="20" font-style="italic" fill="#00b894">{"text example"}</tspan>
                    </text>
                </g>
                
                <g transform="translate(200, 150) rotate(45)">
                    <rect x="-50" y="-10" width="100" height="20" fill="#fdcb6e" />
                    <text x="0" y="5" text-anchor="middle" font-size="14" fill="#2d3436">
                        {"Rotated Text"}
                    </text>
                </g>
                
                <path id="curve" d="M 50 200 Q 200 150 350 200" stroke="none" fill="none" />
                <text font-size="16" fill="#6c5ce7">
                    <textPath href="#curve">
                        {"Text following a curved path"}
                    </textPath>
                </text>
            </svg>
        </div>
    };

    // SVG with animations
    let animated_svg = html! {
        <div class="animation-container">
            <h2>{"SVG Animations"}</h2>
            <svg width="300" height="200" viewBox="0 0 300 200">
                <circle cx="50" cy="100" r="20" fill="#e17055">
                    <animate attributeName="cx" from="50" to="250" dur="3s" repeatCount="indefinite" />
                    <animate attributeName="fill" values="#e17055;#00b894;#0984e3;#e17055" dur="3s" repeatCount="indefinite" />
                </circle>
                
                <rect x="100" y="50" width="40" height="40" fill="#6c5ce7">
                    <animateTransform 
                        attributeName="transform" 
                        type="rotate" 
                        values="0 120 70;360 120 70" 
                        dur="2s" 
                        repeatCount="indefinite" />
                </rect>
                
                <polygon points="200,150 230,150 240,120 210,120" fill="#fd79a8">
                    <animateTransform 
                        attributeName="transform" 
                        type="scale" 
                        values="1;1.5;1" 
                        dur="1.5s" 
                        repeatCount="indefinite" />
                </polygon>
            </svg>
        </div>
    };

    // SVG with filters and effects
    let filtered_svg = html! {
        <div class="filter-container">
            <h2>{"SVG Filters and Effects"}</h2>
            <svg width="400" height="300" viewBox="0 0 400 300">
                <defs>
                    <filter id="blur" x="-50%" y="-50%" width="200%" height="200%">
                        <feGaussianBlur stdDeviation="3" />
                    </filter>
                    
                    <filter id="shadow" x="-50%" y="-50%" width="200%" height="200%">
                        <feOffset dx="3" dy="3" />
                        <feGaussianBlur stdDeviation="2" />
                        <feColorMatrix type="matrix" values="0 0 0 0.3 0 0 0 0 0.3 0 0 0 0 0.3 0 0 0 0 1 0" />
                    </filter>
                    
                    <filter id="glow" x="-50%" y="-50%" width="200%" height="200%">
                        <feGaussianBlur stdDeviation="4" result="coloredBlur" />
                        <feMerge>
                            <feMergeNode in="coloredBlur" />
                            <feMergeNode in="SourceGraphic" />
                        </feMerge>
                    </filter>
                </defs>
                
                <rect x="50" y="50" width="80" height="60" fill="#74b9ff" filter="url(#blur)" />
                <rect x="150" y="50" width="80" height="60" fill="#fd79a8" filter="url(#shadow)" />
                <circle cx="320" cy="80" r="30" fill="#00b894" filter="url(#glow)" />
                
                <text x="200" y="200" font-size="32" font-weight="bold" fill="#2d3436" filter="url(#shadow)">
                    {"Filtered Text"}
                </text>
            </svg>
        </div>
    };

    // Dynamic SVG content
    let dynamic_content = create_dynamic_svg();

    println!("SVG Examples Generated Successfully!");
    println!("Basic SVG: {}", basic_svg);
    println!("Path SVG: {}", path_svg);
    println!("Text SVG: {}", text_svg);
    println!("Animated SVG: {}", animated_svg);
    println!("Filtered SVG: {}", filtered_svg);
    println!("Dynamic SVG: {}", dynamic_content);
}

fn create_dynamic_svg() -> String {
    let width = 300;
    let height = 200;
    let points = vec![(50, 50), (100, 30), (150, 60), (200, 40), (250, 70)];
    let colors = vec!["#ff7675", "#74b9ff", "#55a3ff", "#00b894", "#fdcb6e"];
    
    let dynamic_svg = html! {
        <div class="dynamic-container">
            <h2>{"Dynamic SVG Content"}</h2>
            <svg width={width} height={height} viewBox={format!("0 0 {} {}", width, height)}>
                <rect x="0" y="0" width="100%" height="100%" fill="#f8f9fa" stroke="#dee2e6" />
                
                {for (i, (x, y)) in points.iter().enumerate() {
                    <g>
                        <circle 
                            cx={x} 
                            cy={y} 
                            r="15" 
                            fill={colors.get(i).unwrap_or(&"#2d3436")} 
                        />
                        <text 
                            x={x} 
                            y={y + 5} 
                            text-anchor="middle" 
                            font-size="12" 
                            fill="white"
                        >
                            {i + 1}
                        </text>
                    </g>
                }}
                
                <polyline 
                    points={points.iter().map(|(x, y)| format!("{},{}", x, y)).collect::<Vec<_>>().join(" ")}
                    stroke="#6c5ce7" 
                    stroke-width="2" 
                    fill="none" 
                />
                
                <text x={width / 2} y={height - 20} text-anchor="middle" font-size="14" fill="#2d3436">
                    {format!("Generated with {} points", points.len())}
                </text>
            </svg>
        </div>
    };

    dynamic_svg.to_string()
}