use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct Output {
    stdout: StandardStream,
    stderr: StandardStream,
    no_color: bool,
}

impl Output {
    pub fn new() -> Self {
        let no_color =
            std::env::var("NO_COLOR").is_ok() || std::env::var("TERM").as_deref() == Ok("dumb");
        let choice = if no_color {
            ColorChoice::Never
        } else {
            ColorChoice::Auto
        };
        Self {
            stdout: StandardStream::stdout(choice),
            stderr: StandardStream::stderr(choice),
            no_color,
        }
    }

    pub fn with_color(no_color: bool) -> Self {
        let choice = if no_color {
            ColorChoice::Never
        } else {
            ColorChoice::Auto
        };
        Self {
            stdout: StandardStream::stdout(choice),
            stderr: StandardStream::stderr(choice),
            no_color,
        }
    }

    pub fn success(&self, msg: &str) -> io::Result<()> {
        self.print_colored(
            &self.stdout,
            &format!("{} {}", green("✓"), msg),
            Some(Color::Green),
        )
    }

    pub fn error(&self, msg: &str) -> io::Result<()> {
        self.print_colored(
            &self.stderr,
            &format!("{} {}", red("✗"), msg),
            Some(Color::Red),
        )
    }

    pub fn warning(&self, msg: &str) -> io::Result<()> {
        self.print_colored(
            &self.stdout,
            &format!("{} {}", yellow("⚠"), msg),
            Some(Color::Yellow),
        )
    }

    pub fn info(&self, msg: &str) -> io::Result<()> {
        self.print_colored(&self.stdout, msg, Some(Color::Cyan))
    }

    pub fn println(&self, msg: &str) -> io::Result<()> {
        self.print_colored(&self.stdout, msg, None)
    }

    pub fn eprintln(&self, msg: &str) -> io::Result<()> {
        self.print_colored(&self.stderr, msg, None)
    }

    pub fn bold(&self, msg: &str) -> io::Result<()> {
        let mut spec = ColorSpec::new();
        spec.set_bold(true);
        self.print_with_spec(&self.stdout, msg, &spec)
    }

    pub fn dim(&self, msg: &str) -> io::Result<()> {
        let mut spec = ColorSpec::new();
        spec.set_dimmed(true);
        self.print_with_spec(&self.stdout, msg, &spec)
    }

    pub fn header(&self, msg: &str) -> io::Result<()> {
        self.println("")?;
        let mut spec = ColorSpec::new();
        spec.set_bold(true);
        self.print_with_spec(&self.stdout, msg, &spec)?;
        self.println("")
    }

    pub fn divider(&self, ch: char, width: usize) -> io::Result<()> {
        let line: String = std::iter::repeat(ch).take(width).collect();
        self.print_colored(&self.stdout, &line, None)
    }

    pub fn status_icon(&self, status: Status, msg: &str) -> io::Result<()> {
        let (icon, color) = match status {
            Status::Done => ("✓", Color::Green),
            Status::Error => ("✗", Color::Red),
            Status::Warning => ("⚠", Color::Yellow),
            Status::Active => ("●", Color::Yellow),
            Status::Pending => ("○", Color::White),
            Status::Info => ("●", Color::Cyan),
        };
        self.print_colored(
            &self.stdout,
            &format!("{} {}", color_str(icon, color), msg),
            Some(color),
        )
    }

    fn print_colored(
        &self,
        stream: &StandardStream,
        msg: &str,
        color: Option<Color>,
    ) -> io::Result<()> {
        let mut spec = ColorSpec::new();
        if let Some(c) = color {
            spec.set_fg(Some(c));
        }
        self.print_with_spec(stream, msg, &spec)
    }

    fn print_with_spec(
        &self,
        stream: &StandardStream,
        msg: &str,
        spec: &ColorSpec,
    ) -> io::Result<()> {
        let mut handle = stream.lock();
        handle.set_color(spec)?;
        write!(handle, "{}", msg)?;
        handle.reset()?;
        writeln!(handle)?;
        Ok(())
    }

    pub fn is_no_color(&self) -> bool {
        self.no_color
    }
}

impl Default for Output {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
pub enum Status {
    Done,
    Error,
    Warning,
    Active,
    Pending,
    Info,
}

pub fn green(s: &str) -> String {
    color_str(s, Color::Green)
}

pub fn red(s: &str) -> String {
    color_str(s, Color::Red)
}

pub fn yellow(s: &str) -> String {
    color_str(s, Color::Yellow)
}

pub fn cyan(s: &str) -> String {
    color_str(s, Color::Cyan)
}

pub fn blue(s: &str) -> String {
    color_str(s, Color::Blue)
}

pub fn magenta(s: &str) -> String {
    color_str(s, Color::Magenta)
}

pub fn gray(s: &str) -> String {
    color_str(s, Color::White)
}

pub fn bold(s: &str) -> String {
    if std::env::var("NO_COLOR").is_ok() {
        return s.to_string();
    }
    format!("\x1b[1m{}\x1b[0m", s)
}

pub fn dim(s: &str) -> String {
    if std::env::var("NO_COLOR").is_ok() {
        return s.to_string();
    }
    format!("\x1b[2m{}\x1b[0m", s)
}

fn color_str(s: &str, color: Color) -> String {
    if std::env::var("NO_COLOR").is_ok() {
        return s.to_string();
    }
    let code = match color {
        Color::Green => "32",
        Color::Red => "31",
        Color::Yellow => "33",
        Color::Blue => "34",
        Color::Magenta => "35",
        Color::Cyan => "36",
        Color::White => "37",
        Color::Black => "30",
        _ => return s.to_string(),
    };
    format!("\x1b[{}m{}\x1b[0m", code, s)
}
