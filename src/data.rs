#[cfg(test)]
use petgraph::Direction;
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::error::Error;

/// Custom deserializer for European decimal format (comma as decimal separator)
fn deserialize_european_float<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.replace(',', ".")
        .parse::<f64>()
        .map_err(serde::de::Error::custom)
}

/// Custom deserializer for integer that might come as string
fn deserialize_int_from_string<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<u32>().map_err(serde::de::Error::custom)
}

/// Represents a buoy (boei) with its properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Boei {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Type")]
    pub buoy_type: Option<String>,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "Lat_min_sec")]
    pub lat_min_sec: Option<String>,
    #[serde(rename = "Long_min_sec)")]
    pub long_min_sec: Option<String>,
    #[serde(rename = "Lat_min")]
    pub lat_min: Option<String>,
    #[serde(rename = "Long_min")]
    pub long_min: Option<String>,

    // Parsed coordinates in decimal degrees
    #[serde(skip)]
    pub lat: Option<f64>,
    #[serde(skip)]
    pub long: Option<f64>,
}

impl Boei {
    /// Parse the coordinate strings and populate the lat/long fields
    pub fn parse_coordinates(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(lat_str) = &self.lat_min {
            self.lat = Some(Self::parse_coordinate_string(lat_str)?);
        }

        if let Some(long_str) = &self.long_min {
            self.long = Some(Self::parse_coordinate_string(long_str)?);
        }

        Ok(())
    }

    /// Get the parsed coordinates as a tuple (latitude, longitude) if both are available
    pub fn coordinates(&self) -> Option<(f64, f64)> {
        match (self.lat, self.long) {
            (Some(lat), Some(long)) => Some((lat, long)),
            _ => None,
        }
    }

    /// Check if coordinates are available
    pub fn has_coordinates(&self) -> bool {
        self.lat.is_some() && self.long.is_some()
    }

    /// Parse a coordinate string in the format "53° 5,020'" or "53° 5' 1.20"" to decimal degrees
    fn parse_coordinate_string(coord_str: &str) -> Result<f64, Box<dyn Error>> {
        // Remove any extra whitespace and quotes
        let coord_str = coord_str.trim().trim_matches('"');

        // Split by degree symbol
        let parts: Vec<&str> = coord_str.split('°').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid coordinate format: {coord_str}").into());
        }

        let degrees_str = parts[0].trim();
        let minutes_part = parts[1].trim();

        // Parse degrees
        let degrees: f64 = degrees_str.parse()?;

        // Parse minutes part - handle both formats
        let minutes: f64;

        // Check if it looks like minutes+seconds format (contains space and single quote)
        if minutes_part.contains(' ') && minutes_part.contains('\'') {
            // Format: "20' 17.64'" (minutes and seconds)
            // Find the position of the first single quote
            if let Some(quote_pos) = minutes_part.find('\'') {
                let minutes_str = minutes_part[..quote_pos].trim();
                let seconds_part = minutes_part[quote_pos + 1..].trim();

                // Remove the trailing single quote from seconds
                let seconds_str = seconds_part.trim_end_matches('\'');

                let minutes_val: f64 = minutes_str.parse()?;
                let seconds_val: f64 = seconds_str.parse()?;

                // Convert to decimal minutes: minutes + seconds/60
                minutes = minutes_val + seconds_val / 60.0;
            } else {
                return Err(format!("Invalid minutes format: {minutes_part}").into());
            }
        } else {
            // Format: "5,020'" (decimal minutes)
            let minutes_str = minutes_part.trim_end_matches('\'');
            minutes = minutes_str.replace(',', ".").parse()?;
        }

        // Convert to decimal degrees: degrees + minutes/60
        let decimal_degrees = degrees + minutes / 60.0;

        Ok(decimal_degrees)
    }
}

/// Represents a start line between two points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Start {
    #[serde(rename = "From")]
    pub from: String,
    #[serde(rename = "To")]
    pub to: String,
    #[serde(rename = "Distance", deserialize_with = "deserialize_european_float")]
    pub distance: f64,
    #[serde(rename = "MaxNumber", deserialize_with = "deserialize_int_from_string")]
    pub max_number: u32,
}

/// Represents a leg (rak) between two points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rak {
    #[serde(rename = "From")]
    pub from: String,
    #[serde(rename = "To")]
    pub to: String,
    #[serde(rename = "Distance", deserialize_with = "deserialize_european_float")]
    pub distance: f64,
    #[serde(rename = "MaxNumber", deserialize_with = "deserialize_int_from_string")]
    pub max_number: u32,
}

/// Represents polar performance data for a boat
#[derive(Debug, Clone)]
pub struct PolarData {
    /// Wind speeds in knots (from the first row)
    pub wind_speeds: Vec<f64>,
    /// True wind angles in degrees (from the first column)
    pub wind_angles: Vec<f64>,
    /// Boat speeds in knots, indexed by [wind_angle_index][wind_speed_index]
    pub boat_speeds: Vec<Vec<f64>>,
}

