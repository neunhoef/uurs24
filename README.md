# Uurs24 - 24-Hour Regatta Data Management Tool

A comprehensive Rust-based tool for managing and visualizing 24-hour regatta data, including buoys, start lines, legs, and polar performance data. Features both command-line interface and web-based interface for interactive analysis.

## Features

- **Data Management**: Load and parse regatta data from CSV files
- **Course Visualization**: Generate SVG plots and PDF graphs of the regatta course
- **Performance Analysis**: Display polar performance data for different wind conditions
- **Performance Estimation**: Estimate boat performance between buoys based on wind conditions and polar data
- **Path Finding**: Explore all possible sailing paths from a starting point
- **Target Path Analysis**: Find optimal paths to specific target buoys
- **Graph Representation**: Build and analyze regatta course as a directed graph
- **Web Interface**: Interactive web-based interface for sailing performance analysis
- **REST API**: HTTP server providing programmatic access to all features
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
│   ├── wind.csv        # Wind conditions during the race
│   └── zeiten.csv      # Race timing data
├── templates/          # Web interface templates
│   ├── base.html       # Base template with styling
│   ├── index.html      # Main menu page
│   ├── estimate.html   # Speed estimation form
│   ├── estimate-leg.html # Leg estimation form
│   ├── find-paths.html # Path finding form
│   └── find-target.html # Target path form
├── regatta_course.svg  # Generated course visualization
├── regatta_graph.pdf   # Generated graph visualization
├── regatta-map.svg     # Regatta map visualization
└── src/
    ├── main.rs         # Main application logic and CLI
    ├── data.rs         # Data structures and parsing
    ├── optimize.rs     # Performance estimation and path finding algorithms
    ├── plot.rs         # SVG visualization generation
    └── server.rs       # HTTP server and web interface
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

# Export regatta graph to DOT file for graphviz
./target/release/uurs24 graph
./target/release/uurs24 graph -o my_graph.dot

# Estimate boat performance between two buoys
./target/release/uurs24 estimate OEVE WV12 2.0

# Explore all possible paths from a starting buoy
./target/release/uurs24 paths OEVE 0.0 3

# Find paths to a specific target buoy
./target/release/uurs24 target OEVE WV12 0.0 5

# Start HTTP server to serve regatta data and web interface
./target/release/uurs24 serve
./target/release/uurs24 serve --port 8080
```

### Command Line Options

- `show`: Display comprehensive regatta data including buoys, start lines, legs, and polar data
- `plot`: Generate SVG visualization with optional output file specification
- `graph`: Export the regatta graph to a DOT file for graphviz visualization
- `estimate`: Estimate boat performance between two buoys at a specific time
- `paths`: Explore all possible sailing paths from a starting buoy for a given number of steps
- `target`: Find optimal paths from a starting buoy to a specific target buoy
- `serve`: Start HTTP server to serve regatta data via REST API and web interface

## Web Interface

The `serve` subcommand starts an HTTP server that provides both a web interface and REST API endpoints for accessing regatta data.

### Starting the Server

```bash
# Start server on default port 3030
./target/release/uurs24 serve

