use crate::data::{RegattaData, build_regatta_graph};
use petgraph::visit::EdgeRef;

#[derive(Clone)]
pub struct Step {
    pub from: usize,
    pub to: usize,
    pub distance: f64,   // in nm
    pub speed: f64,      // in knots, estimated by wind and bearing
    pub start_time: f64, // in hours since race start
    pub end_time: f64,   // in hours since race start
}

/// Internal state for path exploration
struct PathExplorationState {
    current_point: usize,
    current_time: f64,
    remaining_steps: usize,
    current_steps: Vec<Step>,
    edges_used: Vec<u8>,
    total_distance: f64,
}

#[derive(Clone)]
pub struct Path {
    pub steps: Vec<Step>,
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



/// Explore all possible paths from a starting point with a given number of steps
pub fn explore_paths(
    data: &RegattaData,
    start_point: usize,    // index of the starting buoy
    start_time: f64,       // time in hours since race start
    num_steps: usize,      // number of steps to explore
    max_paths: Option<usize>, // maximum number of paths to return
) -> Result<Vec<Path>, Box<dyn std::error::Error>> {
    // Build the regatta graph
    let (graph, _node_indices) = build_regatta_graph(data);
    
    if start_point >= data.boeien.len() {
        return Err(format!("Invalid start point index: {start_point}").into());
    }
    
    let mut all_paths = Vec::new();
    let initial_edges_used = vec![0u8; data.starts.len() + data.rakken.len()];
    
    // Start the recursive exploration
    let initial_state = PathExplorationState {
        current_point: start_point,
        current_time: start_time,
        remaining_steps: num_steps,
        current_steps: Vec::new(),
        edges_used: initial_edges_used,
        total_distance: 0.0,
    };
    
    explore_paths_recursive(data, &graph, initial_state, &mut all_paths, max_paths.unwrap_or(usize::MAX))?;
    
    Ok(all_paths)
}

/// Recursive helper function for path exploration
fn explore_paths_recursive(
    data: &RegattaData,
    graph: &petgraph::Graph<Option<String>, crate::data::RegattaEdge>,
    state: PathExplorationState,
    all_paths: &mut Vec<Path>,
    max_paths: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // If no steps remaining, save the current path
    if state.remaining_steps == 0 {
        all_paths.push(Path {
            steps: state.current_steps,
            total_distance: state.total_distance,
            end_time: state.current_time,
        });
        // Exit early if we've reached the maximum number of paths
        if all_paths.len() >= max_paths {
            return Ok(());
        }
        return Ok(());
    }
    
    // Convert current_point to NodeIndex
    let current_node = petgraph::graph::NodeIndex::new(state.current_point);
    
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
        
        if state.edges_used[edge_index] >= max_usage as u8 {
            continue; // Skip this edge if it's been used too many times
        }
        
        // Estimate performance for this leg
        let performance = estimate_leg_performance(data, state.current_point, target_point, state.current_time);
        let speed = performance.estimated_speed;
        let distance = edge_weight.distance;
        
        // Calculate time to traverse this edge
        let travel_time = if speed > 0.0 {
            distance / speed // distance in nm, speed in knots, result in hours
        } else {
            // If speed is 0 (shouldn't happen but safety check), use a default slow speed
            distance / 1.0 // 1 knot as fallback
        };
        
        let end_time = state.current_time + travel_time;
        
        // Create the step
        let step = Step {
            from: state.current_point,
            to: target_point,
            distance,
            speed,
            start_time: state.current_time,
            end_time,
        };
        
        // Update the path and edge usage
        let mut new_steps = state.current_steps.clone();
        new_steps.push(step);
        
        let mut new_edges_used = state.edges_used.clone();
        new_edges_used[edge_index] += 1;
        
        // Create new state for recursive call
        let new_state = PathExplorationState {
            current_point: target_point,
            current_time: end_time,
            remaining_steps: state.remaining_steps - 1,
            current_steps: new_steps,
            edges_used: new_edges_used,
            total_distance: state.total_distance + distance,
        };
        
        // Continue exploring recursively
        explore_paths_recursive(data, graph, new_state, all_paths, max_paths)?;
    }
    
    Ok(())
}

/// Internal state for target path exploration with Rak usage tracking
struct TargetPathExplorationState {
    current_point: usize,
    target_point: usize,
    current_time: f64,
    remaining_steps: usize,
    current_steps: Vec<Step>,
    edges_used: Vec<u8>,
    rak_usage: Vec<u8>,  // Track Rak usage (max 2 per Rak)
    total_distance: f64,
}

