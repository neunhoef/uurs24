use crate::data::RegattaData;
use crate::optimize::{estimate_leg_performance, explore_paths, explore_target_paths};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tera::{Context, Tera};
use warp::Filter;
use warp::reply::html;

pub async fn start_server(data: RegattaData, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Tera templates
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => Arc::new(t),
        Err(e) => {
            eprintln!("Failed to initialize Tera templates: {e}");
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

    // Estimate leg form page route
    let estimate_leg_form_route = warp::path("estimate-leg")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_tera(tera.clone()))
        .and(with_data(data.clone()))
        .and_then(handle_estimate_leg_form);

    // Find paths form page route
    let find_paths_form_route = warp::path("find-paths")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_tera(tera.clone()))
        .and(with_data(data.clone()))
        .and_then(handle_find_paths_form);

    // Find target form page route
    let find_target_form_route = warp::path("find-target")
        .and(warp::path::end())
        .and(warp::get())
        .and(with_tera(tera.clone()))
        .and(with_data(data.clone()))
        .and_then(handle_find_target_form);

    // Version endpoint
    let version_route = warp::path("version").and(warp::get()).map(|| {
        let response = json!({
            "version": env!("CARGO_PKG_VERSION")
        });
        warp::reply::json(&response)
    });

    // Health check endpoint
    let health_route = warp::path("health").and(warp::get()).map(|| {
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

    // Estimate leg performance API endpoint
    let estimate_leg_api_route = warp::path("api")
        .and(warp::path("estimateleg"))
        .and(warp::get())
        .and(warp::query::<EstimateLegQuery>())
        .and(with_data(data.clone()))
        .and_then(handle_estimate_leg);

    // Find paths API endpoint
    let find_paths_api_route = warp::path("api")
        .and(warp::path("find-paths"))
        .and(warp::get())
        .and(warp::query::<FindPathsQuery>())
        .and(with_data(data.clone()))
        .and_then(handle_find_paths);

    // Find target API endpoint
    let find_target_api_route = warp::path("api")
        .and(warp::path("find-targets"))
        .and(warp::get())
        .and(warp::query::<FindTargetQuery>())
        .and(with_data(data.clone()))
        .and_then(handle_find_target);

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
        .or(estimate_leg_form_route)
        .or(find_paths_form_route)
        .or(find_target_form_route)
        .or(version_route)
        .or(health_route)
        .or(estimate_api_route)
        .or(estimate_leg_api_route)
        .or(find_paths_api_route)
        .or(find_target_api_route)
        .or(pdf_route)
        .or(svg_route)
        .with(warp::cors().allow_any_origin());

    println!(
        "Starting HTTP server on http://0.0.0.0:{port} (all interfaces)"
    );
    println!("Available endpoints:");
    println!("  GET /              - Main menu");
    println!("  GET /estimate      - Estimate form");
    println!("  GET /estimate-leg  - Estimate leg form");
    println!("  GET /find-paths    - Find paths form");
    println!("  GET /find-target   - Find target paths form");
    println!("  GET /regatta-graph.pdf - Show regatta graph as PDF");
    println!("  GET /regatta-course.svg - Show regatta map as SVG");
    println!("  GET /version       - Get program version");
    println!("  GET /health        - Health check");
    println!("  GET /api/estimate?from=X&to=Y&time=Z - Estimate leg performance");
    println!("  GET /api/estimateleg?from=X&to=Y&reverse=Z&time=W - Estimate leg performance");
    println!("  GET /api/find-paths?start=X&time=Y&steps=Z&max_paths=N - Find paths from starting point");
    println!("  GET /api/find-targets?start=X&target=Y&time=Z&steps=W&max_paths=N - Find paths to specific target");

    // Start the server
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;

    Ok(())
}

// Query parameters for the estimate endpoint
#[derive(Debug, Deserialize)]
struct EstimateQuery {
    from: String,
    to: String,
    time: f64,
}

// Query parameters for the estimate leg endpoint
#[derive(Debug, Deserialize)]
struct EstimateLegQuery {
    from: String,
    to: String,
    reverse: Option<bool>,
    time: f64,
}

// Query parameters for the find paths endpoint
#[derive(Debug, Deserialize)]
struct FindPathsQuery {
    start: String,
    time: f64,
    steps: usize,
    max_paths: Option<usize>,
}

// Query parameters for the find target endpoint
#[derive(Debug, Deserialize)]
struct FindTargetQuery {
    start: String,
    target: String,
    time: f64,
    steps: usize,
    max_paths: Option<usize>,
}

// Helper function to inject Tera into route handlers
fn with_tera(
    tera: Arc<Tera>,
) -> impl Filter<Extract = (Arc<Tera>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tera.clone())
}

// Helper function to inject data into route handlers
fn with_data(
    data: RegattaData,
) -> impl Filter<Extract = (RegattaData,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || data.clone())
}

