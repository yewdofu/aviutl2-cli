mod catalog;
mod catalog_schema;
mod cli;
mod commands;
mod config;
mod log_writer;
mod schema;
mod util;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();
    if cli.no_color {
        colored::control::set_override(false);
    }
    let level = tracing::Level::INFO;
    match cli.log_style {
        cli::LogStyle::Original => {
            tracing_subscriber::fmt()
                .event_format(crate::log_writer::LogFormatter)
                .with_max_level(level)
                .init();
        }
        cli::LogStyle::Default => {
            tracing_subscriber::fmt()
                .with_ansi(!cli.no_color)
                .with_max_level(level)
                .init();
        }
        cli::LogStyle::Compact => {
            tracing_subscriber::fmt()
                .with_ansi(!cli.no_color)
                .compact()
                .with_max_level(level)
                .init();
        }
        cli::LogStyle::Pretty => {
            tracing_subscriber::fmt()
                .with_ansi(!cli.no_color)
                .pretty()
                .with_max_level(level)
                .init();
        }
    }
    let config_opts = config::ConfigLoadOpts {
        patch: cli.config_patch,
        override_path: cli.config_override,
    };
    if let Err(e) = commands::run(cli.command, config_opts) {
        tracing::error!("{:?}", e);
        std::process::exit(1);
    }
}
