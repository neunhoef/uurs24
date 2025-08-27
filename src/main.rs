mod data;
mod optimize;
mod plot;
mod server;

use clap::Command;
use data::{build_regatta_graph, load_regatta_data};
use optimize::{estimate_leg_performance, explore_paths};
use plot::save_regatta_plot;

#[tokio::main]
async fn main() {
    let matches = Command::new("uurs24")
        .about("24-hour regatta data management tool")
        .version("1.0")
        .subcommand_negates_reqs(true)
        .subcommand(Command::new("show").about("Show regatta data and statistics"))
        .subcommand(
            Command::new("plot")
                .about("Generate SVG visualization of the regatta course")
                .arg(
                    clap::Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output SVG file path (default: regatta_course.svg)")
                        .default_value("regatta_course.svg"),
                ),
        )
        .subcommand(
            Command::new("graph")
                .about("Export the regatta graph to a DOT file for graphviz")
                .arg(
                    clap::Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("FILE")
                        .help("Output DOT file path (default: regatta_graph.dot)")
                        .default_value("regatta_graph.dot"),
                ),
        )
        .subcommand(
            Command::new("estimate")
                .about("Estimate boat performance between two buoys at a specific time")
                .arg(
                    clap::Arg::new("from")
                        .help("Name of the starting buoy")
                        .required(true),
                )
                .arg(
                    clap::Arg::new("to")
                        .help("Name of the destination buoy")
                        .required(true),
                )
                .arg(
                    clap::Arg::new("time")
                        .help("Time in hours after race start")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("serve")
                .about("Start HTTP server to serve regatta data")
                .arg(
                    clap::Arg::new("port")
                        .short('p')
                        .long("port")
                        .value_name("PORT")
                        .help("Port to bind the server to (default: 3030)")
                        .default_value("3030"),
                ),
        )
        .subcommand(
            Command::new("paths")
                .about("Explore all possible paths from a starting point")
                .arg(
                    clap::Arg::new("start")
                        .help("Name of the starting buoy")
                        .required(true),
                )
                .arg(
                    clap::Arg::new("time")
                        .help("Starting time in hours after race start")
                        .required(true),
                )
                .arg(
                    clap::Arg::new("steps")
                        .help("Number of steps to explore")
                        .required(true),
                ),
        )
        .get_matches();

    // Load data for every subcommand
    println!("Loading regatta data...");

    let data = match load_regatta_data() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error loading regatta data: {e}");
            std::process::exit(1);
        }
    };

    match matches.subcommand() {
        Some(("show", _)) => {
            show_regatta_data(&data);
        }
        Some(("plot", plot_matches)) => {
            let output_path = plot_matches.get_one::<String>("output").unwrap();
            match save_regatta_plot(&data, output_path, None) {
                Ok(()) => println!("Successfully generated SVG plot!"),
                Err(e) => {
                    eprintln!("Error generating SVG plot: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some(("graph", graph_matches)) => {
            let output_path = graph_matches.get_one::<String>("output").unwrap();
            match export_regatta_graph(&data, output_path) {
                Ok(()) => println!("Successfully exported graph to DOT file: {output_path} and generated PDF: regatta_graph.pdf"),
                Err(e) => {
                    eprintln!("Error exporting graph to DOT file: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some(("estimate", estimate_matches)) => {
            let from_name = estimate_matches.get_one::<String>("from").unwrap();
            let to_name = estimate_matches.get_one::<String>("to").unwrap();
            let time_str = estimate_matches.get_one::<String>("time").unwrap();
            
            match time_str.parse::<f64>() {
                Ok(time) => {
                    match estimate_leg_performance_command(&data, from_name, to_name, time) {
                        Ok(()) => {},
                        Err(e) => {
                            eprintln!("Error estimating leg performance: {e}");
                            std::process::exit(1);
                        }
                    }
                }
                Err(_) => {
                    eprintln!("Error: time must be a valid number");
                    std::process::exit(1);
                }
            }
        }
        Some(("serve", serve_matches)) => {
            let port_str = serve_matches.get_one::<String>("port").unwrap();
            match port_str.parse::<u16>() {
                Ok(port) => {
                    println!("Starting HTTP server on port {}...", port);
                    if let Err(e) = server::start_server(data, port).await {
                        eprintln!("Error starting server: {e}");
                        std::process::exit(1);
                    }
                }
                Err(_) => {
                    eprintln!("Error: port must be a valid number between 1 and 65535");
                    std::process::exit(1);
                }
            }
        }
        Some(("paths", paths_matches)) => {
            let start_name = paths_matches.get_one::<String>("start").unwrap();
            let time_str = paths_matches.get_one::<String>("time").unwrap();
            let steps_str = paths_matches.get_one::<String>("steps").unwrap();
            
            match (time_str.parse::<f64>(), steps_str.parse::<usize>()) {
                (Ok(time), Ok(steps)) => {
                    match explore_paths_command(&data, start_name, time, steps) {
                        Ok(()) => {},
                        Err(e) => {
                            eprintln!("Error exploring paths: {e}");
                            std::process::exit(1);
                        }
                    }
                }
                (Err(_), _) => {
                    eprintln!("Error: time must be a valid number");
                    std::process::exit(1);
                }
                (_, Err(_)) => {
                    eprintln!("Error: steps must be a valid positive integer");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            // Default behavior when no subcommand is provided
            show_regatta_data(&data);
        }
    }
}

fn show_regatta_data(data: &data::RegattaData) {
    println!("Successfully loaded regatta data:");
    println!("  - {} buoys (boeien)", data.boeien.len());
    println!("  - {} start lines", data.starts.len());
    println!("  - {} legs (rakken)", data.rakken.len());

    // Find and display the FINISH buoy
    if let Some(finish_boei) = data.get_boei("FINISH") {
        println!("\nFINISH buoy details:");
        println!("  Name: {}", finish_boei.name);
        if let Some(buoy_type) = &finish_boei.buoy_type {
            println!("  Type: {buoy_type}");
        }
        if let Some(lat) = &finish_boei.lat {
            println!("  Latitude: {lat:.6}° (decimal)");
        }
        if let Some(long) = &finish_boei.long {
            println!("  Longitude: {long:.6}° (decimal)");
        }
        if let Some(lat_str) = &finish_boei.lat_min {
            println!("  Latitude (original): {lat_str}");
        }
        if let Some(long_str) = &finish_boei.long_min {
            println!("  Longitude (original): {long_str}");
        }

        // Demonstrate the convenience methods
        if finish_boei.has_coordinates() {
            if let Some((lat, long)) = finish_boei.coordinates() {
                println!("  Coordinates tuple: ({lat:.6}, {long:.6})");
            }
        }
    }

    // Show all boeien with coordinates
    println!("\nAll boeien with geo coordinates:");
    let mut buoys_with_coords = 0;
    let mut buoys_without_coords = 0;
    
    for boei in &data.boeien {
        if boei.has_coordinates() {
            if let Some((lat, long)) = boei.coordinates() {
                let buoy_type_str = boei.buoy_type.as_ref().map(|t| format!(" ({})", t)).unwrap_or_default();
                println!("  {}{} - {:.6}°N, {:.6}°E", boei.name, buoy_type_str, lat, long);
                buoys_with_coords += 1;
            }
        } else {
            buoys_without_coords += 1;
        }
    }
    
    if buoys_without_coords > 0 {
        println!("  Note: {} boeien have no coordinate data", buoys_without_coords);
    }
    println!("  Total: {} boeien with coordinates, {} without", buoys_with_coords, buoys_without_coords);

    // Show start lines
    println!("\nStart lines:");
    for start in data.get_starts().iter() {
        println!("  {} -> {} ({} nm)", start.from, start.to, start.distance);
    }

    // Show legs
    println!("\nLegs (rakken):");
    for rak in data.get_rakken().iter() {
        println!("  {} -> {} ({} nm)", rak.from, rak.to, rak.distance);
    }

    // Show complete polar data
    let polar_data = data.get_polar_data();
    println!("\nComplete Polar Performance Data:");
    println!("Wind speeds: {:?} knots", polar_data.wind_speeds);
    println!(
        "True Wind Angles (TWA): {:?} degrees",
        polar_data.wind_angles
    );
    println!();

    // Print header row with wind speeds
    print!("TWA/TWS\t");
    for &wind_speed in &polar_data.wind_speeds {
        print!("{wind_speed:>6.0}kt\t");
    }
    println!();

    // Print separator line
    print!("--------");
    for _ in &polar_data.wind_speeds {
        print!("!--------------");
    }
    println!();

    // Print each row with wind angle and boat speeds
    for (angle_idx, &wind_angle) in polar_data.wind_angles.iter().enumerate() {
        print!("{wind_angle:>6.0}°\t");
        for &boat_speed in &polar_data.boat_speeds[angle_idx] {
            print!("{boat_speed:>6.2}kt\t");
        }
        println!();
    }

    println!();
    println!("Note: TWA = True Wind Angle (0° = head to wind, 90° = beam reach, 180° = downwind)");
    println!("      Boat speeds are in knots (kt)");

    // Show wind data
    let wind_data = data.get_wind_data();
    println!("\nWind Conditions During Race:");
    println!("Time (hrs) | Wind Speed (kts) | Wind Direction (°)");
    println!("-----------|------------------|-------------------");
    for condition in wind_data.get_all_conditions() {
        println!(
            "     {:2}    |        {:5.1}      |        {:5.1}",
            condition.time, condition.wind_speed, condition.wind_angle
        );
    }
    println!();
    println!("Note: Wind direction is the angle FROM which the wind is coming");
    println!("      (180° = southerly wind, 0° = northerly wind)");

    // Demonstrate wind interpolation
    println!("\nWind Interpolation Examples:");
    println!("Time (hrs) | Wind Speed (kts) | Wind Direction (°) | Notes");
    println!("-----------|------------------|-------------------|-------");

    // Show some interpolated values
    let test_times = vec![0.0, 0.5, 1.0, 1.5, 2.0, 23.5, 24.0];
    for time in test_times {
        if let Some(wind) = wind_data.get_wind_at_time(time) {
            let notes = if time == time.floor() {
                "Exact hour"
            } else {
                "Interpolated"
            };
            println!(
                "     {:4.1}    |        {:5.1}      |        {:5.1}      | {}",
                time, wind.wind_speed, wind.wind_angle, notes
            );
        }
    }

    // Show buoys by type
    let start_boeien = data.get_boeien_by_type("Startboei");
    println!("\nStart buoys ({} found):", start_boeien.len());
    for boei in start_boeien.iter() {
        println!(
            "  {}: {}",
            boei.name,
            boei.description
                .as_ref()
                .unwrap_or(&"No description".to_string())
        );
    }

    // Create the petgraph from the loaded data
    println!("\nBuilding regatta graph...");
    let (graph, _node_indices) = build_regatta_graph(data);

    println!("Graph created successfully:");
    println!("  - {} nodes (boeien)", graph.node_count());
    println!("  - {} edges (starts + rakken)", graph.edge_count());

    // Show nodes and their types:
    println!("\nNodes in the graph:");
    for (i, node_weight) in graph.node_weights().enumerate() {
        let boei = &data.boeien[i];
        let no_type_str = "No type".to_string();
        let node_type = node_weight.as_ref().unwrap_or(&no_type_str);
        println!("  {}: {} (type: {})", i, boei.name, node_type);
    }

    // Show edges with their properties:
    println!("\nEdges in the graph:");
    for edge_idx in graph.edge_indices() {
        let (source, target) = graph.edge_endpoints(edge_idx).unwrap();
        let sname = &data.boeien[source.index()].name[..];
        let tname = &data.boeien[target.index()].name[..];
        let edge_weight = graph.edge_weight(edge_idx).unwrap();
        println!(
            "  {} -> {}: distance={:.2} nm, is_start={}, forwards={}, index={}",
            sname,
            tname,
            edge_weight.distance,
            edge_weight.is_start,
            edge_weight.forwards,
            edge_weight.index,
        );
    }
}

/// Estimate leg performance between two buoys at a specific time
fn estimate_leg_performance_command(
    data: &data::RegattaData,
    from_name: &str,
    to_name: &str,
    time: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Find the buoys by name
    let from_boei = data.get_boei(from_name)
        .ok_or_else(|| format!("Buoy '{}' not found", from_name))?;
    let to_boei = data.get_boei(to_name)
        .ok_or_else(|| format!("Buoy '{}' not found", to_name))?;
    
    // Get the indices of the buoys
    let from_index = data.get_boei_index(from_name)
        .ok_or_else(|| format!("Buoy '{}' not found in index", from_name))?;
    let to_index = data.get_boei_index(to_name)
        .ok_or_else(|| format!("Buoy '{}' not found in index", to_name))?;
    
    // Check if both buoys have coordinates
    if !from_boei.has_coordinates() || !to_boei.has_coordinates() {
        return Err("Both buoys must have valid coordinates".into());
    }
    
    // Estimate the leg performance
    let performance = estimate_leg_performance(data, from_index, to_index, time);
    
    // Print the results
    println!("Leg Performance Estimate:");
    println!("  From: {} ({})", from_name, from_boei.buoy_type.as_ref().unwrap_or(&"Unknown".to_string()));
    println!("  To:   {} ({})", to_name, to_boei.buoy_type.as_ref().unwrap_or(&"Unknown".to_string()));
    println!("  Time: {:.1} hours after race start", time);
    println!();
    println!("Results:");
    println!("  Estimated Speed: {:.2} knots", performance.estimated_speed);
    println!("  Course Bearing:  {:.1}°", performance.course_bearing);
    println!("  Wind Direction:  {:.1}°", performance.wind_direction);
    println!("  Relative Bearing: {:.1}°", performance.relative_bearing);
    println!("  Wind Speed:      {:.1} knots", performance.wind_speed);
    
    // Add some interpretation
    println!();
    println!("Interpretation:");
    if performance.relative_bearing < 45.0 {
        println!("  Sailing close-hauled (into the wind)");
    } else if performance.relative_bearing < 90.0 {
        println!("  Sailing on a close reach");
    } else if performance.relative_bearing < 135.0 {
        println!("  Sailing on a beam reach");
    } else if performance.relative_bearing < 180.0 {
        println!("  Sailing on a broad reach");
    } else {
        println!("  Sailing downwind");
    }
    
    Ok(())
}

/// Explore all possible paths from a starting buoy
fn explore_paths_command(
    data: &data::RegattaData,
    start_name: &str,
    start_time: f64,
    num_steps: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Find the starting buoy by name
    let start_boei = data.get_boei(start_name)
        .ok_or_else(|| format!("Starting buoy '{}' not found", start_name))?;
    
    // Get the index of the starting buoy
    let start_index = data.get_boei_index(start_name)
        .ok_or_else(|| format!("Starting buoy '{}' not found in index", start_name))?;
    
    println!("Exploring paths from: {} ({})", 
        start_name, 
        start_boei.buoy_type.as_ref().unwrap_or(&"Unknown".to_string())
    );
    println!("Starting time: {:.1} hours after race start", start_time);
    println!("Number of steps: {}", num_steps);
    println!();
    
    // Explore all possible paths
    let paths = explore_paths(data, start_index, start_time, num_steps)?;
    
    if paths.is_empty() {
        println!("No paths found from this starting point.");
        return Ok(());
    }
    
    println!("Found {} possible path(s):", paths.len());
    println!();
    
    // Sort paths by end time for better readability
    let mut sorted_paths = paths;
    sorted_paths.sort_by(|a, b| a.end_time.partial_cmp(&b.end_time).unwrap_or(std::cmp::Ordering::Equal));
    
    // Print each path
    for (i, path) in sorted_paths.iter().enumerate() {
        println!("Path {} (Total: {:.2} nm, End time: {:.2} hours):", 
            i + 1, path.total_distance, path.end_time);
        
        // Print each step in the path
        for (j, step) in path.steps.iter().enumerate() {
            let from_name = &data.boeien[step.from].name;
            let to_name = &data.boeien[step.to].name;
            
            println!("  Step {}: {} -> {} ({:.2} nm, {:.2} kts, {:.2}h -> {:.2}h)", 
                j + 1,
                from_name,
                to_name,
                step.distance,
                step.speed,
                step.start_time,
                step.end_time
            );
        }
        println!();
    }
    
    // Print summary statistics
    if !sorted_paths.is_empty() {
        let fastest_path = &sorted_paths[0];
        let slowest_path = &sorted_paths[sorted_paths.len() - 1];
        let avg_end_time: f64 = sorted_paths.iter().map(|p| p.end_time).sum::<f64>() / sorted_paths.len() as f64;
        let avg_distance: f64 = sorted_paths.iter().map(|p| p.total_distance).sum::<f64>() / sorted_paths.len() as f64;
        
        println!("Summary:");
        println!("  Fastest path: {:.2} hours", fastest_path.end_time);
        println!("  Slowest path: {:.2} hours", slowest_path.end_time);
        println!("  Average end time: {:.2} hours", avg_end_time);
        println!("  Average distance: {:.2} nm", avg_distance);
    }
    
    Ok(())
}

/// Export the regatta graph to a DOT file for graphviz visualization and generate PDF
fn export_regatta_graph(
    data: &data::RegattaData,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build the regatta graph
    let (graph, node_indices) = build_regatta_graph(data);

    // Create the DOT file content
    let mut dot_content = String::new();
    dot_content.push_str("digraph RegattaGraph {\n");
    dot_content.push_str("  // Graph settings\n");
    dot_content.push_str("  rankdir=LR;\n");
    dot_content.push_str("  node [shape=box, style=filled, fillcolor=lightblue];\n");
    dot_content.push_str("  edge [fontsize=10];\n\n");

    // Add nodes
    dot_content.push_str("  // Nodes (Buoys)\n");
    for (boei_name, &node_idx) in &node_indices {
        let node_weight = graph.node_weight(node_idx).unwrap();
        let node_type = node_weight.as_ref().map_or("Unknown", |s| s.as_str());

        // Color nodes based on type
        let fillcolor = if node_type == "Startboei" {
            "lightgreen"
        } else if node_type == "Finishboei" {
            "red"
        } else if node_type == "Merkboei" {
            "yellow"
        } else {
            "lightblue"
        };

        dot_content.push_str(&format!(
            "  \"{boei_name}\" [label=\"{boei_name}\\n({node_type})\", fillcolor={fillcolor}];\n",
        ));
    }

    dot_content.push_str("\n  // Edges (Starts and Legs)\n");

    // Add edges for starts
    for start in data.get_starts() {
        if let (Some(&_from_idx), Some(&_to_idx)) =
            (node_indices.get(&start.from), node_indices.get(&start.to))
        {
            dot_content.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"Start: {:.2}nm\", color=green, style=bold];\n",
                start.from, start.to, start.distance
            ));
        }
    }

    // Add edges for rakken (legs)
    for rak in data.get_rakken() {
        if let (Some(&_from_idx), Some(&_to_idx)) =
            (node_indices.get(&rak.from), node_indices.get(&rak.to))
        {
            dot_content.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"Leg: {:.2}nm\", color=blue];\n",
                rak.from, rak.to, rak.distance
            ));
        }
    }

    dot_content.push_str("}\n");

    // Write the DOT file
    std::fs::write(output_path, dot_content)?;

    // Generate PDF from the DOT file using graphviz
    println!("Generating PDF from DOT file...");
    let pdf_output = "regatta_graph.pdf";
    
    let output = std::process::Command::new("dot")
        .args(&["-Tpdf", output_path, "-o", pdf_output])
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                println!("Successfully generated PDF: {}", pdf_output);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Warning: Failed to generate PDF: {}", stderr);
                eprintln!("Note: Make sure 'dot' (graphviz) is installed on your system");
            }
        }
        Err(e) => {
            eprintln!("Warning: Could not execute 'dot' command: {}", e);
            eprintln!("Note: Make sure 'dot' (graphviz) is installed on your system");
        }
    }

    Ok(())
}
