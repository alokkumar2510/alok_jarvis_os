use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    P0 = 0, // Emergency Interrupt (highest)
    P1 = 1, // User Voice Command
    P2 = 2, // Agent Tasks
    P3 = 3, // Background Monitoring
    P4 = 4, // Learning & Analytics (lowest)
}

impl Priority {
    /// Returns true if self is higher priority than other.
    /// In this system, smaller numerical values represent higher priorities.
    pub fn is_higher_than(self, other: Priority) -> bool {
        (self as u8) < (other as u8)
    }

    /// Returns true if self is higher than or equal priority to other.
    pub fn is_higher_or_equal(self, other: Priority) -> bool {
        (self as u8) <= (other as u8)
    }

    /// Determine priority based on action or command type/keyword
    pub fn determine_priority(action_id: &str, category: &str) -> Self {
        let cmd = action_id.to_lowercase();
        
        // P0: Emergency Stop commands
        if cmd == "stop everything" || cmd == "emergency stop" || cmd == "abort" || cmd == "shut it down" {
            return Priority::P0;
        }

        // P1: Direct speech interrupts/commands
        if cmd == "stop" || cmd == "cancel" || cmd == "pause" || cmd == "resume" 
            || cmd == "continue" || cmd == "be quiet" || cmd == "sleep" 
            || cmd == "wake up" || cmd == "forget that" || cmd == "never mind" 
            || cmd == "abort mission" || cmd == "start over" 
            || cmd == "what are you doing?" || cmd == "how long left?"
        {
            return Priority::P1;
        }

        // P2: Agent task swarms, planners, file modifications, browser activities
        if category == "agent" || category == "planner" || category == "browser" || category == "file" {
            return Priority::P2;
        }

        // P3: Background monitoring, active window trackers
        if category == "monitoring" || category == "sensor" || cmd == "health_check" {
            return Priority::P3;
        }

        // P4: Learning consolidation & offline database optimization
        Priority::P4
    }
}
