use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub text: String,
    pub completed: bool,
    pub created_at: NaiveDate,
}

impl Task {
    pub fn new(id: i64, text: String, completed: bool, created_at: NaiveDate) -> Self {
        Self {
            id,
            text,
            completed,
            created_at,
        }
    }
    
    pub fn is_completed(&self) -> bool {
        self.completed
    }
    
    pub fn mark_completed(&mut self) {
        self.completed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_task_new() {
        let today = Utc::now().date_naive();
        let task = Task::new(1, "Test task".to_string(), false, today);
        
        assert_eq!(task.id, 1);
        assert_eq!(task.text, "Test task");
        assert!(!task.completed);
        assert_eq!(task.created_at, today);
    }

    #[test]
    fn test_task_new_completed() {
        let today = Utc::now().date_naive();
        let task = Task::new(42, "Completed task".to_string(), true, today);
        
        assert_eq!(task.id, 42);
        assert_eq!(task.text, "Completed task");
        assert!(task.completed);
        assert_eq!(task.created_at, today);
    }

    #[test]
    fn test_is_completed_false() {
        let today = Utc::now().date_naive();
        let task = Task::new(1, "Incomplete task".to_string(), false, today);
        
        assert!(!task.is_completed());
    }

    #[test]
    fn test_is_completed_true() {
        let today = Utc::now().date_naive();
        let task = Task::new(1, "Complete task".to_string(), true, today);
        
        assert!(task.is_completed());
    }

    #[test]
    fn test_mark_completed() {
        let today = Utc::now().date_naive();
        let mut task = Task::new(1, "Task to complete".to_string(), false, today);
        
        assert!(!task.is_completed());
        
        task.mark_completed();
        
        assert!(task.is_completed());
        assert!(task.completed);
    }

    #[test]
    fn test_mark_completed_idempotent() {
        let today = Utc::now().date_naive();
        let mut task = Task::new(1, "Already complete".to_string(), true, today);
        
        assert!(task.is_completed());
        
        task.mark_completed();
        
        assert!(task.is_completed());
        assert!(task.completed);
    }

    #[test]
    fn test_task_clone() {
        let today = Utc::now().date_naive();
        let task = Task::new(5, "Original task".to_string(), false, today);
        let cloned_task = task.clone();
        
        assert_eq!(task.id, cloned_task.id);
        assert_eq!(task.text, cloned_task.text);
        assert_eq!(task.completed, cloned_task.completed);
        assert_eq!(task.created_at, cloned_task.created_at);
    }

    #[test]
    fn test_task_fields_independent_after_clone() {
        let today = Utc::now().date_naive();
        let original = Task::new(1, "Original".to_string(), false, today);
        let mut cloned = original.clone();
        
        // Modify clone
        cloned.mark_completed();
        
        // Original should be unchanged
        assert!(!original.is_completed());
        assert!(cloned.is_completed());
    }

    #[test] 
    fn test_task_serialization() {
        let today = Utc::now().date_naive();
        let task = Task::new(1, "Serializable task".to_string(), true, today);
        
        // Test JSON serialization
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("Serializable task"));
        assert!(json.contains("true"));
        
        // Test deserialization
        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.text, deserialized.text);
        assert_eq!(task.completed, deserialized.completed);
        assert_eq!(task.created_at, deserialized.created_at);
    }

    #[test]
    fn test_task_with_special_characters() {
        let today = Utc::now().date_naive();
        let special_text = "Task with émojis 🎯 and unicode ñ";
        let task = Task::new(999, special_text.to_string(), false, today);
        
        assert_eq!(task.text, special_text);
        
        // Should serialize/deserialize correctly
        let json = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(task.text, deserialized.text);
    }

    #[test]
    fn test_task_empty_text() {
        let today = Utc::now().date_naive();
        let task = Task::new(0, "".to_string(), false, today);
        
        assert_eq!(task.text, "");
        assert_eq!(task.id, 0);
    }
}