// Handler for the main index page
async fn handle_index(
    tera: Arc<Tera>,
    _data: RegattaData,
) -> Result<impl warp::Reply, warp::Rejection> {
    let context = Context::new();
    let rendered_html = tera.render("index.html", &context).map_err(|e| {
        eprintln!("Template rendering error: {e}");
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
    let boeien: Vec<String> = data.boeien.iter().map(|boei| boei.name.clone()).collect();

    context.insert("boeien", &boeien);

    let rendered_html = tera.render("estimate.html", &context).map_err(|e| {
        eprintln!("Template rendering error: {e}");
        warp::reject::custom(TemplateError)
    })?;

    Ok(html(rendered_html))
}

// Handler for the estimate leg form page
async fn handle_estimate_leg_form(
    tera: Arc<Tera>,
    data: RegattaData,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut context = Context::new();

    // Get legs sorted alphabetically by from, then to
    let mut legs = data.rakken.clone();
    legs.sort_by(|a, b| a.from.cmp(&b.from).then_with(|| a.to.cmp(&b.to)));

    // Convert Rak structs to a format that Tera can understand
    let legs_for_template: Vec<serde_json::Value> = legs
        .iter()
        .map(|rak| {
            json!({
                "from": rak.from,
                "to": rak.to,
                "distance": rak.distance
            })
        })
        .collect();

    context.insert("legs", &legs_for_template);

    let rendered_html = tera.render("estimate-leg.html", &context).map_err(|e| {
        eprintln!("Template rendering error: {e:?}");
        warp::reject::custom(TemplateError)
    })?;

    Ok(html(rendered_html))
}

// Handler for the find paths form page
async fn handle_find_paths_form(
    tera: Arc<Tera>,
    data: RegattaData,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut context = Context::new();

    // Get boeien names for the dropdown
    let boeien: Vec<String> = data.boeien.iter().map(|boei| boei.name.clone()).collect();

    context.insert("boeien", &boeien);

    let rendered_html = tera.render("find-paths.html", &context).map_err(|e| {
        eprintln!("Template rendering error: {e}");
        warp::reject::custom(TemplateError)
    })?;

    Ok(html(rendered_html))
}

// Handler for the find target form page
async fn handle_find_target_form(
    tera: Arc<Tera>,
    data: RegattaData,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut context = Context::new();

    // Get boeien names for the dropdown
    let boeien: Vec<String> = data.boeien.iter().map(|boei| boei.name.clone()).collect();

    context.insert("boeien", &boeien);

    let rendered_html = tera.render("find-target.html", &context).map_err(|e| {
        eprintln!("Template rendering error: {e}");
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

// Handler for the estimate leg endpoint
async fn handle_estimate_leg(
    query: EstimateLegQuery,
    data: RegattaData,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Handle reverse direction by swapping from and to
    let (from_name, to_name) = if query.reverse.unwrap_or(false) {
        (query.to.clone(), query.from.clone())
    } else {
        (query.from.clone(), query.to.clone())
    };

    // Get boei indices by name
    let from_idx = match data.get_boei_index(&from_name) {
        Some(idx) => idx,
        None => {
            let error_response = json!({
                "error": "Boei not found",
                "message": format!("Boei '{}' not found", from_name)
            });
            return Ok(warp::reply::json(&error_response));
        }
    };

    let to_idx = match data.get_boei_index(&to_name) {
        Some(idx) => idx,
        None => {
            let error_response = json!({
                "error": "Boei not found",
                "message": format!("Boei '{}' not found", to_name)
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
        "from": from_name,
        "to": to_name,
        "time": query.time,
        "estimated_speed": performance.estimated_speed,
        "course_bearing": performance.course_bearing,
        "wind_direction": performance.wind_direction,
        "relative_bearing": performance.relative_bearing,
        "wind_speed": performance.wind_speed
    });

    Ok(warp::reply::json(&response))
}

// Handler for the find paths endpoint
async fn handle_find_paths(
    query: FindPathsQuery,
    data: RegattaData,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Get starting buoy index by name
    let start_idx = match data.get_boei_index(&query.start) {
        Some(idx) => idx,
        None => {
            let error_response = json!({
                "error": "Buoy not found",
                "message": format!("Starting buoy '{}' not found", query.start)
            });
            return Ok(warp::reply::json(&error_response));
        }
    };

    // Validate time parameter
    if query.time < 0.0 || query.time > 24.0 {
        let error_response = json!({
            "error": "Invalid time",
            "message": "Time must be between 0 and 24 hours"
        });
        return Ok(warp::reply::json(&error_response));
    }

    // Validate steps parameter
    if query.steps == 0 || query.steps > 10 {
        let error_response = json!({
            "error": "Invalid steps",
            "message": "Number of steps must be between 1 and 10"
        });
        return Ok(warp::reply::json(&error_response));
    }

    // Validate max_paths parameter
    let max_paths = query.max_paths;
    if let Some(max_paths_val) = max_paths {
        if max_paths_val == 0 || max_paths_val > 100000 {
            let error_response = json!({
                "error": "Invalid max_paths",
                "message": "Maximum number of paths must be between 1 and 100000"
            });
            return Ok(warp::reply::json(&error_response));
        }
    }

    // Explore paths
    match explore_paths(&data, start_idx, query.time, query.steps, query.max_paths) {
        Ok(paths) => {
            // Convert paths to JSON-friendly format
            let paths_json: Vec<serde_json::Value> = paths
                .iter()
                .map(|path| {
                    let steps_json: Vec<serde_json::Value> = path
                        .steps
                        .iter()
                        .map(|step| {
                            json!({
                                "from": step.from,
                                "to": step.to,
                                "from_name": data.boeien[step.from].name,
                                "to_name": data.boeien[step.to].name,
                                "distance": step.distance,
                                "speed": step.speed,
                                "start_time": step.start_time,
                                "end_time": step.end_time
                            })
                        })
                        .collect();

                    json!({
                        "steps": steps_json,
                        "total_distance": path.total_distance,
                        "end_time": path.end_time
                    })
                })
                .collect();

            let response = json!({
                "start": query.start,
                "start_time": query.time,
                "steps": query.steps,
                "paths": paths_json
            });

            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            let error_response = json!({
                "error": "Path exploration failed",
                "message": format!("Error exploring paths: {}", e)
            });
            Ok(warp::reply::json(&error_response))
        }
    }
}

// Handler for the find target endpoint
async fn handle_find_target(
    query: FindTargetQuery,
    data: RegattaData,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Get starting buoy index by name
    let start_idx = match data.get_boei_index(&query.start) {
        Some(idx) => idx,
        None => {
            let error_response = json!({
                "error": "Buoy not found",
                "message": format!("Starting buoy '{}' not found", query.start)
            });
            return Ok(warp::reply::json(&error_response));
        }
    };

    // Get target buoy index by name
    let target_idx = match data.get_boei_index(&query.target) {
        Some(idx) => idx,
        None => {
            let error_response = json!({
                "error": "Buoy not found",
                "message": format!("Target buoy '{}' not found", query.target)
            });
            return Ok(warp::reply::json(&error_response));
        }
    };

    // Validate time parameter
    if query.time < 0.0 || query.time > 24.0 {
        let error_response = json!({
            "error": "Invalid time",
            "message": "Time must be between 0 and 24 hours"
        });
        return Ok(warp::reply::json(&error_response));
    }

    // Validate steps parameter
    if query.steps == 0 || query.steps > 10 {
        let error_response = json!({
            "error": "Invalid steps",
            "message": "Maximum number of steps must be between 1 and 10"
        });
        return Ok(warp::reply::json(&error_response));
    }

    // Validate max_paths parameter
    let max_paths = query.max_paths;
    if let Some(max_paths_val) = max_paths {
        if max_paths_val == 0 || max_paths_val > 100000 {
            let error_response = json!({
                "error": "Invalid max_paths",
                "message": "Maximum number of paths must be between 1 and 100000"
            });
            return Ok(warp::reply::json(&error_response));
        }
    }

    // Check if start and target are the same
    if start_idx == target_idx {
        let error_response = json!({
            "error": "Invalid request",
            "message": "Starting and target buoys must be different"
        });
        return Ok(warp::reply::json(&error_response));
    }

    // Explore paths to target
    match explore_target_paths(&data, start_idx, target_idx, query.time, query.steps, max_paths) {
        Ok(paths) => {
            // Convert paths to JSON-friendly format
            let paths_json: Vec<serde_json::Value> = paths
                .iter()
                .map(|path| {
                    let steps_json: Vec<serde_json::Value> = path
                        .steps
                        .iter()
                        .map(|step| {
                            json!({
                                "from": step.from,
                                "to": step.to,
                                "from_name": data.boeien[step.from].name,
                                "to_name": data.boeien[step.to].name,
                                "distance": step.distance,
                                "speed": step.speed,
                                "start_time": step.start_time,
                                "end_time": step.end_time
                            })
                        })
                        .collect();

                    json!({
                        "steps": steps_json,
                        "total_distance": path.total_distance,
                        "end_time": path.end_time
                    })
                })
                .collect();

            let response = json!({
                "start": query.start,
                "target": query.target,
                "start_time": query.time,
                "steps": query.steps,
                "paths": paths_json
            });

            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            let error_response = json!({
                "error": "Path exploration failed",
                "message": format!("Error exploring paths to target: {}", e)
            });
            Ok(warp::reply::json(&error_response))
        }
    }
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
                "application/pdf",
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
                "image/svg+xml",
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
