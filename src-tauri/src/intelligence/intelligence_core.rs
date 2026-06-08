use crate::database::Database;
use crate::conversation_context::ConversationContext;
use crate::planner::PlannerEngine as BasePlanner;
use crate::intelligence::{
    IntentEngine,
    knowledge_graph::KnowledgeGraph,
    context_engine::ContextEngine,
    planner_engine::PlannerEngine,
};
use crate::action_bus::ActionRequest;
use std::sync::{Arc, Mutex};

pub struct IntelligenceCore {
    pub intent_engine: Arc<IntentEngine>,
    pub knowledge_graph: Arc<KnowledgeGraph>,
    pub planner_engine: Arc<PlannerEngine>,
    pub context_engine: Arc<ContextEngine>,
}

impl IntelligenceCore {
    pub fn new(
        db: Arc<Mutex<Database>>,
        context: Arc<Mutex<ConversationContext>>,
        planner: Arc<BasePlanner>,
    ) -> Self {
        Self {
            intent_engine: Arc::new(IntentEngine::new()),
            knowledge_graph: Arc::new(KnowledgeGraph::new(db)),
            planner_engine: Arc::new(PlannerEngine::new(planner)),
            context_engine: Arc::new(ContextEngine::new(context)),
        }
    }

    /// Resolves dynamic pronouns and parses intents locally.
    /// Returns Some(ActionRequest) if an offline action match is successful.
    pub fn resolve_local_action(&self, text: &str, plugin_commands: &[crate::intelligence::command_registry::CommandDefinition]) -> Option<ActionRequest> {
        // 1. Resolve pronouns in input first using ContextEngine
        let resolved = self.context_engine.resolve_pronouns(text);
        
        // 2. Parse intent locally using IntentEngine
        if let Some(mut request) = self.intent_engine.parse_intent(&resolved, plugin_commands) {
            // 3. Resolve arguments locally (e.g. 'it', 'that') using ContextEngine
            self.context_engine.resolve_arguments(&mut request.args);
            Some(request)
        } else {
            None
        }
    }
}
