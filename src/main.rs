mod data;

use clap::Command;
use data::{load_regatta_data, build_regatta_graph};

fn main() {
    let matches = Command::new("uurs24")
        .about("24-hour regatta data management tool")
        .version("1.0")
        .subcommand_negates_reqs(true)
        .subcommand(
            Command::new("show")
                .about("Show regatta data and statistics")
        )
        .subcommand(
            Command::new("nop")
                .about("Do nothing (placeholder command)")
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
    
    // Example: Find and display the FINISH buoy
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
    
    // Example: Show some start lines
    println!("\nStart lines:");
    for start in data.get_starts().iter().take(5) {
        println!("  {} -> {} ({} nm)", start.from, start.to, start.distance);
    }
    
    // Example: Show some legs
    println!("\nLegs (rakken):");
    for rak in data.get_rakken().iter().take(5) {
        println!("  {} -> {} ({} nm)", rak.from, rak.to, rak.distance);
    }
    
    // Example: Show buoys by type
    let start_boeien = data.get_boeien_by_type("Startboei");
    println!("\nStart buoys ({} found):", start_boeien.len());
    for boei in start_boeien.iter().take(3) {
        println!("  {}: {}", boei.name, boei.description.as_ref().unwrap_or(&"No description".to_string()));
    }
    
    // Create the petgraph from the loaded data
    println!("\nBuilding regatta graph...");
    let (graph, node_indices) = build_regatta_graph(&data);
    
    println!("Graph created successfully:");
    println!("  - {} nodes (boeien)", graph.node_count());
    println!("  - {} edges (starts + rakken)", graph.edge_count());
    
    // Show some example nodes and their types
    println!("\nExample nodes in the graph:");
    for (i, node_weight) in graph.node_weights().enumerate().take(5) {
        let node_idx = graph.node_indices().nth(i).unwrap();
        let unknown_str = "Unknown".to_string();
        let boei_name = node_indices.iter()
            .find(|(_, idx)| **idx == node_idx)
            .map(|(name, _)| name)
            .unwrap_or(&unknown_str);
        
        let no_type_str = "No type".to_string();
        let node_type = node_weight.as_ref().unwrap_or(&no_type_str);
        println!("  {}: {} (type: {})", node_idx.index(), boei_name, node_type);
    }
    
    // Show some example edges with their properties
    println!("\nExample edges in the graph:");
    for edge_idx in graph.edge_indices().take(5) {
        let (source, target) = graph.edge_endpoints(edge_idx).unwrap();
        let edge_weight = graph.edge_weight(edge_idx).unwrap();
        
        let unknown_str = "Unknown".to_string();
        let source_name = node_indices.iter()
            .find(|(_, idx)| **idx == source)
            .map(|(name, _)| name)
            .unwrap_or(&unknown_str);
        let target_name = node_indices.iter()
            .find(|(_, idx)| **idx == target)
            .map(|(name, _)| name)
            .unwrap_or(&unknown_str);
        
        println!("  {} -> {}: distance={:.2} nm, speed={:.1}", 
            source_name, target_name, edge_weight.distance, edge_weight.speed);
    }
}
