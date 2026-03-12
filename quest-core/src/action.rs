use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionTrigger {
    EveryAction,
    EveryNActions(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Action {
    pub id: String,
    pub name: String,
    pub trigger: ActionTrigger,
}

impl Action {
    pub fn default_attack() -> Self {
        Self {
            id: "attack".to_string(),
            name: "Attack".to_string(),
            trigger: ActionTrigger::EveryAction,
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
}
