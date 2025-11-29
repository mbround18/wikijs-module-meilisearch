use log::{Level, LevelFilter, Log, Metadata, Record};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["WIKI", "logger"])]
    fn info(s: &str);

    #[wasm_bindgen(js_namespace = ["WIKI", "logger"])]
    fn warn(s: &str);

    #[wasm_bindgen(js_namespace = ["WIKI", "logger"])]
    fn error(s: &str);
}

pub struct WasmLogger {
    namespace: &'static str,
}

impl Log for WasmLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        matches!(metadata.level(), Level::Info | Level::Warn | Level::Error)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let message = format!("{}: {}", self.namespace, record.args());

        match record.level() {
            Level::Info => {
                info(&message);
            }
            Level::Warn => {
                warn(&message);
            }
            Level::Error => {
                error(&message);
            }
            _ => {} // Extend to handle other levels if needed
        }
    }

    fn flush(&self) {}
}

impl WasmLogger {
    pub fn init(namespace: &'static str) {
        let logger = WasmLogger { namespace };
        log::set_boxed_logger(Box::new(logger)).unwrap();
        log::set_max_level(LevelFilter::Info);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_filters_above_warn() {
        // Create a logger instance and directly call enabled() to avoid calling externs
        let lg = WasmLogger { namespace: "test" };
        let meta_info = Metadata::builder().level(Level::Info).target("t").build();
        let meta_warn = Metadata::builder().level(Level::Warn).target("t").build();
        let meta_error = Metadata::builder().level(Level::Error).target("t").build();
        let meta_debug = Metadata::builder().level(Level::Debug).target("t").build();

        assert!(lg.enabled(&meta_info));
        assert!(lg.enabled(&meta_warn));
        assert!(lg.enabled(&meta_error));
        assert!(!lg.enabled(&meta_debug));
    }
}
