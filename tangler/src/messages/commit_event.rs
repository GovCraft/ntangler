// use crate::messages::commit_authoring::CommitAuthoring;
// use crate::messages::CommitPosted;
// use crate::models::*;
// use akton::prelude::*;
// use owo_colors::OwoColorize;
// use std::fmt;
// use console::{Alignment, pad_str};
// use derive_more::*;
//
// /// Represents a successful commit message with its details.
// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
// pub(crate) enum CommitEventCategory {
//     FilePending(PendingCommit),
//     DiffGenerated(DiffGeneratedCommit),
//     CommitMessageGenerated(CommitMessageGeneratedCommit),
//     FileCommitted(CommittedCommit),
// }
//
// #[akton_message]
// #[derive(PartialEq, Eq, PartialOrd, Ord)]
// pub(crate) struct CommitEvent {
//     pub(crate) id: String,
//     pub(crate) timestamp: TimeStamp,
//     pub(crate) category: CommitEventCategory,
// }
//
// /* TODO: this is convenient but incorrect, types should be responsible for their own formatting and the scribe should be responsible for layout; this is a message */
// impl fmt::Display for CommitEvent {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         // let tab = " ".repeat(TAB_WIDTH);
//         let half_tab = " ".repeat(TAB_WIDTH / 2);
//         let emdash = "  \u{2022}  ".style(*STATUS_PENDING);
//         let time_stamp = "\u{2014}\u{2014}".style(*TIME_COLOR);
//
//         let display = match &self.category {
//             CommitEventCategory::FilePending(event) => {
//                 let time_stamp = "\u{2014}\u{2014}".style(*STATUS_PENDING);
//                 let filename = &event.filename.style(*FILENAME_PENDING);
//                 let repository = &event.repository.style(*REPO_PENDING_COLOR);
//                 let status = &event.status.style(*STATUS_PENDING).to_string();
//                 format!(
//                     "\
//                     {half_tab}\
//                     {repository:<COLUMN_HEADING_ONE_LENGTH$} \
//                     {time_stamp:^COLUMN_HEADING_TWO_LENGTH$} \
//                     {status:^COLUMN_HEADING_THREE_LENGTH$}\
//                     {emdash:^COLUMN_HEADING_FOUR_LENGTH$} \
//                     {filename:<COLUMN_HEADING_FIVE_LENGTH$}"
//                 )
//             }
//             CommitEventCategory::DiffGenerated(event) => {
//                 let time_stamp = "\u{2014}\u{2014}".style(*STATUS_PENDING);
//                 let filename = &event.filename.style(*FILENAME_PENDING);
//                 let repository = &event.repository.style(*REPO_PENDING_COLOR);
//                 let status = &event.status.style(*STATUS_PENDING).to_string();
//                 format!(
//                     "\
//                     {half_tab}\
//                     {repository:<COLUMN_HEADING_ONE_LENGTH$} \
//                     {time_stamp:^COLUMN_HEADING_TWO_LENGTH$} \
//                     {status:^COLUMN_HEADING_THREE_LENGTH$}\
//                     {emdash:^COLUMN_HEADING_FOUR_LENGTH$} \
//                     {filename:<COLUMN_HEADING_FIVE_LENGTH$}"
//                 )
//             }
//             CommitEventCategory::CommitMessageGenerated(event) => {
//                 let emdash = "  \u{2022}  ".style(*ALERT_COLOR);
//
//                 let time_stamp = "\u{2014}\u{2014}".style(*ALERT_COLOR);
//                 let filename = &event.commit.filename.style(*ALERT_COLOR);
//                 let repository = &event.commit.repository.style(*ALERT_COLOR);
//                 let status = &event.status.style(*ALERT_COLOR).to_string();
//                 format!(
//                     "{half_tab}\
//                     {repository:<COLUMN_HEADING_ONE_LENGTH$} \
//                     {time_stamp:^COLUMN_HEADING_TWO_LENGTH$} \
//                     {status:^COLUMN_HEADING_THREE_LENGTH$}\
//                     {emdash:^COLUMN_HEADING_FOUR_LENGTH$} \
//                     {filename:<COLUMN_HEADING_FIVE_LENGTH$}"
//                 )
//             }
//             CommitEventCategory::FileCommitted(event) => {
//                 let timestamp = &self.timestamp.style(*TIME_COLOR);
//                 let oid: OidTerminal = (&event.oid).into();
//                 let description: DescriptionTerminal = (&event.description).into();
//                 let semver_impact: SemVerImpactTerminal = (&event.semver_impact).into();
//                 let semver_impact = semver_impact.to_string();
//                 let semver_impact = pad_str(&semver_impact, *COLUMN_HEADING_FOUR_LENGTH, Alignment::Center, None);
//                 // let semver_impact = format!("{semver_impact:^COLUMN_HEADING_FOUR_LENGTH$}");
//                 let commit_heading: CommitHeadingTerminal = (
//                     (&event.commit_type).into(),
//                     (&event.scope).into(),
//                     (&event.is_breaking).into(),
//                 )
//                     .into();
//                 let repository = &event.repository.style(*REPO_COLOR);
//                 let filename = &event.filename;
//                 format!(
//                     "{half_tab}\
//                     {repository:<COLUMN_HEADING_ONE_LENGTH$} \
//                     {timestamp:^COLUMN_HEADING_TWO_LENGTH$} \
//                     {oid:^COLUMN_HEADING_THREE_LENGTH$} \
//                     {semver_impact} \
//                     {filename:<COLUMN_HEADING_FIVE_LENGTH$} \
//                     {commit_heading:<COLUMN_HEADING_SIX_LENGTH$} \
//                     {description:<COLUMN_HEADING_SEVEN_LENGTH$}"
//                 )
//             }
//         };
//
//         write!(f, "{}", display)
//     }
// }
//
// impl CommitEvent {
//     pub(crate) fn new(category: CommitEventCategory) -> CommitEvent {
//         let (filename, repository) = match &category {
//             CommitEventCategory::FilePending(c) => (&c.filename, &c.repository),
//             CommitEventCategory::CommitMessageGenerated(c) => (&c.commit.filename, &c.commit.repository),
//             CommitEventCategory::FileCommitted(c) => (&c.filename, &c.repository),
//             CommitEventCategory::DiffGenerated(c) => {(&Filename::new(&c.filename), &c.repository)}
//         };
//         let timestamp = TimeStamp::new();
//         let id = generate_id(repository, filename.clone());
//         CommitEvent {
//             id,
//             timestamp,
//             category,
//         }
//     }
// }