impl PolarData {
    /// Create a new empty PolarData instance
    pub fn new() -> Self {
        Self {
            wind_speeds: Vec::new(),
            wind_angles: Vec::new(),
            boat_speeds: Vec::new(),
        }
    }

    /// Get boat speed for a given true wind angle and wind speed
    pub fn _get_boat_speed(&self, wind_angle: f64, wind_speed: f64) -> Option<f64> {
        // Find the closest wind angle index
        let angle_idx = self._find_closest_index(&self.wind_angles, wind_angle)?;

        // Find the closest wind speed index
        let speed_idx = self._find_closest_index(&self.wind_speeds, wind_speed)?;

        // Return the boat speed at this intersection
        self.boat_speeds.get(angle_idx)?.get(speed_idx).copied()
    }

    /// Find the index of the closest value in a sorted vector
    fn _find_closest_index(&self, values: &[f64], target: f64) -> Option<usize> {
        if values.is_empty() {
            return None;
        }

        let mut closest_idx = 0;
        let mut closest_diff = (values[0] - target).abs();

        for (idx, &value) in values.iter().enumerate() {
            let diff = (value - target).abs();
            if diff < closest_diff {
                closest_diff = diff;
                closest_idx = idx;
            }
        }

        Some(closest_idx)
    }
}

/// Main data structure containing all loaded data
pub struct RegattaData {
    pub boeien: Vec<Boei>,
    pub starts: Vec<Start>,
    pub rakken: Vec<Rak>,
    pub boeien_by_name: HashMap<String, Boei>,
    pub polar_data: PolarData,
}

impl RegattaData {
    /// Create a new empty RegattaData instance
    pub fn new() -> Self {
        Self {
            boeien: Vec::new(),
            starts: Vec::new(),
            rakken: Vec::new(),
            boeien_by_name: HashMap::new(),
            polar_data: PolarData::new(),
        }
    }

    /// Get a buoy by name
    pub fn get_boei(&self, name: &str) -> Option<&Boei> {
        self.boeien_by_name.get(name)
    }

    /// Get all buoys of a specific type
    pub fn get_boeien_by_type(&self, buoy_type: &str) -> Vec<&Boei> {
        self.boeien
            .iter()
            .filter(|b| b.buoy_type.as_ref().is_some_and(|t| t == buoy_type))
            .collect()
    }

    /// Get all start lines
    pub fn get_starts(&self) -> &[Start] {
        &self.starts
    }

    /// Get all legs
    pub fn get_rakken(&self) -> &[Rak] {
        &self.rakken
    }

    /// Get polar data
    pub fn get_polar_data(&self) -> &PolarData {
        &self.polar_data
    }
}

/// Load all regatta data from CSV files
pub fn load_regatta_data() -> Result<RegattaData, Box<dyn Error>> {
    let mut data = RegattaData::new();

    // Load boeien data
    let mut boeien_reader = csv::Reader::from_path("data/boeien.csv")?;
    for result in boeien_reader.deserialize() {
        let mut boei: Boei = result?;
        boei.parse_coordinates()?;
        data.boeien.push(boei.clone());
        data.boeien_by_name.insert(boei.name.clone(), boei);
    }

    // Load starts data
    let mut starts_reader = csv::Reader::from_path("data/starts.csv")?;
    for result in starts_reader.deserialize() {
        let start: Start = result?;
        data.starts.push(start);
    }

    // Load rakken data
    let mut rakken_reader = csv::Reader::from_path("data/rakken.csv")?;
    for result in rakken_reader.deserialize() {
        let rak: Rak = result?;
        data.rakken.push(rak);
    }

    // Load polar data
    data.polar_data = load_polar_data()?;

    Ok(data)
}

/// Load polar performance data from CSV file
fn load_polar_data() -> Result<PolarData, Box<dyn Error>> {
    let mut polar_data = PolarData::new();

    // Read the CSV file manually since it has a specific format
    let content = std::fs::read_to_string("data/polars.csv")?;
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return Err("Empty polar data file".into());
    }

    // Parse the first line to get wind speeds
    let first_line = lines[0];
    let wind_speed_parts: Vec<&str> = first_line.split(';').collect();

    // Skip the first part (twa/tws) and parse wind speeds
    for part in wind_speed_parts.iter().skip(1) {
        let wind_speed: f64 = part.parse()?;
        polar_data.wind_speeds.push(wind_speed);
    }

    // Parse the remaining lines to get wind angles and boat speeds
    for line in lines.iter().skip(1) {
        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() != wind_speed_parts.len() {
            continue; // Skip malformed lines
        }

        // Parse wind angle (first column)
        let wind_angle: f64 = parts[0].parse()?;
        polar_data.wind_angles.push(wind_angle);

        // Parse boat speeds for this wind angle
        let mut speeds = Vec::new();
        for part in parts.iter().skip(1) {
            let speed: f64 = part.parse()?;
            speeds.push(speed);
        }
        polar_data.boat_speeds.push(speeds);
    }

    Ok(polar_data)
}

