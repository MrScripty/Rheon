//! External consumer qualification, compiled against an exact API branch's rlib.
//! RHEON_SESSION_FIXTURE names that checkout's documented session fixture.
mod support {
    include!(env!("RHEON_SESSION_FIXTURE"));
}
use rheon::{LiquidStepStage, SessionError, SimulationSession};
use std::io::{self, Write};
use std::time::Instant;

fn capture(session: &SimulationSession) -> Vec<u8> {
    let mut bytes = Vec::new();
    session.snapshot().write_dense3d(&mut bytes).unwrap();
    bytes
}

struct BrokenConsumer {
    remaining: usize,
}
impl Write for BrokenConsumer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::Error::other("consumer transport failed"));
        }
        let accepted = bytes.len().min(self.remaining);
        self.remaining -= accepted;
        Ok(accepted)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn complete_consumer_failure_chain_recovers_the_uninterrupted_trajectory() {
    let (mut reference, boundary) = support::fixture([4, 2, 2], 0);
    let (mut subject, _) = support::fixture([4, 2, 2], 0);
    for _ in 0..2 {
        reference
            .step(support::inputs(), boundary, None, |_| false)
            .unwrap();
        subject
            .step(support::inputs(), boundary, None, |_| false)
            .unwrap();
    }
    let accepted = capture(&subject);
    assert_eq!(accepted, capture(&reference));
    let mut reached_commit = false;
    assert!(
        subject
            .step(support::inputs(), boundary, None, |stage| {
                if stage == LiquidStepStage::BeforeCommit {
                    reached_commit = true;
                    true
                } else {
                    false
                }
            })
            .is_err()
    );
    assert!(reached_commit, "probe must reach late publication boundary");
    assert_eq!(capture(&subject), accepted);
    assert!(matches!(
        subject.step(support::inputs(), boundary, Some(Instant::now()), |_| false),
        Err(SessionError::Deadline)
    ));
    assert_eq!(capture(&subject), accepted);
    let mut invalid = support::inputs();
    invalid.requested_dt = f64::NAN;
    assert!(subject.step(invalid, boundary, None, |_| false).is_err());
    assert_eq!(capture(&subject), accepted);
    let mut broken = BrokenConsumer { remaining: 80 };
    assert!(subject.snapshot().write_dense3d(&mut broken).is_err());
    assert_eq!(
        broken.remaining, 0,
        "consumer must fail after partial output"
    );
    assert_eq!(capture(&subject), accepted);
    // Check the entire remaining trajectory: rejected work consumes no quota and
    // must not perturb later state, pressure, reports or published stamps.
    for frame in 3..=8 {
        reference
            .step(support::inputs(), boundary, None, |_| false)
            .unwrap();
        subject
            .step(support::inputs(), boundary, None, |_| false)
            .unwrap();
        assert_eq!(capture(&subject), capture(&reference), "frame {frame}");
    }
    let final_frame = capture(&subject);
    assert!(matches!(
        subject.step(support::inputs(), boundary, None, |_| false),
        Err(SessionError::StepLimit)
    ));
    assert_eq!(capture(&subject), final_frame);
    subject.stop();
    assert!(matches!(
        subject.step(support::inputs(), boundary, None, |_| false),
        Err(SessionError::Stopped)
    ));
    assert_eq!(capture(&subject), final_frame);
}
