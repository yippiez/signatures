//! Demonstrates deep nesting: modules inside modules, impls inside mods,
//! functions inside impls, and constants at every level.

pub const ROOT_VERSION: &str = "2.0";

pub mod config {
    /// Maximum number of workers across all pools.
    pub const MAX_WORKERS: usize = 256;

    pub struct Settings {
        pub workers: usize,
        pub debug: bool,
    }

    impl Settings {
        pub fn default() -> Self {
            Settings { workers: 4, debug: false }
        }

        pub fn with_workers(mut self, n: usize) -> Self {
            self.workers = n;
            self
        }
    }

    pub mod logging {
        pub const DEFAULT_LEVEL: &str = "info";

        pub enum Level {
            Trace,
            Debug,
            Info,
            Warn,
            Error,
        }

        impl Level {
            pub fn from_str(s: &str) -> Option<Level> {
                match s {
                    "trace" => Some(Level::Trace),
                    "debug" => Some(Level::Debug),
                    "info" => Some(Level::Info),
                    "warn" => Some(Level::Warn),
                    "error" => Some(Level::Error),
                    _ => None,
                }
            }

            pub fn as_str(&self) -> &'static str {
                match self {
                    Level::Trace => "trace",
                    Level::Debug => "debug",
                    Level::Info => "info",
                    Level::Warn => "warn",
                    Level::Error => "error",
                }
            }
        }

        pub trait Sink {
            fn write_record(&mut self, level: &Level, message: &str);
            fn flush(&mut self);
        }

        pub struct ConsoleSink {
            prefix: String,
        }

        impl ConsoleSink {
            pub fn new(prefix: impl Into<String>) -> Self {
                ConsoleSink { prefix: prefix.into() }
            }
        }

        impl Sink for ConsoleSink {
            fn write_record(&mut self, level: &Level, message: &str) {
                println!("[{}] {}: {}", self.prefix, level.as_str(), message);
            }

            fn flush(&mut self) {
                // nothing to flush for stdout
            }
        }
    }

    pub mod storage {
        pub const DEFAULT_PATH: &str = "/var/lib/app";
        pub const BUFFER_SIZE: usize = 8192;

        pub struct DiskStore {
            path: std::path::PathBuf,
            buf_size: usize,
        }

        impl DiskStore {
            pub fn open(path: impl Into<std::path::PathBuf>) -> std::io::Result<Self> {
                Ok(DiskStore { path: path.into(), buf_size: BUFFER_SIZE })
            }

            pub fn buf_size(&self) -> usize {
                self.buf_size
            }

            fn flush_buffers(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
    }
}

pub mod runtime {
    pub struct Handle {
        id: u64,
    }

    impl Handle {
        pub fn spawn<F>(&self, future: F)
        where
            F: std::future::Future<Output = ()> + Send + 'static,
        {
            let _ = (self.id, future);
        }

        pub fn block_on<F, T>(&self, future: F) -> T
        where
            F: std::future::Future<Output = T>,
        {
            let _ = self.id;
            let _ = future;
            unimplemented!()
        }
    }

    pub fn current() -> Handle {
        Handle { id: 0 }
    }
}
