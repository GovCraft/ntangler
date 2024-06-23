use std::collections::VecDeque;
use std::fmt::{format, Debug};
use std::io::{stderr, Write};
use std::ops::Deref;
use std::str::FromStr;
use std::time::Duration;
use std::{any::TypeId, fmt, fmt::Display};

use akton::prelude::*;
use atty::is;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use console::{style, Color as TermColor, StyledObject, Term};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use tokio::sync::mpsc;
use tracing::*;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::messages::{
    CommitEvent, CommitEventCategory, CommitPending, CommitPosted, NotifyError, SubscribeBroker,
    SystemStarted,
};
use crate::models::{
    Commit, CommitHeadingTerminal, CommitType, CommitTypeTerminal, DescriptionTerminal, DimStatic,
    FilenameTerminal, IsBreakingTerminal, OidTerminal, OptionalScope, PendingCommit,
    RepositoryTerminal, Scope, ScopeTerminal, SemVerImpactTerminal, TimeStampTerminal, ACCENT,
    BRAND_NAME, GRASS_11, GRASS_12, GRAY_1, GRAY_10, GRAY_11, GRAY_12, GRAY_2, GRAY_7, GRAY_9,
    HR_COLOR, INSTRUCTIONS, RED_11, RED_9, TEAL_11, TEAL_12, TEAL_7, TEAL_9, WHITE_PURE,
};

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
        let actor_config = ActorConfig::new(Arn::with_root(name).unwrap(), None, Some(broker))
            .expect("Failed to create Scribe config");
        let term = Term::stdout();
        term.set_title("Tangler Ai");
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
                let error_message = &event.message.error_message;
                tracing::warn!("Displayed user error: {:?}", error_message);
                if let Some(stderr) = &actor.state.stderr {
                    stderr.write_line(error_message);
                }
            })
            .act_on::<CommitEvent>(|actor, event| {
                let event_id = &event.message.id;
                let mut replaced = false;

                for existing_event in actor.state.events.iter_mut() {
                    if existing_event.id == *event_id {
                        *existing_event = event.message.clone();
                        replaced = true;
                        break;
                    }
                }

                if !replaced {
                    actor.state.events.push_front(event.message.clone());
                }

                if actor.state.events.len() > 10 {
                    actor.state.events.pop_back();
                }

                if let Some(stderr) = &actor.state.stderr {
                    stderr.move_cursor_to(0, LIST_ROW).unwrap();

                    Scribe::clear_console();

                    let mut events: Vec<_> = actor.state.events.clone().drain(..).collect();
                    events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

                    for event in &events {
                        stderr.clear_line();
                        stderr.write_line(event.to_string().as_str()).unwrap();
                    }
                }
            })
            .act_on::<SystemStarted>(|actor, _event| {
                Self::print_hero_message(&actor.state);
            })
            .on_before_stop(|actor| {
                if let Some(stderr) = &actor.state.stderr {
                    stderr.show_cursor().expect("Failed to re-show cursor");
                }
            });

        actor.context.subscribe::<CommitEvent>().await;
        actor.context.subscribe::<SystemStarted>().await;

        actor.activate(None).await
    }

    fn clear_console() {
        let term = Term::stderr();
        term.clear_to_end_of_screen();
    }

    #[instrument]
    fn print_hero_message(scribe: &Scribe) {
        if let Some(stderr) = &scribe.stderr {
            trace!("printing hero");
            stderr.clear_screen();

            stderr.move_cursor_to(0, LIST_ROW - 2).unwrap();

            let tangler = "Tangler".style(*BRAND_NAME);
            let instructions = "Ctrl-C (Stop)".style(*INSTRUCTIONS);
            let display = format!("{}{tangler} {instructions}", &scribe.half_tab);
            stderr.write_line(&display);
            scribe.print_horizontal_rule();
        }
    }
    fn print_horizontal_rule(&self) {
        if let Some(stderr) = &self.stderr {
            let hr = "-".repeat(75);
            eprintln!("{}{}", self.half_tab, hr.style(*HR_COLOR));
        }
    }
}
