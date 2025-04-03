use s4_vaultify::backend::file_flow::file_flow::receive;

#[tokio::main]
async fn main() {
    receive().await.unwrap();
}
