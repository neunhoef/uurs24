use crate::data::RegattaData;
use svg::node::element::{Line, Text, Group, Definitions, Marker, Polygon};
use svg::Document;

/// Plot configuration for the SVG output
pub struct PlotConfig {
    pub width: u32,
    pub height: u32,
    pub margin: f64,
    pub buoy_size: f64,
    pub text_size: f64,
    pub line_width: f64,
}

impl Default for PlotConfig {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            margin: 50.0,
            buoy_size: 4.0,
            text_size: 12.0,
            line_width: 2.0,
        }
    }
}

/// Calculate the bounding box for all coordinates
fn calculate_bounds(data: &RegattaData) -> Option<(f64, f64, f64, f64)> {
    let mut min_lat = f64::INFINITY;
    let mut max_lat = f64::NEG_INFINITY;
    let mut min_long = f64::INFINITY;
    let mut max_long = f64::NEG_INFINITY;
    
    let mut has_coordinates = false;
    
    for boei in &data.boeien {
        if let Some((lat, long)) = boei.coordinates() {
            has_coordinates = true;
            min_lat = min_lat.min(lat);
            max_lat = max_lat.max(lat);
            min_long = min_long.min(long);
            max_long = max_long.max(long);
        }
    }
    
    if has_coordinates {
        Some((min_lat, max_lat, min_long, max_long))
    } else {
        None
    }
}

/// Convert geographic coordinates to SVG coordinates
fn geo_to_svg(
    lat: f64,
    long: f64,
    bounds: (f64, f64, f64, f64),
    config: &PlotConfig,
) -> (f64, f64) {
    let (min_lat, max_lat, min_long, max_long) = bounds;
    
    // Calculate normalized coordinates (0.0 to 1.0)
    let norm_lat = (lat - min_lat) / (max_lat - min_lat);
    let norm_long = (long - min_long) / (max_long - min_long);
    
    // Convert to SVG coordinates with margins
    let x = config.margin + norm_long * (config.width as f64 - 2.0 * config.margin);
    let y = config.margin + (1.0 - norm_lat) * (config.height as f64 - 2.0 * config.margin);
    
    (x, y)
}

