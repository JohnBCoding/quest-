use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionTrigger {
    EveryAction,
    EveryNActions(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ActionCondition {
    #[default]
    None,
    HealthBelowPercent(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Action {
    pub id: String,
    pub name: String,
    pub trigger: ActionTrigger,
    #[serde(default)]
    pub condition: ActionCondition,
}

impl Action {
    pub fn default_attack() -> Self {
        Self {
            id: "attack".to_string(),
            name: "Attack".to_string(),
            trigger: ActionTrigger::EveryAction,
            condition: ActionCondition::None,
        }
    }

    pub fn default_health_potion() -> Self {
        Self {
            id: "health_potion".to_string(),
            name: "Health Potion".to_string(),
            trigger: ActionTrigger::EveryAction,
            condition: ActionCondition::HealthBelowPercent(50),
        }
    }

    pub fn default_assassination() -> Self {
        Self {
            id: "assassination".to_string(),
            name: "Assassinate".to_string(),
            trigger: ActionTrigger::EveryAction,
            condition: ActionCondition::None,
        }
    }

    pub fn trigger_matches(&self, action_number: u32) -> bool {
        match self.trigger {
            ActionTrigger::EveryAction => true,
            ActionTrigger::EveryNActions(n) => n > 0 && action_number % n == 0,
        }
    }

    pub fn health_threshold_percent(&self) -> Option<u32> {
        match self.condition {
            ActionCondition::HealthBelowPercent(percent) => Some(percent),
            ActionCondition::None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_attack_has_correct_values() {
        let action = Action::default_attack();
        assert_eq!(action.id, "attack");
        assert_eq!(action.name, "Attack");
        assert_eq!(action.trigger, ActionTrigger::EveryAction);
        assert_eq!(action.condition, ActionCondition::None);
    }

    #[test]
    fn default_health_potion_has_correct_values() {
        let action = Action::default_health_potion();
        assert_eq!(action.id, "health_potion");
        assert_eq!(action.name, "Health Potion");
        assert_eq!(action.trigger, ActionTrigger::EveryAction);
        assert_eq!(action.condition, ActionCondition::HealthBelowPercent(50));
    }

    #[test]
    fn action_serialization_roundtrip() {
        let action = Action::default_attack();
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, deserialized);
    }

    #[test]
    fn every_n_actions_serialization_roundtrip() {
        let action = Action {
            id: "fireball".to_string(),
            name: "Fireball".to_string(),
            trigger: ActionTrigger::EveryNActions(3),
            condition: ActionCondition::None,
        };
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(action, deserialized);
    }

    #[test]
    fn deserialization_rejects_invalid_trigger() {
        let bad_json = r#"{"id":"x","name":"X","trigger":"invalid"}"#;
        let result = serde_json::from_str::<Action>(bad_json);
        assert!(result.is_err());
    }

    #[test]
    fn trigger_matches_every_n() {
        let action = Action {
            id: "burst".to_string(),
            name: "Burst".to_string(),
            trigger: ActionTrigger::EveryNActions(3),
            condition: ActionCondition::None,
        };
        assert!(!action.trigger_matches(1));
        assert!(!action.trigger_matches(2));
        assert!(action.trigger_matches(3));
    }

    #[test]
    fn missing_condition_defaults_to_none() {
        let json = r#"{"id":"attack","name":"Attack","trigger":"EveryAction"}"#;
        let action: Action = serde_json::from_str(json).unwrap();
        assert_eq!(action.condition, ActionCondition::None);
    }

    #[test]
    fn default_assassination_has_correct_values() {
        let action = Action::default_assassination();
        assert_eq!(action.id, "assassination");
        assert_eq!(action.name, "Assassinate");
        assert_eq!(action.trigger, ActionTrigger::EveryAction);
        assert_eq!(action.condition, ActionCondition::None);
    }
}
