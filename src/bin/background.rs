#[tokio::main]
async fn main() {
    oxide_sei_market::background().await;
}
