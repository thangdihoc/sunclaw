use std::io;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::sync::Arc;
use sunclaw_runtime::Runtime;

pub struct TuiApp {
    pub input: String,
    pub messages: Vec<String>,
    pub logs: Vec<String>,
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            messages: vec!["🤖 Xin chào! Tôi là Sunclaw. Bạn cần giúp gì?".to_string()],
            logs: vec!["[System] Sunclaw đã sẵn sàng...".to_string()],
        }
    }
}

pub async fn run_tui(_runtime: Arc<Runtime>) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = TuiApp::new();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
                .split(f.size());

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
                .split(chunks[0]);

            // Trình bày Chat
            let items: Vec<ListItem> = app.messages.iter()
                .map(|m| ListItem::new(m.as_str()))
                .collect();
            let chat = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("🗨️ Chat"));
            f.render_widget(chat, main_chunks[0]);

            // Trình bày Logs (Trace)
            let log_items: Vec<ListItem> = app.logs.iter()
                .map(|l| ListItem::new(l.as_str()))
                .collect();
            let logs = List::new(log_items)
                .block(Block::default().borders(Borders::ALL).title("🔍 Live Trace"));
            f.render_widget(logs, main_chunks[1]);

            // Ô nhập liệu
            let input = Paragraph::new(app.input.as_str())
                .block(Block::default().borders(Borders::ALL).title("Nhập câu hỏi (Esc để thoát)"));
            f.render_widget(input, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => { app.input.pop(); },
                    KeyCode::Enter => {
                        let query = app.input.drain(..).collect::<String>();
                        app.messages.push(format!("👤 Bạn: {}", query));
                        app.logs.push(format!("[Agent] Đang xử lý: {}", query));
                        
                        // Fake async execution call here (trình diễn UI)
                        // Trong thực tế sẽ gọi runtime.run_once
                        app.messages.push("🤖 Sunclaw đang suy nghĩ...".to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
