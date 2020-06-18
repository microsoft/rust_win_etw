use log::{debug, error, info, log_enabled, trace, warn, Level};
use std::time::Duration;
use win_etw_logger::TraceLogger;

fn main() {
    let logger = TraceLogger::new().unwrap();

    log::set_boxed_logger(Box::new(logger)).unwrap();
    log::set_max_level(log::LevelFilter::Debug);

    info!("Rust logging through ETW!  n = {}", 42);
    warn!("This is too much fun");
    debug!("maybe we can make this code work");

    let args = std::env::args().collect::<Vec<String>>();
    if args.len() >= 2 && args[1] == "loop" {
        eprintln!("looping");
        loop {
            std::thread::sleep(Duration::from_millis(3000));
            error!("error: something pretty bad happened!");
            warn!("warn: something warning-worthy happened");
            info!("info: something normal happened");
            debug!("debug: noisy debug noisy debug");
            trace!("trace: noisy tracing noisy tracing");

            let error_is_enabled = log_enabled!(Level::Error);
            let warn_is_enabled = log_enabled!(Level::Warn);
            let info_is_enabled = log_enabled!(Level::Info);
            let debug_is_enabled = log_enabled!(Level::Debug);
            let trace_is_enabled = log_enabled!(Level::Trace);
            eprintln!(
                "is_enabled?  error: {:5?}, warn: {:5?}, info: {:5?}, debug: {:5?}, trace: {:5?}",
                error_is_enabled,
                warn_is_enabled,
                info_is_enabled,
                debug_is_enabled,
                trace_is_enabled,
            );
        }
    }
}
