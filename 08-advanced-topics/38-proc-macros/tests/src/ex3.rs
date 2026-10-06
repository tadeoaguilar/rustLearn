//! Exercise 3: state_machine! used for real.

use crate::sut::state_machine;

state_machine! {
    machine Order {
        initial Created;
        Created -> Paid on Pay;
        Created -> Cancelled on Cancel;
        Paid -> Shipped on Ship;
        Paid -> Refunded on Refund;
        Shipped -> Delivered on Deliver;
    }
}

#[test]
fn ex3_transitions() {
    let mut order = Order::new();
    assert_eq!(order.state(), OrderState::Created);
    assert!(order.can_fire(OrderEvent::Pay) && !order.can_fire(OrderEvent::Ship));
    assert_eq!(order.fire(OrderEvent::Pay), Ok(OrderState::Paid));
    assert_eq!(order.fire(OrderEvent::Ship), Ok(OrderState::Shipped));
    let err = order.fire(OrderEvent::Refund).unwrap_err();
    assert_eq!(
        err,
        OrderError {
            state: OrderState::Shipped,
            event: OrderEvent::Refund
        }
    );
    assert_eq!(err.to_string(), "no transition from Shipped on Refund");
    assert_eq!(
        order.state(),
        OrderState::Shipped,
        "unchanged after a refused event"
    );
    assert_eq!(order.fire(OrderEvent::Deliver), Ok(OrderState::Delivered));
    assert_eq!(Order::default(), Order::new());
}

#[test]
fn ex3_generated_items() {
    assert_eq!(Order::TRANSITIONS.len(), 5);
    assert_eq!(
        Order::TRANSITIONS[1],
        (
            OrderState::Created,
            OrderEvent::Cancel,
            OrderState::Cancelled
        )
    );
    let error: &dyn std::error::Error = &OrderError {
        state: OrderState::Created,
        event: OrderEvent::Ship,
    };
    assert!(error.to_string().contains("Created"));
    // variants in order of first appearance, the initial state first
    assert_eq!(
        format!(
            "{:?}",
            [OrderState::Created, OrderState::Paid, OrderState::Cancelled]
        ),
        "[Created, Paid, Cancelled]"
    );
}
