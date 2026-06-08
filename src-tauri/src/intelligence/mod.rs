pub mod intent;
pub mod groq;
pub mod entity;
pub mod command_registry;
pub mod memory_engine;
pub mod behavior_model;
pub mod pattern_detector;
pub mod habit_engine;
pub mod proactive_assistant;
pub mod emotional_intelligence;
pub mod knowledge_graph;
pub mod context_engine;
pub mod planner_engine;
pub mod intelligence_core;
pub mod learning_engine;
pub mod project_detector;
pub mod session_manager;
pub mod workspace_engine;
pub mod execution_history;
pub mod solution_database;
pub mod task_memory;
pub mod code_analyzer;
pub mod log_analyzer;
pub mod developer_engine;
pub mod performance_tracker;
pub mod health_monitor;
pub mod multimodal_context;
pub mod self_improvement;

pub mod self_diagnostics;
pub mod recovery_engine;

pub use intent::IntentEngine;
pub use groq::GroqClient;
pub use intelligence_core::IntelligenceCore;
pub use learning_engine::LearningEngine;
pub use workspace_engine::WorkspaceEngine;
pub use developer_engine::DeveloperEngine;
pub use multimodal_context::MultimodalContextEngine;
pub use self_improvement::SelfImprovementEngine;
pub use self_diagnostics::run_diagnostics;
pub use recovery_engine::RecoveryEngine;


