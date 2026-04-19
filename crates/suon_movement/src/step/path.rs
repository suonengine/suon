//! Queued step paths for movement planning.

use bevy::prelude::*;
use std::collections::*;
use suon_position::prelude::*;

#[derive(Component, Default, Debug)]
/// FIFO queue of pending step directions for an entity.
pub struct StepPath(VecDeque<Direction>);

impl StepPath {
    /// Appends a step direction to the back of the queue.
    ///
    /// # Examples
    /// ```
    /// use suon_movement::prelude::*;
    /// use suon_position::prelude::*;
    ///
    /// let mut path = StepPath::default();
    /// path.push(Direction::North);
    ///
    /// assert_eq!(path.len(), 1);
    /// ```
    pub fn push(&mut self, direction: Direction) {
        self.0.push_back(direction);
    }

    /// Removes and returns the next queued direction from the front of the queue.
    ///
    /// # Examples
    /// ```
    /// use suon_movement::prelude::*;
    /// use suon_position::prelude::*;
    ///
    /// let mut path = StepPath::default();
    /// path.push(Direction::East);
    ///
    /// assert_eq!(path.pop(), Some(Direction::East));
    /// ```
    pub fn pop(&mut self) -> Option<Direction> {
        self.0.pop_front()
    }

    /// Clears every queued step direction.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Returns the number of queued directions.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the path currently holds no queued directions.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_preserve_fifo_order_when_pushing_and_popping_steps() {
        const FIRST_DIRECTION: Direction = Direction::North;
        const SECOND_DIRECTION: Direction = Direction::South;

        let mut path = StepPath::default();

        assert!(
            path.is_empty(),
            "The newly created StepPath must be empty by default"
        );

        path.push(FIRST_DIRECTION);
        path.push(SECOND_DIRECTION);

        assert_eq!(
            path.len(),
            2,
            "The path should contain exactly two steps after two successful pushes"
        );

        assert_eq!(
            path.pop(),
            Some(FIRST_DIRECTION),
            "The pop_next method should return the first direction pushed following FIFO logic"
        );

        assert_eq!(
            path.pop(),
            Some(SECOND_DIRECTION),
            "The pop_next method should return the subsequent direction in the sequence"
        );

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
    fn should_clear_all_queued_steps() {
        const DIRECTION_A: Direction = Direction::North;
        const DIRECTION_B: Direction = Direction::East;

        let mut path = StepPath::default();

        path.push(DIRECTION_A);
        path.push(DIRECTION_B);

        assert!(
            !path.is_empty(),
            "The path should not be empty after adding directions"
        );

        path.clear();

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
    fn should_preserve_step_order_across_large_queues() {
        let mut path = StepPath::default();
        const TOTAL_ITERATIONS: usize = 100;

        for index in 0..TOTAL_ITERATIONS {
            let direction = if index % 2 == 0 {
                Direction::North
            } else {
                Direction::South
            };

            path.push(direction);
        }

        assert_eq!(
            path.len(),
            TOTAL_ITERATIONS,
            "The path component should handle a large number of steps without data loss"
        );

        for index in 0..TOTAL_ITERATIONS {
            let expected_direction = if index % 2 == 0 {
                Direction::North
            } else {
                Direction::South
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
    fn should_toggle_empty_state_as_steps_are_added_and_removed() {
        const TEST_DIRECTION: Direction = Direction::East;
        let mut path = StepPath::default();

        assert!(path.is_empty(), "StepPath should start in an empty state");

        path.push(TEST_DIRECTION);

        assert!(
            !path.is_empty(),
            "The is_empty method should return false as soon as a direction is added"
        );

        path.pop();

        assert!(
            path.is_empty(),
            "The path should return to an empty state once the final element has been popped"
        );
    }

    #[test]
    fn should_report_length_changes_after_each_mutation() {
        let mut path = StepPath::default();

        assert_eq!(
            path.len(),
            0,
            "A default path should start with zero directions"
        );

        path.push(Direction::North);
        assert_eq!(
            path.len(),
            1,
            "Pushing one direction should increase the length to one"
        );

        path.push(Direction::East);
        assert_eq!(
            path.len(),
            2,
            "Pushing a second direction should increase the length to two"
        );

        path.pop();
        assert_eq!(
            path.len(),
            1,
            "Popping one direction should decrease the length by one"
        );

        path.clear();
        assert_eq!(
            path.len(),
            0,
            "Clearing the path should remove every queued direction"
        );
    }
}
