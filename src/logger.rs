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
        metadata.level() <= Level::Warn
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
