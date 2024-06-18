use std::{any::TypeId, fmt, fmt::Display};
use std::collections::VecDeque;
use std::fmt::{Debug, format};
use std::io::Write;
use std::ops::Deref;
use std::str::FromStr;
use std::time::Duration;

use akton::prelude::*;
use akton::prelude::async_trait::async_trait;
use atty::is;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use console::{style, StyledObject, Term, Color as TermColor};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use tokio::sync::mpsc;
use tracing::{error, trace};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::messages::{Category, CommitEvent, CommitPending, CommitPosted, NotifyError, SubscribeBroker, SystemStarted};
use crate::models::{TEAL_11, TEAL_9, TEAL_12, CommitTypeTerminal, DescriptionTerminal, DimStatic, OidTerminal, OptionalScope, Scope, ScopeTerminal, SemVerImpactTerminal, GRASS_11, GRASS_12, TimeStampTerminal, RED_9, GRAY_10, TEAL_7, RED_11, GRAY_9, GRAY_7, GRAY_12, WHITE_PURE, GRAY_11, IsBreakingTerminal, CommitHeadingTerminal, CommitType, RepositoryTerminal, ACCENT, INSTRUCTIONS, BRAND_NAME, HR_COLOR, FilenameTerminal, PendingCommit, Commit};

const TAB_WIDTH: usize = 8; // You can set this to any number of spaces you want
const LIST_ROW: usize = 3;

#[akton_actor]
pub(crate) struct Scribe {
    broker: Context,
    stdout: Option<Term>,
    stderr: Option<Term>,
    tab: String,
    half_tab: String,
    events: VecDeque<CommitEvent>,
}

impl Scribe {
    pub(crate) async fn initialize(name: String, broker: Context) -> Context {
        let tab = " ".repeat(TAB_WIDTH);
        let half_tab = " ".repeat(TAB_WIDTH / 2);
        let actor_config = ActorConfig::new(name, None, None);
        let term = Term::stdout();
        term.set_title("Tangler Ai");


        let mut actor = Akton::<Scribe>::create_with_config(actor_config);
        actor.state.broker = broker;
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
                // Determine the appropriate ColorChoice for stdout and stderr
                if let Some(stderr) = &actor.state.stderr {
                    stderr.clear_screen();

                    stderr.move_cursor_to(0, LIST_ROW - 2).unwrap();
                    // Write colored text to stderr using termcolor
                    let tangler = "Tangler".style(BRAND_NAME.clone());
                    let instructions = "Ctrl-C (Stop)".style(INSTRUCTIONS.clone().clone());
                    let display = format!("{}{tangler} {instructions}", actor.state.half_tab);
                    stderr.write_line(&display);
                    actor.state.hr();
                }
            });

        let subscription = SubscribeBroker {
            subscriber_id: actor.key.value.clone(),
            message_type_id: TypeId::of::<CommitEvent>(),
            subscriber_context: actor.context.clone(),
        };
        trace!(type_id=?TypeId::of::<CommitEvent>(),subscriber=actor.key.value.clone(),"Subscribed to CommitEvent:");
        actor.state.broker.emit_async(subscription, None).await;

        let subscription = SubscribeBroker {
            subscriber_id: actor.key.value.clone(),
            message_type_id: TypeId::of::<SystemStarted>(),
            subscriber_context: actor.context.clone(),
        };
        trace!(type_id=?TypeId::of::<SystemStarted>(),subscriber=actor.key.value.clone(),"Subscribed to SystemStarted:");

        actor.state.broker.emit_async(subscription, None).await;

        let context = actor
            .activate(None)
            .await
            .expect("Failed to activate Scribe. This should never happen, and yet, here we are.");
        trace!(id = &context.key.value, "Activated Scribe:");
        context
    }
    fn hr(&self) {
        if let Some(stdout) = &self.stderr {
            let hr = "-".repeat(75);

            eprintln!("{}{}", &self.half_tab, hr.style(HR_COLOR.clone()));
        }
    }
}
