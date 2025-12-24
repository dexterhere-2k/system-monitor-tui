use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use sysinfo::System;
use std::{io, time::Duration};

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut sys = System::new_all();

    loop {
        sys.refresh_all();
        
        let cpu_usage = sys.global_cpu_info().cpu_usage();
        
        let total_mem = sys.total_memory() as f64;
        let used_mem = sys.used_memory() as f64;
        let mem_usage_percent = (used_mem / total_mem) * 100.0;

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(5),
                ])
                .split(f.size());

            let cpu_gauge = Gauge::default()
                .block(Block::default().title(" CPU Usage ").borders(Borders::ALL))
                .gauge_style(Style::default().fg(get_usage_color(cpu_usage)))
                .percent(cpu_usage as u16);
            f.render_widget(cpu_gauge, chunks[0]);

            let ram_gauge = Gauge::default()
                .block(Block::default().title(" Memory Usage ").borders(Borders::ALL))
                .gauge_style(Style::default().fg(get_usage_color(mem_usage_percent as f32)))
                .percent(mem_usage_percent as u16);
            f.render_widget(ram_gauge, chunks[1]);

            let mut processes: Vec<_> = sys.processes().values().collect();
            processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());

            let rows: Vec<_> = processes.iter().take(10).map(|p| {
                ratatui::widgets::Row::new(vec![
                    p.pid().to_string(),
                    p.name().to_string(),
                    format!("{:.1}%", p.cpu_usage()),
                ])
            }).collect();

            let table = ratatui::widgets::Table::new(rows, [
                    Constraint::Percentage(20),
                    Constraint::Percentage(50),
                    Constraint::Percentage(30),
                ])
                .header(ratatui::widgets::Row::new(vec!["PID", "Name", "CPU%"]).style(Style::default().fg(Color::Yellow)))
                .block(Block::default().title(" Top Processes ").borders(Borders::ALL));

            f.render_widget(table, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn get_usage_color(usage: f32) -> Color {
    if usage < 50.0 { Color::Green }
    else if usage < 80.0 { Color::Yellow }
    else { Color::Red }
}