use tracing::info;

use crate::Result;

pub fn measure(name: &str, f: impl FnOnce()) {
    let start = std::time::Instant::now();
    f();
    let elapsed = start.elapsed();
    info!("{} took {:?}", name, elapsed);
}

pub fn try_measure(name: &str, f: impl FnOnce() -> Result<()>) -> Result<()> {
    let start = std::time::Instant::now();
    f()?;
    let elapsed = start.elapsed();
    info!("{} took {:?}", name, elapsed);
    Ok(())
}
