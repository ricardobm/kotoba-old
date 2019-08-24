use std::{thread, time};

const INTERVAL: time::Duration = time::Duration::from_millis(2000);

fn main() {
    let start = time::Instant::now();
    loop {
        let duration = start.elapsed();
        println!("Hello from Hongo! (Δ = {:?})", duration);
        thread::sleep(INTERVAL);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_succeed() {
        assert_eq!(42, 42);
    }
}
