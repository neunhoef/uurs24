use crate::data::RegattaData;

pub struct Step {
    pub from: usize,
    pub to: usize,
    pub edge: usize,
    pub distance: f64,   // in nm
    pub speed: f64,      // in knots, estimated by wind and bearing
    pub start_time: f64, // in hours since race start
    pub end_time: f64,   // in hours since race start
}

pub struct Path {
    pub steps: Vec<Step>,
    pub edges_used: Vec<u8>, // always length of number of edges
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
