use s4_vaultify::backend::file_flow::receive_from::receive;

#[tokio::main]
async fn main() {
    receive().await.unwrap();
}