use std::fmt::format;

use clap::Parser;
use crossterm::event::{self, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, BorderType, Paragraph, Tabs};
use scoph::prelude::*;
use tui_prompts::{Prompt, State, TextPrompt, TextState};

use crate::commands::{ExtractError, ReplCommand, extract_command};

#[derive(Parser)]
struct Args {
    game: Option<String>,
}

mod commands;
mod history;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // let game_tree = BehaviorTree::load_from_json_file(&args.game)?;
    let mut intrp = match args.game {
        Some(path) => {
            let bt = BehaviorTree::load_from_json_file(path)?;
            Some(Interpreter::new(bt))
        }
        None => None,
    };

    ratatui::run(move |terminal| {
        let mut rh = history::ReplHistory::new();
        if let Some(ref mut intrp) = intrp {
            intrp.startup(&mut rh);
        }

        let mut prompt_state = TextState::new();
        prompt_state.focus();
        let mut prompt_output = String::new();
        let mut selected_tab = 0;
        loop {
            terminal.draw(|frame| {
                draw(
                    frame,
                    selected_tab,
                    &mut prompt_state,
                    &prompt_output,
                    intrp.as_mut(),
                )
            })?;

            let mut game_over = false;
            if let Some(ref mut intrp) = intrp {
                match intrp.step() {
                    Ok(finished) => game_over = finished,
                    Err(e) => {}
                }
            }

            if prompt_state.is_finished() {
                match extract_command(&mut prompt_state) {
                    Ok(cmd) => match cmd {
                        ReplCommand::Quit => return Ok(()),
                        ReplCommand::Load { path } => {
                            todo!()
                        }
                        c => {
                            if let Some(ref mut intrp) = intrp {
                                c.apply(intrp);
                            } else {
                                prompt_output = "Can't use command without a game".to_string();
                            }
                        }
                    },
                    Err(ExtractError::BadText) => {
                        prompt_output = "Input had a mismatched quote".to_string()
                    }
                    Err(ExtractError::ClapError(e)) => {
                        prompt_output = format!("Couldn't parse input into args: \"{e}\"")
                    }
                }
            }

            let ev = event::read()?;
            if let Some(key) = ev.as_key_press_event() {
                use crossterm::event::KeyCode as K;
                if key.modifiers == KeyModifiers::CONTROL {
                    match key.code {
                        K::Left => selected_tab = (selected_tab.checked_sub(1).unwrap_or(2)) % 3,
                        K::Right => selected_tab = (selected_tab + 1) % 3,
                        _ => {}
                    }
                } else if let K::Enter = key.code {
                    prompt_state.complete();
                } else {
                    prompt_state.handle_key_event(key);
                }
            }
        }
    })
}

fn draw(
    frame: &mut Frame,
    selected_tab: usize,
    prompt_state: &mut TextState,
    prompt_output: &str,
    intrp: Option<&mut Interpreter>,
) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            // tabs container with the board, tree, and stack
            Constraint::Percentage(80),
            Constraint::Fill(1),
            Constraint::Fill(2),
        ])
        .split(frame.area());

    let tabs = Tabs::new(["board", "tree", "stack"]).select(selected_tab);
    draw_content(frame, main_layout[0], selected_tab, intrp);
    frame.render_widget(tabs, main_layout[0]);

    let cmd = TextPrompt::from("🐠").with_block(Block::bordered().border_type(BorderType::Rounded));
    cmd.draw(frame, main_layout[1], prompt_state);

    let p = Paragraph::new(prompt_output).block(Block::bordered().border_type(BorderType::Rounded));
    frame.render_widget(p, main_layout[2]);
}

fn draw_content(
    frame: &mut Frame,
    area: Rect,
    selected_tab: usize,
    intrp: Option<&mut Interpreter>,
) {
    if let Some(intrp) = intrp {
        match selected_tab {
            0 => {
                draw_board(frame, area, intrp);
            }
            1 => {
                draw_tree(frame, area, intrp);
            }
            2 => {
                todo!()
            }
            _ => unreachable!(),
        }
    } else {
        let p = Paragraph::new(
            "There's no active game right now. Load one with \"load <path-to-game>\". For more information, try \"help <load>\"",
        ).centered().block(Block::bordered().border_type(BorderType::Double))
        .centered();
        frame.render_widget(p, area);
    }
}

fn draw_board(frame: &mut Frame, area: Rect, intrp: &mut Interpreter) {}

fn draw_tree(frame: &mut Frame, area: Rect, intrp: &mut Interpreter) {}
