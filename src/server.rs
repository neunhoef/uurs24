use warp::Filter;
use serde_json::json;
use serde::Deserialize;
use crate::data::RegattaData;
use crate::optimize::estimate_leg_performance;
use tera::{Tera, Context};
use std::sync::Arc;
use warp::reply::html;

pub async fn start_server(data: RegattaData, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Tera templates
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => Arc::new(t),
        Err(e) => {
            eprintln!("Failed to initialize Tera templates: {}", e);
            return Err("Template initialization failed".into());
        }
    };

    // Main page route
    let index_route = warp::path::end()
        .and(with_tera(tera.clone()))
        .and(with_data(data.clone()))
        .and_then(handle_index);

    // Estimate form page route
    let estimate_form_route = warp::path("estimate")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_tera(tera.clone()))
        .and(with_data(data.clone()))
        .and_then(handle_estimate_form);

    // Version endpoint
    let version_route = warp::path("version")
        .and(warp::get())
        .map(|| {
            let response = json!({
                "version": env!("CARGO_PKG_VERSION")
            });
            warp::reply::json(&response)
        });

    // Health check endpoint
    let health_route = warp::path("health")
        .and(warp::get())
        .map(|| {
            let response = json!({
                "status": "ok",
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            warp::reply::json(&response)
        });

    // Estimate leg performance API endpoint
    let estimate_api_route = warp::path("api")
        .and(warp::path("estimate"))
        .and(warp::get())
        .and(warp::query::<EstimateQuery>())
        .and(with_data(data.clone()))
        .and_then(handle_estimate);

    // PDF file serving route
    let pdf_route = warp::path("regatta-graph.pdf")
        .and(warp::path::end())
        .and(warp::get())
        .and_then(handle_pdf);

    // SVG file serving route
    let svg_route = warp::path("regatta-course.svg")
        .and(warp::path::end())
        .and(warp::get())
        .and_then(handle_svg);

    // Combine all routes - API routes must come before page routes to avoid conflicts
    let routes = index_route
        .or(estimate_form_route)
        .or(version_route)
        .or(health_route)
        .or(estimate_api_route)
        .or(pdf_route)
        .or(svg_route)
        .with(warp::cors().allow_any_origin());

    println!("Starting HTTP server on http://0.0.0.0:{} (all interfaces)", port);
    println!("Available endpoints:");
    println!("  GET /              - Main menu");
    println!("  GET /estimate      - Estimate form");
    println!("  GET /regatta-graph.pdf - Show regatta graph as PDF");
    println!("  GET /regatta-course.svg - Show regatta map as SVG");
    println!("  GET /version       - Get program version");
    println!("  GET /health        - Health check");
    println!("  GET /api/estimate?from=X&to=Y&time=Z - Estimate leg performance");

    // Start the server
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;

    Ok(())
}

// Query parameters for the estimate endpoint
#[derive(Debug, Deserialize)]
struct EstimateQuery {
    from: String,
    to: String,
    time: f64,
}

// Helper function to inject Tera into route handlers
fn with_tera(tera: Arc<Tera>) -> impl Filter<Extract = (Arc<Tera>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tera.clone())
}

// Helper function to inject data into route handlers
fn with_data(data: RegattaData) -> impl Filter<Extract = (RegattaData,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || data.clone())
}

// Handler for the main index page
async fn handle_index(
    tera: Arc<Tera>,
    _data: RegattaData,
) -> Result<impl warp::Reply, warp::Rejection> {
    let context = Context::new();
    let rendered_html = tera.render("index.html", &context)
        .map_err(|e| {
            eprintln!("Template rendering error: {}", e);
            warp::reject::custom(TemplateError)
        })?;
    
    Ok(html(rendered_html))
}