/// Explore paths from a starting point to a specific target with Rak usage tracking
pub fn explore_target_paths(
    data: &RegattaData,
    start_point: usize,    // index of the starting buoy
    target_point: usize,   // index of the target buoy
    start_time: f64,       // time in hours since race start
    max_steps: usize,      // maximum number of steps to explore
    max_paths: Option<usize>, // maximum number of paths to return
) -> Result<Vec<Path>, Box<dyn std::error::Error>> {
    // Build the regatta graph
    let (graph, _node_indices) = build_regatta_graph(data);
    
    if start_point >= data.boeien.len() {
        return Err(format!("Invalid start point index: {start_point}").into());
    }
    
    if target_point >= data.boeien.len() {
        return Err(format!("Invalid target point index: {target_point}").into());
    }
    
    let mut all_paths = Vec::new();
    let initial_edges_used = vec![0u8; data.starts.len() + data.rakken.len()];
    let initial_rak_usage = vec![0u8; data.rakken.len()];  // Track Rak usage separately
    
    // Start the recursive exploration
    let initial_state = TargetPathExplorationState {
        current_point: start_point,
        target_point,
        current_time: start_time,
        remaining_steps: max_steps,
        current_steps: Vec::new(),
        edges_used: initial_edges_used,
        rak_usage: initial_rak_usage,
        total_distance: 0.0,
    };
    
    explore_target_paths_recursive(data, &graph, initial_state, &mut all_paths, max_paths.unwrap_or(usize::MAX))?;
    
    Ok(all_paths)
}

/// Recursive helper function for target path exploration with Rak usage tracking
fn explore_target_paths_recursive(
    data: &RegattaData,
    graph: &petgraph::Graph<Option<String>, crate::data::RegattaEdge>,
    state: TargetPathExplorationState,
    all_paths: &mut Vec<Path>,
    max_paths: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // If we reached the target, save the current path
    if state.current_point == state.target_point {
        all_paths.push(Path {
            steps: state.current_steps,
            total_distance: state.total_distance,
            end_time: state.current_time,
        });
        // Exit early if we've reached the maximum number of paths
        if all_paths.len() >= max_paths {
            return Ok(());
        }
        return Ok(());
    }
    
    // If no steps remaining, don't save anything (we didn't reach target)
    if state.remaining_steps == 0 {
        return Ok(());
    }
    
    // Convert current_point to NodeIndex
    let current_node = petgraph::graph::NodeIndex::new(state.current_point);
    
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
        
        if state.edges_used[edge_index] >= max_usage as u8 {
            continue; // Skip this edge if it's been used too many times
        }
        
        // For Rak edges, check if this Rak has been used more than twice
        if !edge_weight.is_start {
            let rak_index = edge_weight.index;
            if state.rak_usage[rak_index] >= 2 {
                continue; // Skip this Rak if it's been used twice already
            }
        }
        
        // Estimate performance for this leg
        let performance = estimate_leg_performance(data, state.current_point, target_point, state.current_time);
        let speed = performance.estimated_speed;
        let distance = edge_weight.distance;
        
        // Calculate time to traverse this edge
        let travel_time = if speed > 0.0 {
            distance / speed // distance in nm, speed in knots, result in hours
        } else {
            // If speed is 0 (shouldn't happen but safety check), use a default slow speed
            distance / 1.0 // 1 knot as fallback
        };
        
        let end_time = state.current_time + travel_time;
        
        // Create the step
        let step = Step {
            from: state.current_point,
            to: target_point,
            distance,
            speed,
            start_time: state.current_time,
            end_time,
        };
        
        // Update the path and edge usage
        let mut new_steps = state.current_steps.clone();
        new_steps.push(step);
        
        let mut new_edges_used = state.edges_used.clone();
        new_edges_used[edge_index] += 1;
        
        // Update Rak usage if this is a Rak edge
        let mut new_rak_usage = state.rak_usage.clone();
        if !edge_weight.is_start {
            let rak_index = edge_weight.index;
            new_rak_usage[rak_index] += 1;
        }
        
        // Create new state for recursive call
        let new_state = TargetPathExplorationState {
            current_point: target_point,
            target_point: state.target_point,
            current_time: end_time,
            remaining_steps: state.remaining_steps - 1,
            current_steps: new_steps,
            edges_used: new_edges_used,
            rak_usage: new_rak_usage,
            total_distance: state.total_distance + distance,
        };
        
        // Continue exploring recursively
        explore_target_paths_recursive(data, graph, new_state, all_paths, max_paths)?;
        
        // Exit early if we've reached the maximum number of paths
        if all_paths.len() >= max_paths {
            return Ok(());
        }
    }
    
    Ok(())
}
