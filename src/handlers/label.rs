//! Purpose: Allow any user to modify issue labels on GitHub via comments.
//!
//! The current syntax allows adding labels (+labelname or just labelname) following the
//! `label:` prefix. Users can also remove labels with -labelname.
//!
//! No verification is currently attempted of the added labels (only currently present labels
//! can be removed). XXX: How does this affect users workflow?
//!
//! There will be no feedback beyond the label change to reduce notification noise.

use crate::{
    github::{GithubClient, Label},
    registry::{Event, Handler},
};
use failure::Error;
use lazy_static::lazy_static;
use regex::Regex;

pub struct LabelHandler {
    pub client: GithubClient,
}

impl Handler for LabelHandler {
    fn handle_event(&self, event: &Event) -> Result<(), Error> {
        #[allow(irrefutable_let_patterns)]
        let event = if let Event::IssueComment(e) = event {
            e
        } else {
            // not interested in other events
            return Ok(());
        };

        lazy_static! {
            static ref LABEL_RE: Regex = Regex::new(r#"\blabel: (\S+\s*)+"#).unwrap();
        }

        let mut issue_labels = event.issue.labels().to_owned();

        for label_block in LABEL_RE.find_iter(&event.comment.body) {
            let label_block = &label_block.as_str()["label: ".len()..]; // guaranteed to start with this
            for label in label_block.split_whitespace() {
                if label.starts_with('-') {
                    if let Some(label) = issue_labels.iter().position(|el| el.name == &label[1..]) {
                        issue_labels.remove(label);
                    } else {
                        // do nothing, if the user attempts to remove a label that's not currently
                        // set simply skip it
                    }
                } else if label.starts_with('+') {
                    // add this label, but without the +
                    issue_labels.push(Label {
                        name: label[1..].to_string(),
                    });
                } else {
                    // add this label (literally)
                    issue_labels.push(Label {
                        name: label.to_string(),
                    });
                }
            }
        }

        event.issue.set_labels(&self.client, issue_labels)?;

        Ok(())
    }
}