# Start server on custom port
./target/release/uurs24 serve --port 8080
```

### Web Interface Features

The web interface provides an intuitive, maritime-themed interface with the following pages:

- **Main Menu** (`/`) - Central navigation hub with links to all features
- **Speed Estimation** (`/estimate`) - Form to estimate boat performance between two buoys
- **Leg Speed Estimation** (`/estimate-leg`) - Form to estimate performance for specific course legs
- **Path Finding** (`/find-paths`) - Explore all possible sailing paths from a starting point
- **Target Path Analysis** (`/find-target`) - Find optimal paths to specific target buoys
- **Course Visualization** (`/regatta-course.svg`) - Interactive SVG map of the regatta course
- **Graph Visualization** (`/regatta-graph.pdf`) - PDF visualization of the regatta graph

### REST API Endpoints

- `GET /version` - Get program version information
  - Response: `{"version": "1.0"}`
- `GET /health` - Health check endpoint
  - Response: `{"status": "ok", "timestamp": "2025-01-27T..."}`
- `GET /api/estimate?from=X&to=Y&time=Z` - Estimate boat performance between buoys
- `GET /api/estimateleg?start=X&end=Y&time=Z` - Estimate performance for specific legs
- `GET /api/find-paths?start=X&time=Y&steps=Z` - Find all possible paths
- `GET /api/find-target?start=X&target=Y&time=Z&steps=W` - Find paths to target

The server runs on `127.0.0.1` and supports CORS for cross-origin requests.

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
- Robust fallback handling for missing wind data hours

### Performance Estimation
- Estimates boat speed between any two buoys based on:
  - Course bearing calculation from coordinates
  - Wind conditions at specific race times
  - Polar performance data interpolation
  - Relative wind angle calculations
- Provides comprehensive output including:
  - Estimated boat speed in knots
  - Course bearing and wind direction
  - Relative bearing to wind
  - Wind speed and sailing interpretation
- Handles edge cases like beating (sailing into the wind) with appropriate speed reduction

### Path Finding and Route Optimization
- Explores all possible sailing paths from any starting buoy
- Analyzes multi-step routes with performance calculations for each leg
- Finds optimal paths to specific target buoys
- Takes into account wind conditions and boat performance for each route segment
- Provides comprehensive route analysis including total time and distance

### Advanced Visualization
- Generates high-quality SVG course maps with detailed buoy layouts
- Creates PDF graph visualizations using Graphviz integration
- Supports both static file generation and web-based viewing
- Interactive web interface for real-time visualization access

### Legacy Visualization (Single Purpose)
- SVG output with configurable dimensions
- Geographic coordinate mapping
- Buoy markers and course lines
- Customizable styling options

## Dependencies

- **clap**: Command-line argument parsing
- **chrono**: Date and time handling
- **csv**: CSV file reading and parsing
- **petgraph**: Graph data structures and algorithms
- **serde**: Serialization/deserialization
- **serde_json**: JSON serialization support
- **svg**: SVG generation and manipulation
- **tera**: Template engine for web interface
- **tokio**: Asynchronous runtime for HTTP server
- **warp**: Fast, lightweight HTTP framework
- **mime_guess**: MIME type detection for static files

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
- **`src/optimize.rs`**: Performance estimation algorithms, path finding, and optimization
- **`src/plot.rs`**: SVG visualization generation and coordinate mapping
- **`src/server.rs`**: HTTP server implementation and web interface handlers
- **`templates/`**: Tera templates for the web interface

## Example Output

The tool provides comprehensive output including:

- Buoy count and types
- Start line and leg information
- Polar performance data tables
- Wind conditions during the race (hourly data)
- Graph statistics (nodes and edges)
- Coordinate information for navigation

### Performance Estimation Example

```bash
$ ./target/release/uurs24 estimate OEVE WV12 2.0

Leg Performance Estimate:
  From: OEVE (Startboei)
  To:   WV12 (Markeerboei)
  Time: 2.0 hours after race start

Results:
  Estimated Speed: 3.76 knots
  Course Bearing:  130.9°
  Wind Direction:  180.0°
  Relative Bearing: 49.1°
  Wind Speed:      10.0 knots

Interpretation:
  Sailing on a close reach
```

This shows how the tool estimates boat performance between buoys OEVE and WV12 at 2 hours into the race, taking into account the course bearing, wind conditions, and polar performance data.

### Path Finding Examples

```bash
# Explore all possible 3-step paths from OEVE starting at race start
$ ./target/release/uurs24 paths OEVE 0.0 3

Path Exploration from OEVE (Startboei):
  Starting time: 0.0 hours
  Max steps: 3

Found paths:
  1. OEVE → WV01 → WV02 → WV03 (Total time: 2.45h, Distance: 8.2nm)
  2. OEVE → WV12 → WV13 → WV14 (Total time: 3.12h, Distance: 9.8nm)
  ...

# Find optimal paths from OEVE to WV12 with up to 5 steps
$ ./target/release/uurs24 target OEVE WV12 0.0 5

Target Path Analysis:
  From: OEVE (Startboei)
  To: WV12 (Markeerboei)  
  Starting time: 0.0 hours
  Max steps: 5

Optimal paths found:
  1. OEVE → WV12 (Direct, 1.8h, 6.2nm)
  2. OEVE → WV01 → WV12 (Via WV01, 2.1h, 7.1nm)
  ...
```

### Web Interface Example

Access the web interface by starting the server and navigating to `http://127.0.0.1:3030/`:

1. **Main Menu**: Choose from speed estimation, path finding, or visualization options
2. **Interactive Forms**: Select buoys from dropdown menus and enter timing parameters
3. **Real-time Results**: Get instant performance calculations and path analysis
4. **Visual Navigation**: View course maps and graph visualizations directly in the browser

## Browser Requirements

The web interface is compatible with modern browsers:
- Chrome/Chromium (recommended)
- Firefox
- Safari 
- Edge

JavaScript is required for interactive features and API communication.

## License

[Add your license information here]

## Contributing

[Add contribution guidelines here]

## Acknowledgments

[Add acknowledgments here]
