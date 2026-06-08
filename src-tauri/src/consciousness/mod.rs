use serde::{Serialize, Deserialize};
use crate::AppState;

pub mod self_state;
pub mod world_state;
pub mod user_state;
pub mod timeline;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PresenceState {
    Sleeping,
    Listening,
    Thinking,
    Speaking,
    Working,
    Monitoring,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessState {
    pub self_state: self_state::SelfState,
    pub world_state: world_state::WorldState,
    pub user_state: user_state::UserState,
    pub timeline: Vec<timeline::TimelineEntry>,
}

pub fn get_current_state(state: &AppState) -> serde_json::Value {
    let self_st = self_state::get_self_state(state);
    let world_st = world_state::get_world_state(state);
    let user_st = user_state::get_user_state(state);
    let tl = timeline::get_recent_timeline(state);

    serde_json::json!(ConsciousnessState {
        self_state: self_st,
        world_state: world_st,
        user_state: user_st,
        timeline: tl,
    })
}
