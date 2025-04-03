use s4_vaultify::backend::file_flow::file_flow::send;

#[tokio::main]
async fn main() {
    send("/Users/lothaire/RustroverProjects/S4-Vaultify/test-files/IMG_1204.HEIC")
        .await
        .expect("TODO: panic message");
}
