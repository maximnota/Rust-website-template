// main.rs

use rwa::run_server; // Assuming your library is called `my_library`

#[tokio::main]
async fn main() {
    // Call the `run_server` function from the library to start the server
    run_server().await;
}

