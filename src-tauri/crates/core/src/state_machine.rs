use thiserror::Error;
use crate::core::types::DownloadStatus;

#[derive(Error, Debug, PartialEq)]
pub enum StateMachineError {
    #[error("Illegal transition from {from:?} to {to:?}")]
    IllegalTransition {
        from: DownloadStatus,
        to: DownloadStatus,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DownloadStateMachine {
    current: DownloadStatus,
}

impl DownloadStateMachine {
    pub fn new() -> Self {
        Self {
            current: DownloadStatus::Idle,
        }
    }

    pub fn with_state(initial: DownloadStatus) -> Self {
        Self { current: initial }
    }

    pub fn current_state(&self) -> DownloadStatus {
        self.current
    }

    pub fn transition(&mut self, next: DownloadStatus) -> Result<(), StateMachineError> {
        if self.is_valid_transition(self.current, next) {
            self.current = next;
            Ok(())
        } else {
            Err(StateMachineError::IllegalTransition {
                from: self.current,
                to: next,
            })
        }
    }

    fn is_valid_transition(&self, from: DownloadStatus, to: DownloadStatus) -> bool {
        if from == to {
            return true; // No-op transition
        }

        match from {
            DownloadStatus::Idle => matches!(
                to,
                DownloadStatus::Analyzing | DownloadStatus::Ready | DownloadStatus::Downloading
            ),
            DownloadStatus::Analyzing => matches!(
                to,
                DownloadStatus::Ready | DownloadStatus::Failed | DownloadStatus::Idle
            ),
            DownloadStatus::Ready => matches!(
                to,
                DownloadStatus::Downloading | DownloadStatus::Analyzing | DownloadStatus::Idle
            ),
            DownloadStatus::Downloading => matches!(
                to,
                DownloadStatus::PostProcessing
                    | DownloadStatus::Verifying
                    | DownloadStatus::Failed
                    | DownloadStatus::Cancelling
            ),
            DownloadStatus::PostProcessing => matches!(
                to,
                DownloadStatus::Verifying
                    | DownloadStatus::Failed
                    | DownloadStatus::Cancelling
            ),
            DownloadStatus::Verifying => {
                matches!(to, DownloadStatus::Completed | DownloadStatus::Failed)
            }
            DownloadStatus::Cancelling => {
                matches!(to, DownloadStatus::Cancelled | DownloadStatus::Failed)
            }
            DownloadStatus::Completed | DownloadStatus::Failed | DownloadStatus::Cancelled => {
                // Terminal states can transition to Idle or Analyzing on restart
                matches!(
                    to,
                    DownloadStatus::Idle | DownloadStatus::Analyzing | DownloadStatus::Ready
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        let mut sm = DownloadStateMachine::new();
        assert_eq!(sm.current_state(), DownloadStatus::Idle);

        assert!(sm.transition(DownloadStatus::Analyzing).is_ok());
        assert!(sm.transition(DownloadStatus::Ready).is_ok());
        assert!(sm.transition(DownloadStatus::Downloading).is_ok());
        assert!(sm.transition(DownloadStatus::PostProcessing).is_ok());
        assert!(sm.transition(DownloadStatus::Verifying).is_ok());
        assert!(sm.transition(DownloadStatus::Completed).is_ok());
    }

    #[test]
    fn test_cancellation_flow() {
        let mut sm = DownloadStateMachine::with_state(DownloadStatus::Downloading);
        assert!(sm.transition(DownloadStatus::Cancelling).is_ok());
        assert!(sm.transition(DownloadStatus::Cancelled).is_ok());
    }

    #[test]
    fn test_illegal_transition() {
        let mut sm = DownloadStateMachine::new();
        assert_eq!(
            sm.transition(DownloadStatus::Completed),
            Err(StateMachineError::IllegalTransition {
                from: DownloadStatus::Idle,
                to: DownloadStatus::Completed
            })
        );
    }
}
