use clap::Parser;
use crossterm::event::{self, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, Paragraph, Tabs};
use ratatui::{DefaultTerminal, Frame};
use scoph::prelude::*;
use tui_prompts::{FocusState, Prompt, State, TextPrompt, TextState};
use tui_tree_widget::TreeState;

use crate::commands::{ExtractError, ReplCommand, extract_command};
use crate::history::ReplHistory;
use crate::tree::TuiTrrbtTree;

#[derive(Parser)]
struct Args {
    game: Option<String>,
}

mod commands;
mod history;
mod tree;

enum PromptOutputStatus {
    Neutral,
    HadError,
    HadSuccess,
}

struct App<'a> {
    prompt_output: String,
    prompt_output_status: PromptOutputStatus,
    prompt_state: TextState<'a>,
    intrp: Option<Interpreter>,
    selected_tab: usize,
    should_quit: bool,
    tree_state: TreeState<String>,
    history: ReplHistory,
}

impl<'a> App<'a> {
    fn new(path: Option<String>) -> Self {
        let mut out = Self {
            prompt_output: Default::default(),
            prompt_output_status: PromptOutputStatus::Neutral,
            prompt_state: Default::default(),
            selected_tab: 0,
            should_quit: false,
            intrp: Default::default(),
            tree_state: Default::default(),
            history: ReplHistory::new(),
        };

        out.prompt_state.focus();

        if let Some(path) = path {
            out.try_to_load_game(&path);
        }

        out
    }

    fn try_to_load_game(&mut self, path: &str) {
        let bt = match BehaviorTree::load_from_json_file(path) {
            Ok(bt) => {
                self.prompt_output_status = PromptOutputStatus::HadSuccess;
                bt
            }
            Err(e) => {
                self.prompt_output_status = PromptOutputStatus::HadError;
                self.prompt_output = format!("Couldn't load file: {}", e);
                return;
            }
        };

        self.prompt_output = format!("Successfully loaded game at \"{}\"", path);
        self.intrp = Some(Interpreter::new(bt));
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        terminal.draw(|frame| self.draw_ui(frame))?;
        while !self.should_quit {
            self.handle_events()?;
            terminal.draw(|frame| self.draw_ui(frame))?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> anyhow::Result<()> {
        let Some(key) = event::read()?.as_key_press_event() else {
            return Ok(());
        };

        use crossterm::event::KeyCode as K;

        if key.modifiers == KeyModifiers::CONTROL {
            match key.code {
                K::Left => self.selected_tab = (self.selected_tab.checked_sub(1).unwrap_or(2)) % 3,
                K::Right => self.selected_tab = (self.selected_tab + 1) % 3,
                _ => {}
            }
        } else {
            match key.code {
                K::Enter => {
                    self.submit();
                }
                K::Esc => {
                    if self.prompt_state.is_focused() {
                        self.prompt_state.blur();
                    } else {
                        self.prompt_state.focus()
                    }
                }
                K::Left if !self.prompt_state.is_focused() && self.selected_tab == 1 => {
                    self.tree_state.key_left();
                }
                K::Right if !self.prompt_state.is_focused() && self.selected_tab == 1 => {
                    self.tree_state.key_right();
                }
                K::Up if !self.prompt_state.is_focused() && self.selected_tab == 1 => {
                    self.tree_state.key_up();
                }
                K::Down if !self.prompt_state.is_focused() && self.selected_tab == 1 => {
                    self.tree_state.key_down();
                }
                _ => {
                    if self.prompt_state.is_focused() {
                        self.prompt_state.handle_key_event(key)
                    }
                }
            }
        }

        Ok(())
    }

    fn submit(&mut self) {
        match extract_command(&mut self.prompt_state) {
            Ok(cmd) => {
                self.use_command(cmd);
            }
            Err(ExtractError::BadText) => {
                self.prompt_output_status = PromptOutputStatus::HadError;
                self.prompt_output = "Mismatched quote in input command".to_string()
            }
            Err(ExtractError::ClapError(mut e)) => {
                self.prompt_output_status = PromptOutputStatus::HadError;
                e.remove(clap::error::ContextKind::Usage);
                self.prompt_output = format!("{}", e);
            }
        }
    }

    fn use_command(&mut self, cmd: ReplCommand) {
        match cmd {
            ReplCommand::Quit => self.should_quit = true,
            ReplCommand::Load { path } => self.try_to_load_game(&path),
            c => {
                if let Some(ref mut intrp) = self.intrp {
                    c.apply(intrp);
                }
            }
        }
    }

    fn draw_ui(&mut self, frame: &mut Frame) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                // tabs container with the board, tree, and stack
                Constraint::Percentage(80),
                Constraint::Fill(1),
                Constraint::Fill(2),
            ])
            .split(frame.area());

        let tabs = Tabs::new(["board", "tree", "stack"]).select(self.selected_tab);
        self.draw_content(frame, main_layout[0]);
        frame.render_widget(tabs, main_layout[0]);

        let cmd =
            TextPrompt::from("🐠").with_block(Block::bordered().border_type(BorderType::Rounded));
        cmd.draw(frame, main_layout[1], &mut self.prompt_state);

        let mut output_style = Style::new();
        match self.prompt_output_status {
            PromptOutputStatus::Neutral => output_style = output_style.white(),
            PromptOutputStatus::HadError => output_style = output_style.red(),
            PromptOutputStatus::HadSuccess => output_style = output_style.green(),
        }
        let p = Paragraph::new(self.prompt_output.as_str())
            .style(output_style)
            .block(Block::bordered().border_type(BorderType::Rounded));
        frame.render_widget(p, main_layout[2]);
    }

    fn draw_content(&mut self, frame: &mut Frame, area: Rect) {
        if self.intrp.is_some() {
            match self.selected_tab {
                0 => {
                    self.draw_board(frame, area);
                }
                1 => {
                    self.draw_tree(frame, area);
                }
                2 => {
                    self.draw_stack(frame, area);
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

    fn draw_tree(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let intrp = self.intrp.as_mut().unwrap();
        frame.render_stateful_widget(
            TuiTrrbtTree::try_from(intrp.tree()).expect("Tree should be able to be serialized"),
            area,
            &mut self.tree_state,
        );
    }

    fn draw_board(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let intrp = self.intrp.as_mut().unwrap();
    }

    fn draw_stack(&mut self, frame: &mut Frame, area: Rect) {
        let intrp = self.intrp.as_mut().unwrap();
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut app = App::new(args.game);
    ratatui::run(|terminal| app.run(terminal))
}
