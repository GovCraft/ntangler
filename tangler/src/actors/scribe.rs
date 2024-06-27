use std::cmp::PartialEq;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::io::{stderr, Write};
use std::time::Duration;
use std::{any::TypeId, fmt::Display};
use std::sync::Arc;

use akton::prelude::*;
use atty::is;
use chrono::{DateTime, Local, TimeZone, Utc};
use console::{pad_str, style, Alignment, Term};
use indicatif::{ProgressBar, ProgressStyle, TermLike};
use owo_colors::OwoColorize;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use tokio::sync::mpsc;
use tracing::*;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::messages::{CommitPosted, DiffQueued, FinalizedCommit, GenerationStarted, NotifyError, SubscribeBroker, SystemStarted};
use crate::models::CommittedCommit;
use crate::models::*;
use crate::models::{CommitTypeTerminal, MENU_COLOR};

const TAB_WIDTH: usize = 8;
const LIST_ROW: usize = 3;
const DISPLAY_WINDOW: usize = 11;

#[akton_actor]
#[derive(Clone)]
pub(crate) struct Scribe {
    stdout: Option<Term>,
    stderr: Option<Term>,
    tab: String,
    half_tab: String,
    events: VecDeque<AppEvent>,
    session_count: usize,
    session_recommendation: SemVerImpact,
}

impl Scribe {
    pub(crate) async fn initialize(name: String, system: &mut AktonReady) -> Context {
        let broker = system.get_broker().clone();
        let tab = " ".repeat(TAB_WIDTH);
        let half_tab = " ".repeat(TAB_WIDTH / 2);
        let actor_config = ActorConfig::new(Arn::with_root(name).unwrap(), None, Some(broker))
            .expect("Failed to create Scribe config");
        let term = Term::stdout();
        term.set_title("Tangler Ai Commits");
        term.hide_cursor();

        let mut actor = system
            .create_actor_with_config::<Scribe>(actor_config)
            .await;
        actor.state.stdout = Some(term.clone());
        actor.state.stderr = Some(Term::stderr());
        actor.state.tab = tab;
        actor.state.half_tab = half_tab;

        actor
            .setup
            .act_on::<NotifyError>(|actor, event| {
                Scribe::handle_notify_error(&mut actor.state, &event.message.error_message);
            })
            .act_on::<DiffQueued>(|actor, event| {
                let msg = event.message.clone();
                let app_event: AppEvent = msg.into();
                Scribe::handle_commit_event(&mut actor.state, &app_event);
            })
            .act_on::<GenerationStarted>(|actor, event| {
                let msg = event.message.clone();
                let app_event: AppEvent = msg.into();
                Scribe::handle_commit_event(&mut actor.state, &app_event);
            })
            .act_on::<FinalizedCommit>(|actor, event| {
                let msg = event.message.clone();
                let app_event: AppEvent = msg.clone().into();
                actor.state.session_recommendation = std::cmp::max(
                    msg.commit_message.semver_impact.clone(),
                    actor.state.session_recommendation.clone(),
                );
                // Update session_count to reflect the true number of Posted events
                actor.state.session_count += 1;
                Scribe::handle_commit_event(&mut actor.state, &app_event);
            })
            .act_on::<SystemStarted>(|actor, _event| {
                Scribe::handle_system_started(&mut actor.state);
            })
            .on_before_stop(|actor| {
                actor
                    .state
                    .stderr
                    .as_ref()
                    .unwrap()
                    .show_cursor()
                    .expect("Failed to re-show cursor");
            });

        actor.context.subscribe::<SystemStarted>().await;
        actor.context.subscribe::<DiffQueued>().await;
        actor.context.subscribe::<GenerationStarted>().await;
        actor.context.subscribe::<FinalizedCommit>().await;

        actor.activate(None).await
    }

    fn handle_notify_error(actor: &mut Scribe, error_message: &str) {
        warn!("Displayed user error: {:?}", error_message);
        if let Some(stderr) = &actor.stderr {
            stderr.write_line(error_message).unwrap();
        }
    }

    fn handle_system_started(actor: &mut Scribe) {
        Scribe::clear_console();
        Scribe::print_headings(&actor);
    }


    fn handle_commit_event(scribe: &mut Scribe, event: &AppEvent) {
        let previous_events = scribe.events.clone();
        // Update events or add new ones
        if let Some(existing_event) = scribe.events.iter_mut().find(|e| e.get_id() == event.get_id()) {
            *existing_event = event.clone();
        } else {
            scribe.events.push_front(event.clone());
        }

        scribe.truncate_events();

        // Update the display for changed events only
        scribe.update_changed_events(&previous_events, &scribe.events);
        scribe.print_menu();
    }

    fn truncate_events(&mut self) {
        while self.events.len() > DISPLAY_WINDOW {
            self.events.pop_back();
        }
    }

    fn update_changed_events(
        &self,
        previous_events: &VecDeque<AppEvent>,
        current_events: &VecDeque<AppEvent>,
    ) {
        if let Some(stderr) = &self.stderr {
            for i in 0..current_events.len() {
                if current_events.get(i) != previous_events.get(i) {
                    stderr.move_cursor_to(0, LIST_ROW + i).unwrap();
                    stderr.clear_line().unwrap();
                    stderr.write_line(&current_events[i].to_string()).unwrap();
                }
            }
        }
    }

