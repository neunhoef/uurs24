use warp::Filter;
use serde_json::json;
use serde::Deserialize;
use crate::data::RegattaData;
use crate::optimize::estimate_leg_performance;

pub async fn start_server(data: RegattaData, port: u16) -> Result<(), Box<dyn std::error::Error>> {
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

    // Estimate leg performance endpoint
    let estimate_route = warp::path("estimate")
        .and(warp::get())
        .and(warp::query::<EstimateQuery>())
        .and(with_data(data.clone()))
        .and_then(handle_estimate);

    // Combine all routes
    let routes = version_route
        .or(health_route)
        .or(estimate_route)
        .with(warp::cors().allow_any_origin());

    println!("Starting HTTP server on http://127.0.0.1:{}", port);
    println!("Available endpoints:");
    println!("  GET /version  - Get program version");
    println!("  GET /health   - Health check");
    println!("  GET /estimate - Estimate leg performance (query params: from, to, time)");

    // Start the server
    warp::serve(routes)
        .run(([127, 0, 0, 1], port))
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

// Helper function to inject data into route handlers
fn with_data(data: RegattaData) -> impl Filter<Extract = (RegattaData,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || data.clone())
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
