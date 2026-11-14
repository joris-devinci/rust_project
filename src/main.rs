mod models;
mod threads;

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    runtime.block_on(async {
        let mut handles = Vec::new();
        let producer_count = 3;

        for tid in 0..producer_count {
            handles.push(tokio::spawn(threads::producer::producer(
                tid,
                producer_count,
            )));
        }

        handles.push(tokio::spawn(threads::consumer::consumer()));

        for handle in handles {
            if let Err(error) = handle.await.unwrap() {
                eprintln!("Request failed: {error}");
            }
        }
    });
}