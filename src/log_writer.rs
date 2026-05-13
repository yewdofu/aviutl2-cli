use colored::Colorize;
use tracing_log::NormalizeEvent;
use tracing_subscriber::fmt::FormatFields;

#[derive(Debug, Clone, Default)]
pub struct LogFormatter;

impl<C, N> tracing_subscriber::fmt::FormatEvent<C, N> for LogFormatter
where
    C: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> tracing_subscriber::fmt::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, C, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let meta = event.normalized_metadata();
        let meta = meta.as_ref().unwrap_or_else(|| event.metadata());
        let target = meta.target();
        let level_prefix = match *meta.level() {
            tracing::Level::TRACE => "=)".bright_black(),
            tracing::Level::DEBUG => "-)".bright_magenta(),
            tracing::Level::INFO => "i)".bright_blue(),
            tracing::Level::WARN => "!)".bright_yellow(),
            tracing::Level::ERROR => "x)".bright_red(),
        };
        write!(writer, "{} ", level_prefix)?;
        write!(
            writer,
            "{}{}{} ",
            "[".bright_black(),
            target.bright_black(),
            "]".bright_black()
        )?;
        if let Some(span) = ctx.lookup_current() {
            let mut spans = Vec::new();
            let mut current = Some(span);
            while let Some(span) = current {
                current = span.parent();
                spans.push(span);
            }
            spans.reverse();
            for span in spans {
                write!(
                    writer,
                    "{}{}",
                    "<".bright_black(),
                    span.name().bright_black()
                )?;
                let exts = span.extensions();
                let fields = exts
                    .get::<tracing_subscriber::fmt::FormattedFields<N>>()
                    .expect("formatted fields should be present");
                if !fields.is_empty() {
                    write!(
                        writer,
                        "{}{}{}",
                        "{".bright_black(),
                        fields,
                        "}".bright_black()
                    )?;
                }
                write!(writer, "{} ", ">".bright_black())?;
            }
        }
        ctx.format_fields(writer.by_ref(), event)?;
        writer.write_str("\n")?;
        Ok(())
    }
}
