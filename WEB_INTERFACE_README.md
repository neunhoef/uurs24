# 24 Uurs Zeilrace Web Interface

This document describes the web interface for the 24 Uurs Zeilrace application.

## Overview

The web interface provides a user-friendly way to interact with the regatta data and estimate sailing performance between different buoys (boeien).

## Features

- **Main Menu**: Clean, maritime-styled interface with navigation options
- **Speed Estimation**: Form to calculate estimated boat performance between two buoys
- **Responsive Design**: Works on both desktop and mobile devices
- **Real-time API Integration**: JavaScript-based form submission with live results

## Getting Started

### Starting the Server

1. Build the project:
   ```bash
   cargo build
   ```

2. Start the HTTP server:
   ```bash
   cargo run -- serve --port 8080
   ```

3. Open your web browser and navigate to:
   ```
   http://127.0.0.1:8080/
   ```

### Available Endpoints

- `GET /` - Main menu page
- `GET /estimate` - Speed estimation form
- `GET /api/estimate?from=X&to=Y&time=Z` - API endpoint for performance estimation
- `GET /version` - Get program version
- `GET /health` - Health check

## Usage

### Main Menu

The main page displays a clean menu with the option to "Estimate Speed". Click this to proceed to the estimation form.

### Speed Estimation

1. **Select Starting Buoy**: Choose the departure buoy from the dropdown menu
2. **Select Destination Buoy**: Choose the arrival buoy from the dropdown menu
3. **Enter Time**: Specify the time in hours after race start
4. **Submit**: Click the submit button to calculate performance

### Results Display

The form will display:
- **From/To**: Selected buoys
- **Time**: Input time
- **Estimated Speed**: Calculated boat speed in knots
- **Course Bearing**: Direction to sail in degrees
- **Wind Direction**: Wind direction in degrees
- **Relative Bearing**: Angle between course and wind
- **Wind Speed**: Wind speed in knots

## Technical Details

### Frontend

- **HTML5**: Semantic markup with responsive design
- **CSS3**: Modern styling with maritime theme, gradients, and animations
- **JavaScript**: ES6+ async/await for API communication
- **Tera Templates**: Server-side templating for dynamic content

### Backend

- **Rust**: High-performance server implementation
- **Warp**: Lightweight HTTP framework
- **Tera**: Template engine for HTML generation
- **JSON API**: RESTful endpoint for performance calculations

### Styling

The interface features a maritime theme with:
- Ocean blue gradient backgrounds
- Nautical decorations (anchor emoji)
- Smooth hover animations
- Responsive grid layouts
- Professional typography

## Browser Compatibility

- Chrome/Chromium (recommended)
- Firefox
- Safari
- Edge

## Troubleshooting

### Common Issues

1. **Server won't start**: Check if port 8080 is already in use
2. **API errors**: Verify buoy names exist in the data
3. **Page not loading**: Ensure the server is running and accessible

### Error Messages

- **"Boei not found"**: The specified buoy name doesn't exist
- **"Invalid time"**: Time must be a non-negative number
- **"Network error"**: Server connection issues

## Development

### Adding New Features

1. Create new Tera templates in the `templates/` directory
2. Add new routes in `src/server.rs`
3. Update the main menu to include new options

### Modifying Styles

Edit the CSS in `templates/base.html` to change the appearance of all pages.

### API Extensions

Add new endpoints in `src/server.rs` following the existing pattern for the estimate API.

## License

This project is part of the 24 Uurs Zeilrace application.