/// Create an SVG visualization of the regatta data
pub fn create_regatta_plot(data: &RegattaData, config: PlotConfig) -> Result<String, Box<dyn std::error::Error>> {
    // Calculate bounding box
    let bounds = calculate_bounds(data)
        .ok_or("No coordinates found in the data")?;
    
    let (min_lat, max_lat, min_long, max_long) = bounds;
    
    // Create SVG document
    let mut document = Document::new()
        .set("width", config.width)
        .set("height", config.height)
        .set("viewBox", format!("0 0 {} {}", config.width, config.height));
    
    // Create definitions for arrow markers
    let mut defs = Definitions::new();
    
    // Green arrow marker for start legs
    let green_arrow = Marker::new()
        .set("id", "green-arrow")
        .set("markerWidth", "10")
        .set("markerHeight", "10")
        .set("refX", "8")
        .set("refY", "3")
        .set("orient", "auto")
        .set("markerUnits", "strokeWidth")
        .add(
            Polygon::new()
                .set("points", "0,0 0,6 9,3")
                .set("fill", "green")
        );
    
    defs = defs.add(green_arrow);
    document = document.add(defs);
    
    // Create main group
    let mut main_group = Group::new();
    
    // Draw start legs first (as green arrows)
    for start in &data.starts {
        if let (Some(from_boei), Some(to_boei)) = (data.get_boei(&start.from), data.get_boei(&start.to)) {
            if let (Some((from_lat, from_long)), Some((to_lat, to_long))) = 
                (from_boei.coordinates(), to_boei.coordinates()) {
                
                let (from_x, from_y) = geo_to_svg(from_lat, from_long, bounds, &config);
                let (to_x, to_y) = geo_to_svg(to_lat, to_long, bounds, &config);
                
                // Draw the start leg line with arrow
                let start_line = Line::new()
                    .set("x1", from_x)
                    .set("y1", from_y)
                    .set("x2", to_x)
                    .set("y2", to_y)
                    .set("stroke", "green")
                    .set("stroke-width", config.line_width * 1.5) // Make start legs slightly thicker
                    .set("marker-end", "url(#green-arrow)")
                    .set("opacity", "0.8");
                
                main_group = main_group.add(start_line);
                
                // Add distance label near the center of the start line
                let center_x = (from_x + to_x) / 2.0;
                let center_y = (from_y + to_y) / 2.0 + config.text_size; // Offset to avoid overlap with leg labels
                
                let start_distance_text = Text::new(format!("START: {:.1} nm", start.distance))
                    .set("x", center_x)
                    .set("y", center_y)
                    .set("text-anchor", "middle")
                    .set("dominant-baseline", "middle")
                    .set("font-size", config.text_size * 0.9) // Slightly smaller than leg labels
                    .set("fill", "darkgreen")
                    .set("font-weight", "bold");
                
                main_group = main_group.add(start_distance_text);
            }
        }
    }
    
    // Draw all legs (rakken) second (so they appear over start legs but behind buoys)
    for rak in &data.rakken {
        if let (Some(from_boei), Some(to_boei)) = (data.get_boei(&rak.from), data.get_boei(&rak.to)) {
            if let (Some((from_lat, from_long)), Some((to_lat, to_long))) = 
                (from_boei.coordinates(), to_boei.coordinates()) {
                
                let (from_x, from_y) = geo_to_svg(from_lat, from_long, bounds, &config);
                let (to_x, to_y) = geo_to_svg(to_lat, to_long, bounds, &config);
                
                // Draw the leg line
                let line = Line::new()
                    .set("x1", from_x)
                    .set("y1", from_y)
                    .set("x2", to_x)
                    .set("y2", to_y)
                    .set("stroke", "blue")
                    .set("stroke-width", config.line_width)
                    .set("opacity", "0.7");
                
                main_group = main_group.add(line);
                
                // Add distance label near the center of the line
                let center_x = (from_x + to_x) / 2.0;
                let center_y = (from_y + to_y) / 2.0;
                
                let distance_text = Text::new(format!("{:.1} nm", rak.distance))
                    .set("x", center_x)
                    .set("y", center_y)
                    .set("text-anchor", "middle")
                    .set("dominant-baseline", "middle")
                    .set("font-size", config.text_size)
                    .set("fill", "darkblue")
                    .set("font-weight", "bold");
                
                main_group = main_group.add(distance_text);
            }
        }
    }
    
    // Draw all buoys
    for boei in &data.boeien {
        if let Some((lat, long)) = boei.coordinates() {
            let (x, y) = geo_to_svg(lat, long, bounds, &config);
            
            // Draw buoy as a cross
            let cross_size = config.buoy_size;
            
            // Horizontal line of the cross
            let h_line = Line::new()
                .set("x1", x - cross_size)
                .set("y1", y)
                .set("x2", x + cross_size)
                .set("y2", y)
                .set("stroke", "red")
                .set("stroke-width", 2.0);
            
            // Vertical line of the cross
            let v_line = Line::new()
                .set("x1", x)
                .set("y1", y - cross_size)
                .set("x2", x)
                .set("y2", y + cross_size)
                .set("stroke", "red")
                .set("stroke-width", 2.0);
            
            main_group = main_group.add(h_line);
            main_group = main_group.add(v_line);
            
            // Add buoy name label
            let text_x = x + cross_size + 5.0;
            let text_y = y;
            
            let name_text = Text::new(&boei.name)
                .set("x", text_x)
                .set("y", text_y)
                .set("dominant-baseline", "middle")
                .set("font-size", config.text_size)
                .set("fill", "black");
            
            main_group = main_group.add(name_text);
        }
    }
    
    // Add title and coordinate information
    let title_text = Text::new("24-Hour Regatta Course")
        .set("x", config.width as f64 / 2.0)
        .set("y", 20.0)
        .set("text-anchor", "middle")
        .set("font-size", 16.0)
        .set("font-weight", "bold")
        .set("fill", "black");
    
    let bounds_text = Text::new(format!(
        "Bounds: {min_lat:.4}°N to {max_lat:.4}°N, {min_long:.4}°E to {max_long:.4}°E"
    ))
        .set("x", 10.0)
        .set("y", config.height as f64 - 10.0)
        .set("font-size", 10.0)
        .set("fill", "gray");
    
    main_group = main_group.add(title_text);
    main_group = main_group.add(bounds_text);
    
    // Add main group to document
    document = document.add(main_group);
    
    // Convert to string
    Ok(document.to_string())
}

/// Generate and save the regatta plot to a file
pub fn save_regatta_plot(
    data: &RegattaData,
    output_path: &str,
    config: Option<PlotConfig>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = config.unwrap_or_default();
    let svg_content = create_regatta_plot(data, config)?;
    
    std::fs::write(output_path, svg_content)?;
    println!("SVG plot saved to: {output_path}");
    
    Ok(())
}
