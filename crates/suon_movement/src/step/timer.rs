//! Per-entity timers that gate path step advancement.

use bevy::prelude::*;

#[derive(Component, Deref, DerefMut, Default)]
/// Thin ECS wrapper around Bevy's [`Timer`] used by step path progression.
pub(crate) struct StepTimer(pub(crate) Timer);

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn should_expose_inner_timer_state_through_deref() {
        const INITIAL_DURATION_SECONDS: f32 = 1.0;

        let timer_duration = Duration::from_secs_f32(INITIAL_DURATION_SECONDS);
        let step_timer = StepTimer(Timer::new(timer_duration, TimerMode::Once));

        assert_eq!(
            step_timer.duration(),
            timer_duration,
            "The StepTimer should correctly report the duration of the internal timer"
        );

        assert!(
            !step_timer.is_finished(),
            "A newly created timer should not be in a finished state"
        );
    }

    #[test]
    fn should_default_to_a_zero_duration_one_shot_timer() {
        let step_timer = StepTimer::default();

        assert_eq!(
            step_timer.duration(),
            Duration::ZERO,
            "The default StepTimer should initialize with a duration of zero"
        );

        assert_eq!(
            step_timer.mode(),
            TimerMode::Once,
            "The default timer mode should be set to Once"
        );
    }

    #[test]
    fn should_advance_and_finish_when_ticked_through_deref_mut() {
        const TOTAL_DURATION: f32 = 2.0;
        const TICK_AMOUNT: f32 = 1.0;

        let mut step_timer = StepTimer(Timer::from_seconds(TOTAL_DURATION, TimerMode::Once));

        step_timer.tick(Duration::from_secs_f32(TICK_AMOUNT));

        assert_eq!(
            step_timer.elapsed_secs(),
            TICK_AMOUNT,
            "The elapsed time should match the total amount of time ticked"
        );

        assert!(
            !step_timer.is_finished(),
            "The timer should not be finished when the elapsed time is less than the total \
             duration"
        );

        step_timer.tick(Duration::from_secs_f32(TICK_AMOUNT));

        assert!(
            step_timer.is_finished(),
            "The timer must report as finished once the elapsed time reaches the duration"
        );
    }

    #[test]
    fn should_reset_elapsed_time_without_finishing_the_timer() {
        const DURATION_SECONDS: f32 = 5.0;
        let mut step_timer = StepTimer(Timer::from_seconds(DURATION_SECONDS, TimerMode::Once));

        step_timer.tick(Duration::from_secs_f32(1.0));

        assert!(step_timer.elapsed_secs() > 0.0);

        step_timer.reset();

        assert_eq!(
            step_timer.elapsed_secs(),
            0.0,
            "The elapsed time must return to zero after a reset call"
        );

        assert!(
            !step_timer.is_finished(),
            "The timer should no longer be finished after being reset"
        );
    }
}
