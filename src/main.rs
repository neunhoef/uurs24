mod data;
mod plot;

use clap::Command;
use data::{build_regatta_graph, load_regatta_data};
use plot::save_regatta_plot;

fn main() {
    let matches = Command::new("uurs24")
        .about("24-hour regatta data management tool")
        .version("1.0")
        .subcommand_negates_reqs(true)
        .subcommand(Command::new("show").about("Show regatta data and statistics"))
        .subcommand(Command::new("nop").about("Do nothing (placeholder command)"))
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
        Some(("nop", _)) => {
            // Do nothing as requested
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
                Ok(()) => println!("Successfully exported graph to DOT file: {}", output_path),
                Err(e) => {
                    eprintln!("Error exporting graph to DOT file: {e}");
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
    let (graph, node_indices) = build_regatta_graph(data);

    println!("Graph created successfully:");
    println!("  - {} nodes (boeien)", graph.node_count());
    println!("  - {} edges (starts + rakken)", graph.edge_count());

    // Show some example nodes and their types
    println!("\nNodes in the graph:");
    for (i, node_weight) in graph.node_weights().enumerate() {
        let node_idx = graph.node_indices().nth(i).unwrap();
        let unknown_str = "Unknown".to_string();
        let boei_name = node_indices
            .iter()
            .find(|(_, idx)| **idx == node_idx)
            .map(|(name, _)| name)
            .unwrap_or(&unknown_str);

        let no_type_str = "No type".to_string();
        let node_type = node_weight.as_ref().unwrap_or(&no_type_str);
        println!(
            "  {}: {} (type: {})",
            node_idx.index(),
            boei_name,
            node_type
        );
    }

    // Show some example edges with their properties
    println!("\nEdges in the graph:");
    for edge_idx in graph.edge_indices() {
        let (source, target) = graph.edge_endpoints(edge_idx).unwrap();
        let edge_weight = graph.edge_weight(edge_idx).unwrap();

        let unknown_str = "Unknown".to_string();
        let source_name = node_indices
            .iter()
            .find(|(_, idx)| **idx == source)
            .map(|(name, _)| name)
            .unwrap_or(&unknown_str);
        let target_name = node_indices
            .iter()
            .find(|(_, idx)| **idx == target)
            .map(|(name, _)| name)
            .unwrap_or(&unknown_str);

        println!(
            "  {} -> {}: distance={:.2} nm, speed={:.1}",
            source_name, target_name, edge_weight.distance, edge_weight.speed
        );
    }
}

/// Export the regatta graph to a DOT file for graphviz visualization
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
            "  \"{}\" [label=\"{}\\n({})\", fillcolor={}];\n",
            boei_name, boei_name, node_type, fillcolor
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

    Ok(())
}
