use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let routes = warp::post()
        .and(warp::body::json())
        .map(|json: serde_json::Value| {
            println!("Received JSON: {}", json);
            warp::reply::json(&json)
        });

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}
