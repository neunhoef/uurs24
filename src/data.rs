use std::collections::HashMap;
use serde::{Deserialize, Serialize, Deserializer};
use std::error::Error;

/// Custom deserializer for European decimal format (comma as decimal separator)
fn deserialize_european_float<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.replace(',', ".").parse::<f64>().map_err(serde::de::Error::custom)
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
    pub notes: Option<String>,
    #[serde(rename = "Lat_min_sec")]
    pub buoy_type: Option<String>,
    #[serde(rename = "Long_min_sec)")]
    pub lat_min_sec: Option<String>,
    #[serde(rename = "Lat_min")]
    pub long_min_sec: Option<String>,
    #[serde(rename = "Long_min")]
    pub lat_min: Option<String>,
    #[serde(rename = "Description")]
    pub long_min: Option<String>,
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

/// Main data structure containing all loaded data
pub struct RegattaData {
    pub boeien: Vec<Boei>,
    pub starts: Vec<Start>,
    pub rakken: Vec<Rak>,
    pub boeien_by_name: HashMap<String, Boei>,
}

impl RegattaData {
    /// Create a new empty RegattaData instance
    pub fn new() -> Self {
        Self {
            boeien: Vec::new(),
            starts: Vec::new(),
            rakken: Vec::new(),
            boeien_by_name: HashMap::new(),
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
            .filter(|b| b.buoy_type.as_ref().map_or(false, |t| t == buoy_type))
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
}

/// Load all regatta data from CSV files
pub fn load_regatta_data() -> Result<RegattaData, Box<dyn Error>> {
    let mut data = RegattaData::new();
    
    // Load boeien data
    let mut boeien_reader = csv::Reader::from_path("data/boeien.csv")?;
    for result in boeien_reader.deserialize() {
        let boei: Boei = result?;
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

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_regatta_data() {
        let result = load_regatta_data();
        assert!(result.is_ok(), "Failed to load regatta data: {:?}", result.err());
        
        let data = result.unwrap();
        assert!(!data.boeien.is_empty(), "No boeien loaded");
        assert!(!data.starts.is_empty(), "No starts loaded");
        assert!(!data.rakken.is_empty(), "No rakken loaded");
        
        // Test that we can find boeien by name
        let finish_boei = data.get_boei("FINISH");
        assert!(finish_boei.is_some(), "FINISH boei not found");
        
        // Test that we can find boeien by type
        let start_boeien = data.get_boeien_by_type("Startboei");
        assert!(!start_boeien.is_empty(), "No start boeien found");
    }
}
