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

pub fn estimate_speed(
    data: &RegattaData,
    from: usize, // index of vertex in graph resp. Boei in data
    to: usize,   // index of vertex in graph resp. Boei in data
    time: f64,
) -> f64 {
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
    let bearing = (d_lon.sin() * t_lat.cos())
        .atan2(s_lat.cos() * t_lat.sin() - s_lat.sin() * t_lat.cos() * d_lon.cos())
        * 180.0
        / std::f64::consts::PI;

    // Lookup the wind estimate for the given time:
    let hour: u32 = time.floor().clamp(0.0, 24.0) as u32;
    let wind = data.wind_data.get_wind_at_hour(hour).unwrap();

    // Compute the bearing in relation to the wind:
    let mut bearing_relative = (wind.wind_angle - bearing).abs(); // -360 < bearing_relative < 360
    if bearing_relative > 180.0 {
        bearing_relative = 360.0 - bearing_relative;
    }

    data.polar_data
        .get_boat_speed(bearing_relative, wind.wind_speed)
}
