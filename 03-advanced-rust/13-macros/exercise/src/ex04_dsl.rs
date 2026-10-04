//! Exercise 4: A Domain-Specific Language.  -- see exercises.md
//!
//! Write `state_machine!` (#[macro_export]) so that this works:
//!
//!     crate::state_machine! {
//!         pub machine DoorState on DoorEvent {
//!             states: [Open, Closed, Locked],
//!             events: [OpenDoor, CloseDoor, Lock, Unlock],
//!             transitions: {
//!                 Open   + CloseDoor => Closed,
//!                 Closed + OpenDoor  => Open,
//!                 Closed + Lock      => Locked,
//!                 Locked + Unlock    => Closed,
//!             }
//!         }
//!     }
//!
//! generating DoorState and DoorEvent (Debug, Clone, Copy, PartialEq, Eq, Hash),
//! DoorState::ALL, DoorState::next(self, DoorEvent) -> Option<DoorState> and
//! DoorState::accepted_events(self) -> Vec<DoorEvent>.
//!
//! Then define the door above and a traffic light:
//!     machine Light on LightEvent, states [Red, Green, Yellow], events [Timer, Emergency]
//!     Red+Timer=>Green, Green+Timer=>Yellow, Yellow+Timer=>Red,
//!     Green+Emergency=>Red, Yellow+Emergency=>Red

// TODO Exercise 4: the macro, the two machines, and run_door below.

// /// Applies events from DoorState::Open; stops at the first rejected one.
// pub fn run_door(events: &[DoorEvent]) -> Result<DoorState, (DoorState, DoorEvent)>

pub fn run() {
    todo!("Exercise 4")
}
