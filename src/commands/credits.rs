use std::{
    collections::VecDeque,
    fmt::Display,
    io::stdout,
    time::{Duration, Instant},
};

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use parking_lot::Mutex;
use ratatui::{
    backend::CrosstermBackend,
    layout::Alignment,
    style::{Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, Padding, Paragraph},
    Frame, Terminal,
};
use sprinkles::inline_const;

mod contributors {
    include!(concat!(env!("OUT_DIR"), "/contributors.rs"));
}

mod packages {
    include!(concat!(env!("OUT_DIR"), "/packages.rs"));
}

#[derive(Debug)]
struct Timer {
    timeout: Duration,
    current: Mutex<Duration>,
    now: Mutex<std::time::Instant>,
}

impl Timer {
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            current: Mutex::new(Duration::ZERO),
            now: Mutex::new(Instant::now()),
        }
    }

    pub fn tick(&self) -> bool {
        let now = Instant::now();
        let delta = now - *self.now.lock();
        *self.now.lock() = now;

        let mut current = self.current.lock();
        *current += delta;

        debug!("Delta: {:#?}", delta);
        debug!("Current: {:#?}", current);
        if *current >= self.timeout {
            *current = Duration::ZERO;
            true
        } else {
            false
        }
    }
}

struct Url<T: Display> {
    text: T,
    url: String,
}

impl<T: Display> Url<T> {
    fn new(text: T, url: String) -> Self {
        Self { text, url }
    }
}

impl<T: Display> Display for Url<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\",
            self.url, self.text
        )
    }
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(short, long, help = "Show packages")]
    packages: bool,
}

impl super::Command for Args {
    async fn runner(self) -> anyhow::Result<()> {
        self.terminal_ui()?;

        Ok(())
    }
}

impl Args {
    fn terminal_ui(&self) -> anyhow::Result<()> {
        const TITLE_STYLE: Style = Style::new().add_modifier(Modifier::BOLD);

        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let title = format!(
            "🚀 sfsu v{}, created by Juliette Cordor 🚀",
            env!("CARGO_PKG_VERSION")
        );

        let mut items = vec![
            Text::styled(
                "Press Q to exit",
                inline_const![
                    Style
                    Style::new().add_modifier(Modifier::ITALIC)
                ],
            ),
            Text::raw(""),
            Text::styled(
                "💖 Many thanks to all our incredible contributors 💖",
                TITLE_STYLE,
            ),
        ];

        items.extend(
            contributors::CONTRIBUTORS
                .into_iter()
                .map(|(name, url)| Text::from(format!("{name} ({url})"))),
        );

        if self.packages {
            items.extend(packages::PACKAGES.into_iter().map(|(name, version)| {
                let url = Url::new(name, format!("https://crates.io/crates/{name}"));
                Text::from(format!("{url}: {version}"))
            }));
        }

        let mut items: VecDeque<Text<'_>> = items.into();

        let (rows, _) = console::Term::stdout().size();

        let timer = if items.len() > rows as usize {
            Some(Timer::new(Duration::from_millis(343)))
        } else {
            None
        };

        let mut should_quit = false;
        while !should_quit {
            terminal.draw(|f| self.ui(f, timer.as_ref(), &title, &mut items))?;
            should_quit = self.handle_events()?;
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;

        Ok(())
    }

    fn handle_events(&self) -> anyhow::Result<bool> {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn ui(
        &self,
        frame: &mut Frame<'_>,
        timer: Option<&Timer>,
        title: &str,
        items: &mut VecDeque<Text<'_>>,
    ) {
        // println!();

        // if self.packages {
        //     println!("📦📦📦 sfsu is built with the following packages 📦📦📦");
        //     for (name, version) in packages::PACKAGES {
        //         let url = Url::new(name, format!("https://crates.io/crates/{name}"));
        //         println!("{url}: {version}");
        //     }

        //     println!();
        // }

        // println!("💖💖💖 Many thanks to everyone who as contributed to sfsu 💖💖💖");
        // for (name, url) in contributors::CONTRIBUTORS {
        //     let url = Url::new(name, url.to_string());

        //     println!("{url}");
        // }

        if let Some(timer) = timer {
            if timer.tick() {
                debug!("{:#?}", items.pop_front());
            }
        }

        frame.render_widget(
            List::new(
                items
                    .iter()
                    .cloned()
                    .map(|text| text.alignment(Alignment::Center)),
            )
            .block(
                Block::default()
                    .title(title)
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL),
            ),
            frame.size(),
        );
    }
}
