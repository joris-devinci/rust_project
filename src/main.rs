mod models;
mod threads;

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    runtime.block_on(async {
        let mut handles = Vec::new();

        for i in 0..3 {
            handles.push(tokio::spawn(threads::producer::producer(i)));
        }

        handles.push(tokio::spawn(threads::consumer::consumer()));

        for handle in handles {
            if let Err(error) = handle.await.unwrap() {
                eprintln!("Request failed: {error}");
            }
        }
    });
}