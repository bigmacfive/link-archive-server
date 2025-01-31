use std::sync::Once;
use chrono::Local;
use env_logger::{Builder, fmt::Color};
use log::{Level, LevelFilter, Record};
use std::io::Write;

static INIT: Once = Once::new();

pub struct Logger {
    pub level: LevelFilter,
    pub with_colors: bool,
    pub with_target: bool,
    pub with_line_numbers: bool,
    pub with_thread_ids: bool,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            level: LevelFilter::Info,
            with_colors: true,
            with_target: true,
            with_line_numbers: true,
            with_thread_ids: true,
        }
    }

    pub fn with_level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    pub fn with_colors(mut self, enabled: bool) -> Self {
        self.with_colors = enabled;
        self
    }

    pub fn with_target(mut self, enabled: bool) -> Self {
        self.with_target = enabled;
        self
    }

    pub fn with_line_numbers(mut self, enabled: bool) -> Self {
        self.with_line_numbers = enabled;
        self
    }

    pub fn with_thread_ids(mut self, enabled: bool) -> Self {
        self.with_thread_ids = enabled;
        self
    }

    pub fn init(self) {
        INIT.call_once(|| {
            let mut builder = Builder::new();
            builder.format(move |buf, record| {
                let mut style = buf.style();
                let level_color = match record.level() {
                    Level::Error => Color::Red,
                    Level::Warn => Color::Yellow,
                    Level::Info => Color::Green,
                    Level::Debug => Color::Blue,
                    Level::Trace => Color::Cyan,
                };

                let mut level_style = buf.style();
                if self.with_colors {
                    level_style.set_color(level_color).set_bold(true);
                }

                let thread_id = std::thread::current().id();
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");

                writeln!(
                    buf,
                    "[{}] {}{}{}{} {}",
                    timestamp,
                    level_style.value(record.level()),
                    if self.with_thread_ids {
                        format!(" [{:?}]", thread_id)
                    } else {
                        String::new()
                    },
                    if self.with_target {
                        format!(" [{}]", record.target())
                    } else {
                        String::new()
                    },
                    if self.with_line_numbers {
                        if let Some(line) = record.line() {
                            format!(":{}", line)
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    },
                    style.value(record.args())
                )
            });

            builder.filter_level(self.level);
            builder.init();
        });
    }
}

#[macro_export]
macro_rules! log_request {
    ($req:expr) => {
        log::info!(
            "Request: {} {} from {}",
            $req.method(),
            $req.uri(),
            $req.connection_info().realip_remote_addr().unwrap_or("unknown")
        );
    };
}

#[macro_export]
macro_rules! log_response {
    ($res:expr) => {
        log::info!(
            "Response: Status {}",
            $res.status()
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($err:expr) => {
        log::error!("Error: {}", $err);
    };
    ($err:expr, $($arg:tt)*) => {
        log::error!("Error: {}, {}", $err, format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_db_query {
    ($query:expr) => {
        log::debug!("Executing query: {}", $query);
    };
    ($query:expr, $params:expr) => {
        log::debug!("Executing query: {} with params: {:?}", $query, $params);
    };
}