use crate::data::{RegattaData, build_regatta_graph};
use petgraph::visit::EdgeRef;

#[derive(Clone)]
pub struct Step {
    pub from: usize,
    pub to: usize,
    pub edge: usize,
    pub distance: f64,   // in nm
    pub speed: f64,      // in knots, estimated by wind and bearing
    pub start_time: f64, // in hours since race start
    pub end_time: f64,   // in hours since race start
}

#[derive(Clone)]
pub struct Path {
    pub steps: Vec<Step>,
    pub edges_used: Vec<u8>, // always length of number of edges
    pub total_distance: f64, // total distance in nm
    pub end_time: f64,       // end time in hours
}

/// Detailed performance estimation for a leg between two buoys
pub struct LegPerformance {
    pub estimated_speed: f64,      // in knots
    pub course_bearing: f64,       // bearing of the course in degrees
    pub wind_direction: f64,       // wind direction in degrees
    pub relative_bearing: f64,     // bearing relative to wind in degrees
    pub wind_speed: f64,           // wind speed in knots
}

/// Estimate the performance for a leg between two buoys at a specific time
pub fn estimate_leg_performance(
    data: &RegattaData,
    from: usize, // index of vertex in graph resp. Boei in data
    to: usize,   // index of vertex in graph resp. Boei in data
    time: f64,
) -> LegPerformance {
    // We proceed as follows:
    //  - compute the initial bearing of the edge
    //  - lookup the wind estimate for the given time
    //  - compute the bearing in relation to the wind
    //  - use the polar table to estimate the speed
    //  - if we are sailing into the wind we have to beat and
    //    the resulting speed is much smaller.

    // Compute initial bearing of the edge:
    let (source, target) = (&data.boeien[from], &data.boeien[to]);
    let s_lat = source.lat.unwrap() * std::f64::consts::PI / 180.0;
    let s_lon = source.long.unwrap() * std::f64::consts::PI / 180.0;
    let t_lat = target.lat.unwrap() * std::f64::consts::PI / 180.0;
    let t_lon = target.long.unwrap() * std::f64::consts::PI / 180.0;
    let d_lon = t_lon - s_lon;
    let course_bearing = (d_lon.sin() * t_lat.cos())
        .atan2(s_lat.cos() * t_lat.sin() - s_lat.sin() * t_lat.cos() * d_lon.cos())
        * 180.0
        / std::f64::consts::PI;

    // Normalize bearing to 0-360 range
    let course_bearing = (course_bearing + 360.0) % 360.0;

    // Lookup the wind estimate for the given time:
    let wind = data.wind_data.get_wind_at_time(time)
        .unwrap_or_else(|| {
            // Fallback: use the closest available hour
            let hour = time.floor().clamp(0.0, 24.0) as u32;
            data.wind_data.get_wind_at_hour(hour)
                .or_else(|| data.wind_data.get_wind_at_hour(0)) // Final fallback to hour 0
                .unwrap()
                .clone()
        });
    let wind_direction = wind.wind_angle;
    let wind_speed = wind.wind_speed;

    // Compute the bearing in relation to the wind:
    let mut relative_bearing = (wind_direction - course_bearing).abs(); // -360 < relative_bearing < 360
    if relative_bearing > 180.0 {
        relative_bearing = 360.0 - relative_bearing;
    }

    let estimated_speed = data.polar_data
        .get_boat_speed(relative_bearing, wind_speed);

    LegPerformance {
        estimated_speed,
        course_bearing,
        wind_direction,
        relative_bearing,
        wind_speed,
    }
}

/// Legacy function for backward compatibility
pub fn estimate_speed(
    data: &RegattaData,
    from: usize,
    to: usize,
    time: f64,
) -> f64 {
    estimate_leg_performance(data, from, to, time).estimated_speed
}

/// Explore all possible paths from a starting point with a given number of steps
pub fn explore_paths(
    data: &RegattaData,
    start_point: usize,    // index of the starting buoy
    start_time: f64,       // time in hours since race start
    num_steps: usize,      // number of steps to explore
) -> Result<Vec<Path>, Box<dyn std::error::Error>> {
    // Build the regatta graph
    let (graph, _node_indices) = build_regatta_graph(data);
    
    if start_point >= data.boeien.len() {
        return Err(format!("Invalid start point index: {}", start_point).into());
    }
    
    let mut all_paths = Vec::new();
    let initial_edges_used = vec![0u8; data.starts.len() + data.rakken.len()];
    
    // Start the recursive exploration
    explore_paths_recursive(
        data,
        &graph,
        start_point,
        start_time,
        num_steps,
        Vec::new(),
        initial_edges_used,
        0.0, // initial total distance
        &mut all_paths,
    )?;
    
    Ok(all_paths)
}

/// Recursive helper function for path exploration
fn explore_paths_recursive(
    data: &RegattaData,
    graph: &petgraph::Graph<Option<String>, crate::data::RegattaEdge>,
    current_point: usize,
    current_time: f64,
    remaining_steps: usize,
    current_steps: Vec<Step>,
    edges_used: Vec<u8>,
    total_distance: f64,
    all_paths: &mut Vec<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    // If no steps remaining, save the current path
    if remaining_steps == 0 {
        all_paths.push(Path {
            steps: current_steps,
            edges_used,
            total_distance,
            end_time: current_time,
        });
        return Ok(());
    }
    
    // Convert current_point to NodeIndex
    let current_node = petgraph::graph::NodeIndex::new(current_point);
    
    // Explore all neighbors
    for edge_ref in graph.edges(current_node) {
        let edge_weight = edge_ref.weight();
        let target_node = edge_ref.target();
        let target_point = target_node.index();
        
        // Check if we can use this edge (based on max_number constraint)
        let edge_index = if edge_weight.is_start {
            edge_weight.index
        } else {
            // For legs, we need to compute the actual edge index in the combined array
            data.starts.len() + edge_weight.index
        };
        
        // Check if edge has been used too many times
        let max_usage = if edge_weight.is_start {
            data.starts[edge_weight.index].max_number
        } else {
            data.rakken[edge_weight.index].max_number
        };
        
        if edges_used[edge_index] >= max_usage as u8 {
            continue; // Skip this edge if it's been used too many times
        }
        
        // Estimate performance for this leg
        let performance = estimate_leg_performance(data, current_point, target_point, current_time);
        let speed = performance.estimated_speed;
        let distance = edge_weight.distance;
        
        // Calculate time to traverse this edge
        let travel_time = if speed > 0.0 {
            distance / speed // distance in nm, speed in knots, result in hours
        } else {
            // If speed is 0 (shouldn't happen but safety check), use a default slow speed
            distance / 1.0 // 1 knot as fallback
        };
        
        let end_time = current_time + travel_time;
        
        // Create the step
        let step = Step {
            from: current_point,
            to: target_point,
            edge: edge_index,
            distance,
            speed,
            start_time: current_time,
            end_time,
        };
        
        // Update the path and edge usage
        let mut new_steps = current_steps.clone();
        new_steps.push(step);
        
        let mut new_edges_used = edges_used.clone();
        new_edges_used[edge_index] += 1;
        
        // Continue exploring recursively
        explore_paths_recursive(
            data,
            graph,
            target_point,
            end_time,
            remaining_steps - 1,
            new_steps,
            new_edges_used,
            total_distance + distance,
            all_paths,
        )?;
    }
    
    Ok(())
}
