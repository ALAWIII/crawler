use crawler::{get_args, run};

#[tokio::main]
async fn main() {
    match get_args() {
        Ok(c) => run(c)
            .await
            .inspect_err(|e| eprintln!("{}", e))
            .unwrap_or(()),
        Err(e) => eprintln!("{}", e),
    };
}
