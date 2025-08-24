mod data;

use data::load_regatta_data;

fn main() {
    println!("Loading regatta data...");
    
    match load_regatta_data() {
        Ok(data) => {
            println!("Successfully loaded regatta data:");
            println!("  - {} buoys (boeien)", data.boeien.len());
            println!("  - {} start lines", data.starts.len());
            println!("  - {} legs (rakken)", data.rakken.len());
            
            // Example: Find and display the FINISH buoy
            if let Some(finish_boei) = data.get_boei("FINISH") {
                println!("\nFINISH buoy details:");
                println!("  Name: {}", finish_boei.name);
                if let Some(buoy_type) = &finish_boei.buoy_type {
                    println!("  Type: {}", buoy_type);
                }
                if let Some(lat) = &finish_boei.lat {
                    println!("  Latitude: {:.6}° (decimal)", lat);
                }
                if let Some(long) = &finish_boei.long {
                    println!("  Longitude: {:.6}° (decimal)", long);
                }
                if let Some(lat_str) = &finish_boei.lat_min {
                    println!("  Latitude (original): {}", lat_str);
                }
                if let Some(long_str) = &finish_boei.long_min {
                    println!("  Longitude (original): {}", long_str);
                }
                
                // Demonstrate the convenience methods
                if finish_boei.has_coordinates() {
                    if let Some((lat, long)) = finish_boei.coordinates() {
                        println!("  Coordinates tuple: ({:.6}, {:.6})", lat, long);
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
        }
        Err(e) => {
            eprintln!("Error loading regatta data: {}", e);
            std::process::exit(1);
        }
    }
}
