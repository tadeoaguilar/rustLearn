//! Exercise 4: A Domain-Specific Language -- state machines.

/// ```
/// m13_macros_solution::state_machine! {
///     pub machine Switch on Flip {
///         states: [On, Off],
///         events: [Toggle],
///         transitions: { On + Toggle => Off, Off + Toggle => On }
///     }
/// }
/// assert_eq!(Switch::On.next(Flip::Toggle), Some(Switch::Off));
/// ```
#[macro_export]
macro_rules! state_machine {
    (
        $vis:vis machine $state:ident on $event:ident {
            states: [$($s:ident),+ $(,)?],
            events: [$($e:ident),+ $(,)?],
            transitions: { $($from:ident + $ev:ident => $to:ident),* $(,)? } $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $state { $($s),+ }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $event { $($e),+ }

        impl $state {
            pub const ALL: &'static [$state] = &[$($state::$s),+];

            /// The state after `event`, or None if it isn't allowed here.
            pub fn next(self, event: $event) -> ::std::option::Option<$state> {
                // A table of transitions becomes one match arm each.
                #[allow(unreachable_patterns)] // when every pair is covered
                match (self, event) {
                    $(($state::$from, $event::$ev) => ::std::option::Option::Some($state::$to),)*
                    _ => ::std::option::Option::None,
                }
            }

            /// Events with a transition from this state, in declaration order.
            pub fn accepted_events(self) -> ::std::vec::Vec<$event> {
                [$($event::$e),+].into_iter().filter(|e| self.next(*e).is_some()).collect()
            }
        }
    };
}

crate::state_machine! {
    pub machine DoorState on DoorEvent {
        states: [Open, Closed, Locked],
        events: [OpenDoor, CloseDoor, Lock, Unlock],
        transitions: {
            Open   + CloseDoor => Closed,
            Closed + OpenDoor  => Open,
            Closed + Lock      => Locked,
            Locked + Unlock    => Closed,
        }
    }
}

crate::state_machine! {
    pub machine Light on LightEvent {
        states: [Red, Green, Yellow],
        events: [Timer, Emergency],
        transitions: {
            Red    + Timer     => Green,
            Green  + Timer     => Yellow,
            Yellow + Timer     => Red,
            Green  + Emergency => Red,
            Yellow + Emergency => Red,
        }
    }
}

/// Applies events in order; stops at the first one that isn't allowed and
/// returns the state reached so far with the rejected event.
pub fn run_door(events: &[DoorEvent]) -> Result<DoorState, (DoorState, DoorEvent)> {
    let mut state = DoorState::Open;
    for &e in events {
        state = state.next(e).ok_or((state, e))?;
    }
    Ok(state)
}

pub fn run() {
    use DoorEvent::*;
    for s in DoorState::ALL {
        println!("{s:?} accepts {:?}", s.accepted_events());
    }
    println!("close, lock        -> {:?}", run_door(&[CloseDoor, Lock]));
    println!(
        "close, lock, open  -> {:?}",
        run_door(&[CloseDoor, Lock, OpenDoor])
    );
    let mut light = Light::Red;
    let cycle: Vec<Light> = (0..4)
        .map(|_| {
            light = light.next(LightEvent::Timer).unwrap();
            light
        })
        .collect();
    println!("traffic light cycle: {cycle:?}");
}
