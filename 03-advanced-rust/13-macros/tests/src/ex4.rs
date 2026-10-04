use crate::sut;
use crate::sut::ex04_dsl::{DoorEvent, DoorState, Light, LightEvent, run_door};

#[test]
fn door_transitions() {
    use DoorEvent::*;
    assert_eq!(DoorState::Open.next(CloseDoor), Some(DoorState::Closed));
    assert_eq!(DoorState::Closed.next(Lock), Some(DoorState::Locked));
    assert_eq!(DoorState::Locked.next(Unlock), Some(DoorState::Closed));
    assert_eq!(DoorState::Open.next(Lock), None);
    assert_eq!(DoorState::Locked.next(OpenDoor), None);
}

#[test]
fn door_metadata() {
    assert_eq!(
        DoorState::ALL,
        &[DoorState::Open, DoorState::Closed, DoorState::Locked]
    );
    assert_eq!(
        DoorState::Closed.accepted_events(),
        vec![DoorEvent::OpenDoor, DoorEvent::Lock]
    );
    assert_eq!(
        DoorState::Open.accepted_events(),
        vec![DoorEvent::CloseDoor]
    );
}

#[test]
fn run_door_stops_at_the_first_rejected_event() {
    use DoorEvent::*;
    assert_eq!(run_door(&[CloseDoor, Lock]), Ok(DoorState::Locked));
    assert_eq!(
        run_door(&[CloseDoor, Lock, OpenDoor]),
        Err((DoorState::Locked, OpenDoor))
    );
}

#[test]
fn traffic_light() {
    assert_eq!(Light::Red.next(LightEvent::Timer), Some(Light::Green));
    assert_eq!(Light::Yellow.next(LightEvent::Emergency), Some(Light::Red));
    assert_eq!(Light::Red.next(LightEvent::Emergency), None);
}

#[test]
fn the_dsl_works_in_another_crate() {
    sut::state_machine! {
        machine Switch on Flip {
            states: [On, Off],
            events: [Toggle],
            transitions: { On + Toggle => Off, Off + Toggle => On }
        }
    }
    assert_eq!(Switch::On.next(Flip::Toggle), Some(Switch::Off));
    assert_eq!(Switch::Off.accepted_events(), vec![Flip::Toggle]);
}
