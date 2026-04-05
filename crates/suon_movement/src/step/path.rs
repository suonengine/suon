//! Queued step paths for movement planning.

use crate::prelude::*;
use bevy::prelude::*;
use std::collections::*;

#[derive(Component, Default, Debug)]
/// FIFO queue of pending step directions for an entity.
pub struct StepPath(VecDeque<StepDirection>);

impl StepPath {
    pub fn push(&mut self, direction: StepDirection) {
        self.0.push_back(direction);
    }

    pub fn pop(&mut self) -> Option<StepDirection> {
        self.0.pop_front()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_pop_integrity() {
        // Local constants to improve readability and maintainability
        const FIRST_DIRECTION: StepDirection = StepDirection::North;
        const SECOND_DIRECTION: StepDirection = StepDirection::South;

        let mut path = StepPath::default();

        // Ensure the initial state of the path is completely empty
        assert!(
            path.is_empty(),
            "The newly created StepPath must be empty by default"
        );

        // Push directions into the path and verify the first-in-first-out order
        path.push(FIRST_DIRECTION);
        path.push(SECOND_DIRECTION);

        assert_eq!(
            path.len(),
            2,
            "The path should contain exactly two steps after two successful pushes"
        );

        // The first element retrieved must be the first element that was pushed
        assert_eq!(
            path.pop(),
            Some(FIRST_DIRECTION),
            "The pop_next method should return the first direction pushed following FIFO logic"
        );

        // The second element retrieved must be the next one in the queue
        assert_eq!(
            path.pop(),
            Some(SECOND_DIRECTION),
            "The pop_next method should return the subsequent direction in the sequence"
        );

        // Verify that the path returns None when no more directions remain
        assert!(
            path.is_empty(),
            "The path should be empty after all elements are popped"
        );

        assert_eq!(
            path.pop(),
            None,
            "Popping from an empty path must return None to avoid runtime panics"
        );
    }

    #[test]
    fn test_clear_functionality() {
        const DIRECTION_A: StepDirection = StepDirection::North;
        const DIRECTION_B: StepDirection = StepDirection::East;

        let mut path = StepPath::default();

        // Populate the path with multiple entries
        path.push(DIRECTION_A);
        path.push(DIRECTION_B);

        assert!(
            !path.is_empty(),
            "The path should not be empty after adding directions"
        );

        // Execute the clear command to remove all stored directions
        path.clear();

        // Verify that all metadata and storage have been reset
        assert_eq!(
            path.len(),
            0,
            "The length of the path should be exactly zero after calling clear"
        );

        assert!(
            path.is_empty(),
            "The is_empty check should return true immediately after clearing the path"
        );

        assert_eq!(
            path.pop(),
            None,
            "Attempting to pop from a cleared path should return None"
        );
    }

    #[test]
    fn test_large_scale_path_traversal() {
        let mut path = StepPath::default();
        const TOTAL_ITERATIONS: usize = 100;

        // Fill the path with a large sequence of alternating steps to test integrity under load
        for index in 0..TOTAL_ITERATIONS {
            let direction = if index % 2 == 0 {
                StepDirection::North
            } else {
                StepDirection::South
            };

            path.push(direction);
        }

        assert_eq!(
            path.len(),
            TOTAL_ITERATIONS,
            "The path component should handle a large number of steps without data loss"
        );

        // Drain the entire path and verify that every single step matches the expected sequence
        for index in 0..TOTAL_ITERATIONS {
            let expected_direction = if index % 2 == 0 {
                StepDirection::North
            } else {
                StepDirection::South
            };

            assert_eq!(
                path.pop(),
                Some(expected_direction),
                "The direction sequence mismatch detected at index position {}",
                index
            );
        }

        assert!(
            path.is_empty(),
            "The path must be completely empty after draining all previously inserted steps"
        );
    }

    #[test]
    fn test_empty_state_logic_transitions() {
        const TEST_DIRECTION: StepDirection = StepDirection::East;
        let mut path = StepPath::default();

        // Check initial state
        assert!(path.is_empty(), "StepPath should start in an empty state");

        // Check transition to non-empty
        path.push(TEST_DIRECTION);

        assert!(
            !path.is_empty(),
            "The is_empty method should return false as soon as a direction is added"
        );

        // Check transition back to empty
        path.pop();

        assert!(
            path.is_empty(),
            "The path should return to an empty state once the final element has been popped"
        );
    }
}
