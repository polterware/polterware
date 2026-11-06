//! What the shared Composer is asked to create (`ComposeRequest`, `setComposeRequest` in
//! `App.tsx`): C, Commander's actions, and each page's "+" leave a request here, and the
//! app's Composer takes it.

use gpui::Context;

use super::AppState;
use crate::work::Stage;

/// A card, a decision, or a doc, with what the place it was asked from already knows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ComposeRequest {
    /// `stage` starts the Stage pill (Idea when none); `milestone` is a milestone's id, empty
    /// for the Backlog.
    Card {
        stage: Option<Stage>,
        milestone: String,
    },
    Decision,
    Doc,
}

impl AppState {
    /// Asks the shared Composer to open for `request`, as `setComposeRequest` does.
    pub fn compose(&mut self, request: ComposeRequest, cx: &mut Context<Self>) {
        if self.project_id().is_none() {
            return;
        }
        self.commander_open = false;
        self.compose_request = Some(request);
        cx.notify();
    }

    /// The request the Composer has not opened for yet.
    pub fn take_compose_request(&mut self) -> Option<ComposeRequest> {
        self.compose_request.take()
    }
}