// Handler for the estimate form page
async fn handle_estimate_form(
    tera: Arc<Tera>,
    data: RegattaData,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut context = Context::new();
    
    // Get boeien names for the dropdown
    let boeien: Vec<String> = data.boeien.iter()
        .map(|boei| boei.name.clone())
        .collect();
    
    context.insert("boeien", &boeien);
    
    let rendered_html = tera.render("estimate.html", &context)
        .map_err(|e| {
            eprintln!("Template rendering error: {}", e);
            warp::reject::custom(TemplateError)
        })?;
    
    Ok(html(rendered_html))
}

// Handler for the estimate endpoint
async fn handle_estimate(
    query: EstimateQuery,
    data: RegattaData,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Get boei indices by name
    let from_idx = match data.get_boei_index(&query.from) {
        Some(idx) => idx,
        None => {
            let error_response = json!({
                "error": "Boei not found",
                "message": format!("Boei '{}' not found", query.from)
            });
            return Ok(warp::reply::json(&error_response));
        }
    };

    let to_idx = match data.get_boei_index(&query.to) {
        Some(idx) => idx,
        None => {
            let error_response = json!({
                "error": "Boei not found",
                "message": format!("Boei '{}' not found", query.to)
            });
            return Ok(warp::reply::json(&error_response));
        }
    };

    // Validate time parameter
    if query.time < 0.0 {
        let error_response = json!({
            "error": "Invalid time",
            "message": "Time must be non-negative"
        });
        return Ok(warp::reply::json(&error_response));
    }

    // Estimate leg performance
    let performance = estimate_leg_performance(&data, from_idx, to_idx, query.time);

    // Return the result as JSON
    let response = json!({
        "from": query.from,
        "to": query.to,
        "time": query.time,
        "estimated_speed": performance.estimated_speed,
        "course_bearing": performance.course_bearing,
        "wind_direction": performance.wind_direction,
        "relative_bearing": performance.relative_bearing,
        "wind_speed": performance.wind_speed
    });

    Ok(warp::reply::json(&response))
}

// Handler for serving the PDF file
async fn handle_pdf() -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    // Check if the PDF file exists
    if !std::path::Path::new("regatta_graph.pdf").exists() {
        // Return an error response
        let error_response = json!({
            "error": "PDF file not found",
            "message": "The regatta graph PDF file does not exist. Please generate it first using the 'graph' subcommand."
        });
        return Ok(Box::new(warp::reply::json(&error_response)));
    }

    // Read the PDF file
    match std::fs::read("regatta_graph.pdf") {
        Ok(pdf_content) => {
            // Return the PDF file with proper headers
            Ok(Box::new(warp::reply::with_header(
                pdf_content,
                "Content-Type",
                "application/pdf"
            )))
        }
        Err(_) => {
            // Return an error response if we can't read the file
            let error_response = json!({
                "error": "File read error",
                "message": "Could not read the PDF file"
            });
            Ok(Box::new(warp::reply::json(&error_response)))
        }
    }
}

// Handler for serving the SVG file
async fn handle_svg() -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    // Check if the SVG file exists
    if !std::path::Path::new("regatta_course.svg").exists() {
        // Return an error response
        let error_response = json!({
            "error": "SVG file not found",
            "message": "The regatta course SVG file does not exist. Please generate it first using the 'plot' subcommand."
        });
        return Ok(Box::new(warp::reply::json(&error_response)));
    }

    // Read the SVG file
    match std::fs::read("regatta_course.svg") {
        Ok(svg_content) => {
            // Return the SVG file with proper headers
            Ok(Box::new(warp::reply::with_header(
                svg_content,
                "Content-Type",
                "image/svg+xml"
            )))
        }
        Err(_) => {
            // Return an error response if we can't read the file
            let error_response = json!({
                "error": "File read error",
                "message": "Could not read the SVG file"
            });
            Ok(Box::new(warp::reply::json(&error_response)))
        }
    }
}

// Custom error type for template rendering
#[derive(Debug)]
struct TemplateError;

impl warp::reject::Reject for TemplateError {}
