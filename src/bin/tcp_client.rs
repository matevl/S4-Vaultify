use s4_vaultify::backend::file_flow::file_flow::send;

#[tokio::main]
async fn main() {
    send("/Users/lothaire/Document/Photos windows.zip")
        .await
        .expect("TODO: panic message");
}