/// Edge data for the regatta graph
#[derive(Debug, Clone)]
pub struct RegattaEdge {
    pub distance: f64,
    pub speed: f64,
}

/// Build a directed graph from the regatta data
///
/// Nodes represent boeien (buoys) and store their type.
/// Edges represent:
/// - Starts: directed edges from start boeien to target boeien
/// - Rakken: directed edges in both directions between boeien
///
/// Returns a tuple of (graph, node_indices_by_name) where the HashMap
/// maps boei names to their NodeIndex in the graph.
pub fn build_regatta_graph(
    data: &RegattaData,
) -> (
    DiGraph<Option<String>, RegattaEdge>,
    HashMap<String, NodeIndex>,
) {
    let mut graph = DiGraph::new();
    let mut node_indices = HashMap::new();

    // Add all boeien as nodes
    for boei in &data.boeien {
        let node_idx = graph.add_node(boei.buoy_type.clone());
        node_indices.insert(boei.name.clone(), node_idx);
    }

    // Add edges for starts (from start boeien to target boeien)
    for start in &data.starts {
        if let (Some(&from_idx), Some(&to_idx)) =
            (node_indices.get(&start.from), node_indices.get(&start.to))
        {
            graph.add_edge(
                from_idx,
                to_idx,
                RegattaEdge {
                    distance: start.distance,
                    speed: 0.0, // Speed is set to 0 for now as requested
                },
            );
        }
    }

    // Add edges for rakken (in both directions)
    for rak in &data.rakken {
        if let (Some(&from_idx), Some(&to_idx)) =
            (node_indices.get(&rak.from), node_indices.get(&rak.to))
        {
            // Forward edge
            graph.add_edge(
                from_idx,
                to_idx,
                RegattaEdge {
                    distance: rak.distance,
                    speed: 0.0, // Speed is set to 0 for now as requested
                },
            );

            // Reverse edge
            graph.add_edge(
                to_idx,
                from_idx,
                RegattaEdge {
                    distance: rak.distance,
                    speed: 0.0, // Speed is set to 0 for now as requested
                },
            );
        }
    }

    (graph, node_indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_regatta_data() {
        let result = load_regatta_data();
        assert!(
            result.is_ok(),
            "Failed to load regatta data: {:?}",
            result.err()
        );

        let data = result.unwrap();
        assert!(!data.boeien.is_empty(), "No boeien loaded");
        assert!(!data.starts.is_empty(), "No starts loaded");
        assert!(!data.rakken.is_empty(), "No rakken loaded");
        assert!(
            !data.polar_data.wind_speeds.is_empty(),
            "No polar data loaded"
        );

        // Test that we can find boeien by name
        let finish_boei = data.get_boei("FINISH");
        assert!(finish_boei.is_some(), "FINISH boei not found");

        // Test that we can find boeien by type
        let start_boeien = data.get_boeien_by_type("Startboei");
        assert!(!start_boeien.is_empty(), "No start boeien found");
    }

    #[test]
    fn test_coordinate_parsing() {
        let mut boei = Boei {
            name: "TEST".to_string(),
            buoy_type: Some("Test".to_string()),
            description: Some("Test".to_string()),
            lat_min_sec: None,
            long_min_sec: None,
            lat_min: Some("53° 5,020'".to_string()),
            long_min: Some("5° 20,293'".to_string()),
            lat: None,
            long: None,
        };

        // Test coordinate parsing
        let result = boei.parse_coordinates();
        assert!(
            result.is_ok(),
            "Failed to parse coordinates: {:?}",
            result.err()
        );

        // Verify parsed values
        assert!(boei.lat.is_some(), "Latitude should be parsed");
        assert!(boei.long.is_some(), "Longitude should be parsed");

        // Check specific values: 5° 20,293' = 5 + 20.293/60 = 5.33822... (longitude)
        let expected_long = 5.0 + 20.293 / 60.0;
        assert!(
            (boei.long.unwrap() - expected_long).abs() < 0.001,
            "Longitude parsing incorrect. Expected: {}, Got: {}",
            expected_long,
            boei.long.unwrap()
        );

        // Check latitude: 53° 5,020' = 53 + 5.020/60 = 53.08367... (latitude)
        let expected_lat = 53.0 + 5.020 / 60.0;
        assert!(
            (boei.lat.unwrap() - expected_lat).abs() < 0.001,
            "Latitude parsing incorrect. Expected: {}, Got: {}",
            expected_lat,
            boei.lat.unwrap()
        );
    }

    #[test]
    fn test_coordinate_parsing_edge_cases() {
        // Test with different formats
        let test_cases = vec![
            ("53° 5,020'", 53.0 + 5.020 / 60.0),
            ("0° 0,000'", 0.0),
            ("90° 30,500'", 90.0 + 30.5 / 60.0),
        ];

        for (input, expected) in test_cases {
            let result = Boei::parse_coordinate_string(input);
            assert!(
                result.is_ok(),
                "Failed to parse '{}': {:?}",
                input,
                result.err()
            );
            let parsed = result.unwrap();
            assert!(
                (parsed - expected).abs() < 0.001,
                "Parsing '{}' failed. Expected: {}, Got: {}",
                input,
                expected,
                parsed
            );
        }
    }

    #[test]
    fn test_coordinate_convenience_methods() {
        let mut boei = Boei {
            name: "TEST".to_string(),
            buoy_type: Some("Test".to_string()),
            description: Some("Test".to_string()),
            lat_min_sec: None,
            long_min_sec: None,
            lat_min: Some("53° 5,020'".to_string()),
            long_min: Some("5° 20,293'".to_string()),
            lat: None,
            long: None,
        };

        // Initially no coordinates
        assert!(!boei.has_coordinates());
        assert!(boei.coordinates().is_none());

        // Parse coordinates
        boei.parse_coordinates().unwrap();

        // Now coordinates should be available
        assert!(boei.has_coordinates());
        let coords = boei.coordinates().unwrap();
        // coordinates() returns (latitude, longitude)
        assert!((coords.0 - (53.0 + 5.020 / 60.0)).abs() < 0.001); // latitude
        assert!((coords.1 - (5.0 + 20.293 / 60.0)).abs() < 0.001); // longitude
    }

    #[test]
    fn test_build_regatta_graph() {
        let data = load_regatta_data().unwrap();
        let (graph, node_indices) = build_regatta_graph(&data);

        // Check that all boeien are represented as nodes
        assert_eq!(graph.node_count(), data.boeien.len());

        // Check that we can find nodes by name
        for boei in &data.boeien {
            assert!(node_indices.contains_key(&boei.name));
        }

        // Check that start edges are added
        let start_edges = graph.edge_count();
        assert!(start_edges > 0, "Graph should have edges");

        // Verify that start boeien exist and have outgoing edges
        let start_boeien = data.get_boeien_by_type("Startboei");
        for start_boei in start_boeien {
            if let Some(&node_idx) = node_indices.get(&start_boei.name) {
                let outgoing_edges = graph.edges_directed(node_idx, Direction::Outgoing).count();
                assert!(
                    outgoing_edges > 0,
                    "Start boei {} should have outgoing edges",
                    start_boei.name
                );
            }
        }

        // Check that edge data contains distance and speed
        for edge_idx in graph.edge_indices() {
            let edge_weight = graph.edge_weight(edge_idx).unwrap();
            assert!(
                edge_weight.distance > 0.0,
                "Edge distance should be positive"
            );
            assert_eq!(edge_weight.speed, 0.0, "Edge speed should be 0.0 for now");
        }
    }

    #[test]
    fn test_polar_data_loading() {
        let data = load_regatta_data().unwrap();
        let polar_data = data.get_polar_data();

        // Check that wind speeds are loaded correctly
        let expected_wind_speeds = vec![6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 20.0];
        assert_eq!(polar_data.wind_speeds, expected_wind_speeds);

        // Check that wind angles are loaded correctly
        let expected_wind_angles = vec![0.0, 52.0, 60.0, 75.0, 90.0, 110.0, 120.0, 135.0, 150.0];
        assert_eq!(polar_data.wind_angles, expected_wind_angles);

        // Check that boat speeds are loaded correctly
        assert_eq!(polar_data.boat_speeds.len(), expected_wind_angles.len());
        assert_eq!(polar_data.boat_speeds[0].len(), expected_wind_speeds.len());

        // Check some specific values from the CSV
        // At 52° TWA and 6 knots wind: should be 4.72 knots
        let speed = polar_data.get_boat_speed(52.0, 6.0);
        assert!(speed.is_some());
        assert!((speed.unwrap() - 4.72).abs() < 0.01);

        // At 90° TWA and 10 knots wind: should be 7.19 knots
        let speed = polar_data.get_boat_speed(90.0, 10.0);
        assert!(speed.is_some());
        assert!((speed.unwrap() - 7.19).abs() < 0.01);

        // At 135° TWA and 20 knots wind: should be 8.83 knots
        let speed = polar_data.get_boat_speed(135.0, 20.0);
        assert!(speed.is_some());
        assert!((speed.unwrap() - 8.83).abs() < 0.01);
    }
}