    fn clear_console() {
        Term::stderr().clear_to_end_of_screen().unwrap();
    }

    fn print_headings(&self) {
        if let Some(stderr) = &self.stderr {
            stderr.move_cursor_to(0, LIST_ROW - 2).unwrap();
            let display = format!(
                "{}{:^COLUMN_HEADING_ONE_LENGTH$} \
                {:^COLUMN_HEADING_TWO_LENGTH$} \
                {:^COLUMN_HEADING_THREE_LENGTH$}\
                {:^COLUMN_HEADING_FOUR_LENGTH$} \
                {:^COLUMN_HEADING_FIVE_LENGTH$} \
                {:^COLUMN_HEADING_SIX_LENGTH$} \
                {:^COLUMN_HEADING_SEVEN_LENGTH$}",
                self.half_tab,
                COLUMN_HEADING_ONE,
                COLUMN_HEADING_TWO,
                COLUMN_HEADING_THREE,
                COLUMN_HEADING_FOUR,
                COLUMN_HEADING_FIVE,
                COLUMN_HEADING_SIX,
                COLUMN_HEADING_SEVEN
            );
            stderr.write_line(display.as_str()).unwrap();
            self.print_horizontal_rule();
        }
    }

    fn print_horizontal_rule(&self) {
        if let Some(stderr) = &self.stderr {
            let (_, terminal_columns) = Term::stderr().size();
            let canvas_length = terminal_columns - (TAB_WIDTH as u16);
            let hr = "-".repeat(canvas_length as usize);
            stderr
                .write_line(&format!("{}{}", self.half_tab, hr.style(*HR_COLOR)))
                .unwrap();
        }
    }

    fn print_menu(&self) {
        if let Some(stderr) = &self.stderr {
            stderr.move_cursor_to(0, 2 + DISPLAY_WINDOW);
            self.print_horizontal_rule();
            stderr.write_line(&self.format_footer()).unwrap();
            stderr.clear_to_end_of_screen().unwrap()
        }
    }

    fn format_footer(&self) -> String {
        let instructions_text = "Ctrl-C (Stop)".style(*PALETTE_NEUTRAL_11).to_string();
        let (_, screen_width) = Term::stderr().size();
        let semver_recommendation_text_binding: SemVerImpactTerminal =
            (&self.session_recommendation).into();
        let semver_recommendation_text_binding = semver_recommendation_text_binding.to_string();
        let semver_recommendation_text = pad_str(
            &semver_recommendation_text_binding,
            7,
            Alignment::Center,
            None,
        );
        let semver_label_text = "Max SemVer:".style(*PALETTE_NEUTRAL_11).to_string();
        let session_commits_label_text = "Session commits:".style(*PALETTE_NEUTRAL_11).to_string();
        let session_commits_text = format!("{:<5}", self.session_count.style(*COMMIT_COUNT));
        let copyright_text = format!(
            "\u{00A9} Govcraft 2024 Tangler v{}",
            env!("CARGO_PKG_VERSION")
        );

        let canvas_width = screen_width as usize - TAB_WIDTH;
        let instructions_width = self.calculate_instructions_width(
            canvas_width,
            &session_commits_label_text,
            &session_commits_text,
            &copyright_text,
            &semver_label_text,
            &semver_recommendation_text,
        );
        let instructions_text = pad_str(
            &instructions_text,
            instructions_width,
            Alignment::Left,
            None,
        );

        format!(
            "{}{semver_label_text}{semver_recommendation_text} {session_commits_label_text} {session_commits_text} {instructions_text}{copyright_text}",
            self.half_tab
        )
    }

    fn calculate_remaining_width(
        &self,
        canvas_width: usize,
        session_commits_label_text: &str,
        session_commits_text: &str,
        semver_label_text: &str,
    ) -> usize {
        let total_length =
            session_commits_label_text.len() + session_commits_text.len() + semver_label_text.len();
        let remaining_width = canvas_width.saturating_sub(total_length);

        if remaining_width == 0 {
            let adjusted_length = session_commits_label_text.len() + semver_label_text.len();
            canvas_width.saturating_sub(adjusted_length)
        } else {
            remaining_width
        }
    }

    fn calculate_instructions_width(
        &self,
        canvas_width: usize,
        session_commits_label_text: &str,
        session_commits_text: &str,
        copyright_text: &str,
        semver_label_text: &str,
        semver_recommendation_text: &str,
    ) -> usize {
        // Calculate total text length and remaining width
        let remaining_width = self.calculate_remaining_width(
            canvas_width,
            session_commits_label_text,
            session_commits_text,
            semver_label_text,
        );

        trace!("Remaining width without adjustments: {}", remaining_width);

        if remaining_width == 0 {
            let adjusted_total_length = session_commits_label_text.len() + semver_label_text.len();
            let adjusted_width = canvas_width.saturating_sub(adjusted_total_length);
            trace!("Width after adjustment: {}", adjusted_width);
            return adjusted_width;
        }

        remaining_width
    }
}
