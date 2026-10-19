use std::time::{Duration, Instant};

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};
pub struct GameStats {
    started: bool,
    active_since: Option<Instant>,
    elapsed: Duration,
    blocks_placed: usize,
    lines_cleared: usize,
}


//this is the most complex game score calculation ever ok dont question me and shi
impl GameStats {
    pub fn new() -> Self {
        Self {
            started: false,
            active_since: None,
            elapsed: Duration::ZERO,
            blocks_placed: 0,
            lines_cleared: 0,
        }
    }

    pub fn start(&mut self, now: Instant) {
        if !self.started {
            self.started = true;
            self.active_since = Some(now);
        }
    }

    pub fn pause(&mut self, now: Instant) {
        if let Some(active_since) = self.active_since.take() {
            self.elapsed += now.duration_since(active_since);
        }
    }

    pub fn resume(&mut self, now: Instant) {
        if self.active_since.is_none() {
            self.active_since = Some(now);
        }
    }

    pub fn stop(&mut self, now: Instant) {
        self.pause(now);
    }

    pub fn elapsed(&self, now: Instant) -> Duration {
        self.elapsed
            + self
                .active_since
                .map_or(Duration::ZERO, |start| now.duration_since(start))
    }

    pub fn is_started(&self) -> bool {
        self.started
    }

    pub fn record_block_placed(&mut self) {
        self.blocks_placed += 1;
    }

    pub fn record_lines_cleared(&mut self, rows: usize) {
        self.lines_cleared += rows;
    }

    pub fn blocks_placed(&self) -> usize {
        self.blocks_placed
    }

    pub fn lines_cleared(&self) -> usize {
        self.lines_cleared
    }

    pub fn score(&self, now: Instant) -> u64 {
        let seconds = self.elapsed(now).as_secs();
        seconds * 100 + self.blocks_placed as u64 * 250 + self.lines_cleared as u64 * 1_000 //main logic if u wanna change ,change this 
    }
}

fn format_score(score: u64) -> String {
    let digits = score.to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(digit);
    }
    formatted
}


//score display
pub fn draw_stats(
    frame: &mut Frame<'_>,
    area: Rect,
    stats: &GameStats,
    now: Instant,
    paused: bool,
    compact: bool,
) {
    let elapsed_seconds = stats.elapsed(now).as_secs();
    let score = format_score(stats.score(now));
    let time_text = format!("{:02}:{:02}", elapsed_seconds / 60, elapsed_seconds % 60);
    let title = if paused { " stats paused " } else { " stats " };
    let text = if compact {
        format!(
            "SCORE  {score}\nTIME   {time_text}\nBLOCKS {}  LINES {}\n←/→ move  ↓ drop  p pause\n1-7 pieces  g/d mode  q quit",
            stats.blocks_placed(),
            stats.lines_cleared()
        )
    } else {
        format!(
            "SCORE\n{score}\n\nTIME\n{time_text}\n\nBLOCKS PLACED\n{}\n\nLINES CLEARED\n{}",
            stats.blocks_placed(),
            stats.lines_cleared()
        )
    };
    let panel = Paragraph::new(text)
        .style(Style::default().fg(Color::Green).bg(Color::Black))
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );
    frame.render_widget(panel, area);
}
