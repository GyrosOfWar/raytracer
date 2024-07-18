use tracing::info;

use crate::Result;

pub fn measure<T>(name: &str, f: impl FnOnce() -> T) -> T {
    let start = std::time::Instant::now();
    let value = f();
    let elapsed = start.elapsed();
    info!("{} took {:?}", name, elapsed);
    value
}

pub fn try_measure<T>(name: &str, f: impl FnOnce() -> Result<T>) -> Result<T> {
    let start = std::time::Instant::now();
    let result = f()?;
    let elapsed = start.elapsed();
    info!("{} took {:?}", name, elapsed);
    Ok(result)
}
