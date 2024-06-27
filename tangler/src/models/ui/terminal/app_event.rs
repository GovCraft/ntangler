// use crate::messages::commit_authoring::CommitAuthoring;
// use crate::messages::CommitPosted;
use crate::messages::{DiffQueued, FinalizedCommit, GenerationStarted};
use crate::models::*;
use akton::prelude::*;
use console::{pad_str, Alignment};
use derive_more::*;
use derive_new::new;
use owo_colors::OwoColorize;
use std::fmt;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

/// Represents a successful commit message with its details.
#[derive(new, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AppEvent {
    event_id: String,
    display_string: String,
}

impl AppEvent {
    pub(crate) fn get_id(&self) -> &String {
        &self.event_id
    }
}

impl Display for AppEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_string)
    }
}

impl From<FinalizedCommit> for AppEvent {
    fn from(value: FinalizedCommit) -> Self {
        let namespace = Uuid::NAMESPACE_OID;

        let simple_urn = format!("{}://{:?}", &value.repository_nickname, value.target_file);
        let event_id = Uuid::new_v3(&namespace, (&simple_urn).as_ref()).to_string();

        let timestamp = &value.when.style(*TIME_COLOR);
        let oid = Oid::new(&value.hash);
        let oid: OidTerminal = (&oid).into();
        let description: DescriptionTerminal = (&value.commit_message.description).into();
        let semver_impact: SemVerImpactTerminal = (&value.commit_message.semver_impact).into();
        let semver_impact = semver_impact.to_string();
        let semver_impact = pad_str(
            &semver_impact,
            *COLUMN_HEADING_FOUR_LENGTH,
            Alignment::Center,
            None,
        );

        let commit_heading: CommitHeadingTerminal = (
            (&value.commit_message.commit_type).into(),
            (&value.commit_message.scope).into(),
            (&value.commit_message.is_breaking).into(),
        )
            .into();
        let repository = &value.repository_nickname.style(*REPO_COLOR);
        let filename = &value.target_file.display();

        let halftab = &HALFTAB.clone();
        let display_string = format!(
            "{halftab}\
                            {repository:<COLUMN_HEADING_ONE_LENGTH$} \
                            {timestamp:^COLUMN_HEADING_TWO_LENGTH$} \
                            {oid:^COLUMN_HEADING_THREE_LENGTH$} \
                            {semver_impact} \
                            {filename:<COLUMN_HEADING_FIVE_LENGTH$} \
                            {commit_heading:<COLUMN_HEADING_SIX_LENGTH$} \
                            {description:<COLUMN_HEADING_SEVEN_LENGTH$}"
        );
        AppEvent::new(event_id, display_string)
    }
}

impl From<GenerationStarted> for AppEvent {
    fn from(value: GenerationStarted) -> Self {
        let namespace = Uuid::NAMESPACE_OID;

        let simple_urn = format!("{}://{:?}", &value.repository_nickname, value.target_file);
        let event_id = Uuid::new_v3(&namespace, (&simple_urn).as_ref()).to_string();
        let time_stamp = "\u{2014}\u{2014}".style(*ALERT_COLOR);
        let binding = &value.target_file.display();
        let filename = &binding.style(*ALERT_COLOR);
        let repository = &value.repository_nickname.style(*ALERT_COLOR);
        let status = "WRITING".style(*ALERT_COLOR).to_string();
        let emdash = EMDASH.style(*ALERT_COLOR);
        let halftab = &HALFTAB.clone();
        let display_string = format!(
            "\
                            {halftab}\
                            {repository:<COLUMN_HEADING_ONE_LENGTH$} \
                            {time_stamp:^COLUMN_HEADING_TWO_LENGTH$} \
                            {status:^COLUMN_HEADING_THREE_LENGTH$} \
                            {emdash:^COLUMN_HEADING_FOUR_LENGTH$} \
                            {filename:<COLUMN_HEADING_FIVE_LENGTH$}"
        );
        AppEvent::new(event_id, display_string)
    }
}

impl From<DiffQueued> for AppEvent {
    fn from(value: DiffQueued) -> Self {
        let namespace = Uuid::NAMESPACE_OID;

        let simple_urn = format!("{}://{:?}", &value.repository_nickname, value.target_file);
        let event_id = Uuid::new_v3(&namespace, (&simple_urn).as_ref()).to_string();
        let time_stamp = "\u{2014}\u{2014}".style(*STATUS_PENDING);
        let binding = &value.target_file.display();
        let filename = &binding.style(*FILENAME_PENDING);
        let repository = &value.repository_nickname.style(*REPO_PENDING_COLOR);
        let status = "PENDING".style(*STATUS_PENDING).to_string();
        let emdash = EMDASH.style(*STATUS_PENDING);
        let halftab = &HALFTAB.clone();
        let display_string = format!(
            "\
                            {halftab}\
                            {repository:<COLUMN_HEADING_ONE_LENGTH$} \
                            {time_stamp:^COLUMN_HEADING_TWO_LENGTH$} \
                            {status:^COLUMN_HEADING_THREE_LENGTH$} \
                            {emdash:^COLUMN_HEADING_FOUR_LENGTH$} \
                            {filename:<COLUMN_HEADING_FIVE_LENGTH$}"
        );
        AppEvent::new(event_id, display_string)
    }
}
