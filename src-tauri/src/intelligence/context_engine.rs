use crate::conversation_context::ConversationContext;
use crate::environment_awareness;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct ContextEngine {
    context: Arc<Mutex<ConversationContext>>,
}

impl ContextEngine {
    pub fn new(context: Arc<Mutex<ConversationContext>>) -> Self {
        Self { context }
    }

    /// Capture current active window, clipboard, file, folder, and browser URL and update context.
    pub fn capture_workspace_context(&self) -> environment_awareness::EnvironmentContext {
        let env = environment_awareness::capture();
        if let Ok(mut ctx) = self.context.lock() {
            ctx.update_environment_context(&env);
        }
        env
    }

    /// Resolves dynamic pronouns and variables in the user's transcript locally.
    pub fn resolve_pronouns(&self, transcript: &str) -> String {
        if let Ok(ctx) = self.context.lock() {
            ctx.resolve_text(transcript)
        } else {
            transcript.to_string()
        }
    }

    /// Resolves 'it', 'that' references in entity arguments locally.
    pub fn resolve_arguments(&self, args: &mut HashMap<String, String>) {
        if let Ok(ctx) = self.context.lock() {
            ctx.resolve_args(args);
        }
    }

    /// Clears the active dialogue and workspace context.
    pub fn clear_context(&self) {
        if let Ok(mut ctx) = self.context.lock() {
            ctx.clear();
        }
    }
}
