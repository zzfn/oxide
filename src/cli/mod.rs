pub mod command;
pub mod render;

use anyhow::Result;
use colored::*;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{self, MatchingBracketValidator, Validator};
use rustyline::Editor;
use rustyline::{Context, Helper};
use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashSet;

use crate::context::ContextManager;

// 自定义补全器
pub struct OxideHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    commands: HashSet<String>,
}

impl Default for OxideHelper {
    fn default() -> Self {
        let mut commands = HashSet::new();
        commands.insert("/quit".to_string());
        commands.insert("/exit".to_string());
        commands.insert("/clear".to_string());
        commands.insert("/config".to_string());
        commands.insert("/help".to_string());
        commands.insert("/toggle-tools".to_string());
        commands.insert("/history".to_string());
        commands.insert("/load".to_string());
        commands.insert("/sessions".to_string());
        commands.insert("/delete".to_string());

        Self {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            validator: MatchingBracketValidator::new(),
            hinter: HistoryHinter {},
            commands,
        }
    }
}

impl Completer for OxideHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        if line.starts_with('/') {
            let input = &line[..pos];
            let mut matches = Vec::new();

            for command in &self.commands {
                if command.starts_with(input) {
                    matches.push(Pair {
                        display: command.clone(),
                        replacement: command.clone(),
                    });
                }
            }

            matches.sort_by(|a, b| a.display.cmp(&b.display));

            Ok((0, matches))
        } else {
            Ok((pos, vec![]))
        }
    }
}

impl Hinter for OxideHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for OxideHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(prompt)
        } else {
            Owned(prompt.to_string())
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(format!("{}", hint.dimmed()))
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        if line.starts_with('/') {
            if let Some(space_pos) = line.find(' ') {
                let command = &line[..space_pos];
                let rest = &line[space_pos..];
                if self.commands.contains(command) {
                    return Owned(format!("{}{}", command.bright_green(), rest));
                }
            } else if self.commands.contains(line) {
                return Owned(line.bright_green().to_string());
            }
        }

        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
    }
}

impl Validator for OxideHelper {
    fn validate(
        &self,
        ctx: &mut validate::ValidationContext,
    ) -> rustyline::Result<validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

impl Helper for OxideHelper {}

pub const LOGO: &str = r#"
 _______          _________ ______   _______
(  ___  )|\     /|\__   __/(  __  \ (  ____ \
| (   ) |( \   / )   ) (   | (  \  )| (    \/
| |   | | \ (_) /    | |   | |   ) || (__
| |   | |  ) _ (     | |   | |   | ||  __)
| |   | | / ( ) \    | |   | |   ) || (
| (___) |( /   \ )___) (___| (__/  )| (____/\
(_______)|/     \|\_______/(______/ (_______/
"#;

pub struct OxideCli {
    pub context_manager: ContextManager,
}

impl OxideCli {
    pub fn new(context_manager: ContextManager) -> Self {
        Self { context_manager }
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("{}", LOGO);
        self.show_welcome()?;
        self.show_tips()?;

        let result = self.run_input_loop().await;

        match result {
            Ok(_) => println!("\n{}", "👋 Goodbye!".bright_cyan()),
            Err(e) => {
                println!("\n{} {}", "❌ Error:".red(), e);
                return Err(e);
            }
        }

        Ok(())
    }

    async fn run_input_loop(&mut self) -> Result<()> {
        let mut rl = Editor::new()?;
        rl.set_helper(Some(OxideHelper::default()));

        loop {
            self.print_separator()?;
            let readline = rl.readline("❯ ");

            match readline {
                Ok(line) => {
                    let input = line.trim();
                    if input.is_empty() {
                        continue;
                    }

                    let _ = rl.add_history_entry(input);

                    self.print_separator()?;

                    let should_continue = self.handle_command(input).await?;
                    if !should_continue {
                        break;
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("{}", "^C".dimmed());
                    break;
                }
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    println!("{} {:?}", "Error:".red(), err);
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn print_separator(&self) -> Result<()> {
        let width = 80;
        let separator = "-".repeat(width);
        println!("{}", separator.dimmed());
        Ok(())
    }

    pub fn session_id(&self) -> &str {
        self.context_manager.session_id()
    }
}
