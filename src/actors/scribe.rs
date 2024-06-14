use std::{any::TypeId, fmt, fmt::Display};
use std::fmt::Debug;
use std::io::Write;
use std::ops::Deref;
use std::str::FromStr;

use akton::prelude::*;
use akton::prelude::async_trait::async_trait;
use atty::is;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use console::{style, StyledObject, Term, Color as TermColor};
use owo_colors::OwoColorize;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use tokio::sync::mpsc;
use tracing::{error, trace};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::messages::{CommitSuccess, NotifyError, SubscribeBroker, SystemStarted};
use crate::models::{TEAL_11, TEAL_9, TEAL_12, CommitTypeTerminal, DescriptionTerminal, DimStatic, OidTerminal, OptionalScope, Scope, ScopeTerminal, SemVerImpactTerminal, GRASS_11, GRASS_12, TimeStampTerminal, RED_9, GRAY_10, TEAL_7, RED_11, GRAY_9, GRAY_7, GRAY_12, WHITE_PURE, GRAY_11, IsBreakingTerminal, CommitHeadingTerminal, CommitType, RepositoryTerminal, ACCENT, INSTRUCTIONS, BRAND_NAME, HR_COLOR};

const TAB_WIDTH: usize = 8; // You can set this to any number of spaces you want

#[akton_actor]
pub(crate) struct Scribe {
    broker: Context,
    stdout: Option<Term>,
    stderr: Option<Term>,
    tab: String,
    half_tab: String,
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
            .act_on::<CommitSuccess>(|actor, event| {

                let tab = &actor.state.tab;
                //format user message
                let half_tab = &actor.state.half_tab;

                let commit = event.message.commit();
                let repository = &commit.repository;
                let oid = &commit.oid;
                let description = &commit.description;
                let scope = &commit.scope;
                let time_stamp = &commit.time_stamp;
                let commit_type = &commit.commit_type;
                let semver_impact = &commit.semver_impact;
                let is_breaking = &commit.is_breaking;

                // convert to terminal formatted versions
                let oid: OidTerminal = oid.into();
                let description: DescriptionTerminal = description.into();
                let time_stamp: TimeStampTerminal = time_stamp.into();
                let semver_impact: SemVerImpactTerminal = semver_impact.into();
                let commit_heading: (CommitTypeTerminal, ScopeTerminal, IsBreakingTerminal) = (commit_type.into(), scope.into(), is_breaking.into());
                let commit_heading: CommitHeadingTerminal = commit_heading.into();
                let repository: RepositoryTerminal = repository.into();

                eprintln!("{half_tab}{repository} {time_stamp} {oid} {semver_impact} {commit_heading} {description}");
            })
            .act_on::<SystemStarted>(|actor, event| {
                // Determine the appropriate ColorChoice for stdout and stderr

                // Write colored text to stderr using termcolor
                let tangler = "Tangler".style(BRAND_NAME.clone());
                let instructions = " Ctrl-C (Stop)".style(INSTRUCTIONS.clone().clone());
                eprintln!( "{}{tangler}{instructions}", actor.state.half_tab);
                actor.state.hr();
            });

        let subscription = SubscribeBroker {
            subscriber_id: actor.key.value.clone(),
            message_type_id: TypeId::of::<CommitSuccess>(),
            subscriber_context: actor.context.clone(),
        };
        trace!(type_id=?TypeId::of::<CommitSuccess>(),subscriber=actor.key.value.clone(),"Subscribed to CommitSuccess:");

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
    fn style_time(datetime: &str) -> StyledObject<String> {
        let parts: Vec<&str> = datetime.splitn(3, ' ').collect();
        if parts.len() == 3 {
            let prefix = parts[0];
            let date = parts[1];
            let time = parts[2];

            // Parse the date and time
            let datetime_str = format!("{} {}", date, time);
            let naive_datetime =
                NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S").unwrap();
            let local_datetime = Local.from_local_datetime(&naive_datetime).unwrap();
            let now = Local::now();

            // Calculate the relative time
            let duration = now.signed_duration_since(local_datetime);

            // Style the time and date
            let styled_time = style(time).color256(250).to_string();
            let styled_date = style(date).color256(250).to_string();
            //let styled_relative_time = style(relative_time).color256(204).to_string();

            // Format the final output
            let styled_datetime = format!("{} {}", styled_date, styled_time);
            style(styled_datetime)
        } else {
            style(datetime.to_string())
        }
    }
    fn style_commit_msg(msg: &str) -> StyledObject<String> {
        if let Some((prefix, rest)) = msg.split_once(':') {
            let styled_prefix = style(prefix).color256(31).to_string();
            let styled_msg = format!("{}:{}", styled_prefix, style(rest).color256(253));
            style(styled_msg)
        } else {
            style(msg.to_string())
        }
    }

    fn print_indented_message(msg: impl AsRef<str> + Display) {
        let msg_str = msg.as_ref();
        let formatted_msg = format!("{:>width$}", msg_str, width = msg_str.len() + 0);
        eprintln!("{}", formatted_msg);
    }
    fn print_commit_message(msg: impl AsRef<str> + Display) {
        let msg = format!("[{}] {}", style("Tangler").color256(253), msg);
        eprintln!("{}", msg);
    }


}
