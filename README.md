# Uurs24 - 24-Hour Regatta Data Management Tool

A Rust-based command-line tool for managing and visualizing 24-hour regatta data, including buoys, start lines, legs, and polar performance data.

## Features

- **Data Management**: Load and parse regatta data from CSV files
- **Course Visualization**: Generate SVG plots of the regatta course
- **Performance Analysis**: Display polar performance data for different wind conditions
- **Graph Representation**: Build and analyze regatta course as a directed graph
- **Coordinate Handling**: Parse European coordinate formats (degrees, minutes, seconds)

## Project Structure

```
uurs24/
├── Cargo.toml          # Rust project configuration
├── Cargo.lock          # Dependency lock file
├── data/               # CSV data files
│   ├── boeien.csv      # Buoy definitions and coordinates
│   ├── polars.csv      # Polar performance data
│   ├── rakken.csv      # Course legs between buoys
│   ├── starts.csv      # Start line definitions
│   └── wind.csv        # Wind conditions during the race
├── regatta_course.svg  # Generated course visualization
└── src/
    ├── main.rs         # Main application logic and CLI
    ├── data.rs         # Data structures and parsing
    └── plot.rs         # SVG visualization generation
```

## Installation

### Prerequisites

- Rust 1.70+ and Cargo
- Linux/macOS/Windows

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd uurs24

# Build the project
cargo build --release

# Install globally (optional)
cargo install --path .
```

## Usage

### Basic Commands

```bash
# Show regatta data and statistics (default)
./target/release/uurs24
./target/release/uurs24 show

# Generate SVG visualization of the regatta course
./target/release/uurs24 plot
./target/release/uurs24 plot -o my_course.svg
```

### Command Line Options

- `show`: Display comprehensive regatta data including buoys, start lines, legs, and polar data
- `plot`: Generate SVG visualization with optional output file specification

## Data Format

### Buoys (boeien.csv)
Contains buoy definitions with:
- Name and type
- Geographic coordinates (degrees, minutes, seconds format)
- Description and metadata

### Polar Data (polars.csv)
Performance data for different wind conditions:
- Wind speeds (knots)
- True Wind Angles (TWA)
- Boat speeds for each wind condition

### Legs (rakken.csv)
Course segments between buoys:
- Start and end buoy names
- Distance in nautical miles
- Calculated speed

### Start Lines (starts.csv)
Start line definitions:
- From and to buoy names
- Distance in nautical miles

### Wind Data (wind.csv)
Wind conditions during the race:
- Time in hours (0-24)
- Wind speed in knots
- Wind direction in degrees (angle FROM which wind is coming)
- Supports interpolation between hours for continuous wind data

## Features in Detail

### Coordinate Parsing
- Supports European decimal format (comma as decimal separator)
- Handles degrees, minutes, seconds coordinate format
- Automatic conversion to decimal degrees

### Graph Representation
- Builds directed graph from regatta data
- Nodes represent buoys
- Edges represent legs and start lines
- Enables route analysis and optimization

### Wind Data Management
- Loads hourly wind conditions from CSV
- Supports interpolation between hours for continuous data
- Handles wind direction changes (including 0°/360° transitions)
- Provides easy access to wind conditions at any time during the race

### Visualization
- SVG output with configurable dimensions
- Geographic coordinate mapping
- Buoy markers and course lines
- Customizable styling options

## Dependencies

- **clap**: Command-line argument parsing
- **csv**: CSV file reading and parsing
- **petgraph**: Graph data structures and algorithms
- **serde**: Serialization/deserialization
- **svg**: SVG generation and manipulation

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy
```

### Project Structure

- **`src/main.rs`**: CLI interface and main application logic
- **`src/data.rs`**: Data structures, CSV parsing, and graph building
- **`src/plot.rs`**: SVG visualization generation and coordinate mapping

## Example Output

The tool provides comprehensive output including:

- Buoy count and types
- Start line and leg information
- Polar performance data tables
- Wind conditions during the race (hourly data)
- Graph statistics (nodes and edges)
- Coordinate information for navigation

## License

[Add your license information here]

## Contributing

[Add contribution guidelines here]

## Acknowledgments

[Add acknowledgments here]
