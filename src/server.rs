use warp::Filter;
use serde_json::json;
use crate::data::RegattaData;

pub async fn start_server(_data: RegattaData, port: u16) -> Result<(), Box<dyn std::error::Error>> {
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

    // Combine all routes
    let routes = version_route
        .or(health_route)
        .with(warp::cors().allow_any_origin());

    println!("Starting HTTP server on http://127.0.0.1:{}", port);
    println!("Available endpoints:");
    println!("  GET /version  - Get program version");
    println!("  GET /health   - Health check");

    // Start the server
    warp::serve(routes)
        .run(([127, 0, 0, 1], port))
        .await;

    Ok(())
}
