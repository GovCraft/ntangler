use std::{any::TypeId, fmt, fmt::Display};
use std::collections::VecDeque;
use std::fmt::{Debug, format};
use std::io::Write;
use std::ops::Deref;
use std::str::FromStr;
use std::time::Duration;

use akton::prelude::*;
use atty::is;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use console::{Color as TermColor, style, StyledObject, Term};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use tokio::sync::mpsc;
use tracing::*;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::messages::{CommitEventCategory, CommitEvent, CommitPending, CommitPosted, NotifyError, SubscribeBroker, SystemStarted};
use crate::models::{ACCENT, BRAND_NAME, Commit, CommitHeadingTerminal, CommitType, CommitTypeTerminal, DescriptionTerminal, DimStatic, FilenameTerminal, GRASS_11, GRASS_12, GRAY_10, GRAY_11, GRAY_12, GRAY_7, GRAY_9, HR_COLOR, INSTRUCTIONS, IsBreakingTerminal, OidTerminal, OptionalScope, PendingCommit, RED_11, RED_9, RepositoryTerminal, Scope, ScopeTerminal, SemVerImpactTerminal, TEAL_11, TEAL_12, TEAL_7, TEAL_9, TimeStampTerminal, WHITE_PURE};

const TAB_WIDTH: usize = 8; // You can set this to any number of spaces you want
const LIST_ROW: usize = 3;

#[akton_actor]
pub(crate) struct Scribe {
    stdout: Option<Term>,
    stderr: Option<Term>,
    tab: String,
    half_tab: String,
    events: VecDeque<CommitEvent>,
}

impl Scribe {
    pub(crate) async fn initialize(name: String, system: &mut AktonReady) -> Context {
        let broker = system.get_broker().clone();
        let tab = " ".repeat(TAB_WIDTH);
        let half_tab = " ".repeat(TAB_WIDTH / 2);
        let actor_config = ActorConfig::new(Arn::with_root(name).unwrap(), None, Some(broker)).expect("Failed to create Scribe config");
        let term = Term::stdout();
        term.set_title("Tangler Ai");
        term.hide_cursor();


        let mut actor = system.create_actor_with_config::<Scribe>(actor_config).await;
        actor.state.stdout = Some(term);
        actor.state.stderr = Some(Term::stderr());
        actor.state.tab = tab;
        actor.state.half_tab = half_tab;
        // Setting up handlers for different message types

        actor
            .setup
            .act_on::<NotifyError>(|actor, event| {
                let error_message = &event.message.error_message;
                tracing::warn!("Displayed user error: {:?}", &error_message);
                if let Some(stderr) = &actor.state.stderr {
                    stderr.write_line(&format!("{}", error_message));
                }
            })
            .act_on::<CommitEvent>(|actor, event| {
                let event_id = &event.message.id;
                // Check if an event with the same ID already exists in the queue
                let mut replaced = false;
                for existing_event in actor.state.events.iter_mut() {
                    let existing_id = &existing_event.id;
                    if existing_id == event_id {
                        *existing_event = event.message.clone();
                        replaced = true;
                        break;
                    }
                }

                // If no existing event was replaced, push the new event to the front of the deque
                if !replaced {
                    actor.state.events.push_front(event.message.clone());
                }

                // If the deque exceeds 10 events, pop the back (oldest) event
                if actor.state.events.len() > 10 {
                    actor.state.events.pop_back();
                }

                // Clear the console before printing
                if let Some(stderr) = &actor.state.stderr {
                    // Move the cursor to the top-left corner
                    stderr.move_cursor_to(0, LIST_ROW).unwrap();

                    // Clear the screen
                    stderr.clear_to_end_of_screen();
                    // Convert VecDeque to Vec
                    let mut events: Vec<_> = actor.state.events.clone().drain(..).collect();

                    // Sort the Vec by timestamp from newest to oldest
                    events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

                    // Print only the last 10 events
                    for event in &events {
                        stderr.clear_line();
                        stderr.write_line(event.to_string().as_str()).unwrap();
                    }
                }
            })
            .act_on::<SystemStarted>(|actor, event| {
                let scribe = &actor.state;
                Self::print_hero_message(&scribe);
            });
        actor.context.subscribe::<CommitEvent>().await;
        actor.context.subscribe::<SystemStarted>().await;

        let context = actor
            .activate(None)
            .await;
        trace!(id = &context.key, "Activated Scribe:");
        context
    }

    #[instrument]
    fn print_hero_message(scribe: &&Scribe) {
        if let Some(stderr) = &scribe.stderr {
            trace!("printing hero");
            stderr.clear_screen();

            stderr.move_cursor_to(0, LIST_ROW - 2).unwrap();

            let tangler = "Tangler".style(BRAND_NAME.clone());
            let instructions = "Ctrl-C (Stop)".style(INSTRUCTIONS.clone().clone());
            let display = format!("{}{tangler} {instructions}", &scribe.half_tab);
            stderr.write_line(&display);
            scribe.hr();
        }
    }
    fn hr(&self) {
        if let Some(stdout) = &self.stderr {
            let hr = "-".repeat(75);

            eprintln!("{}{}", &self.half_tab, hr.style(HR_COLOR.clone()));
        }
    }
}

