use crate::{
    JsObject, JsSymbol,
    object::{WeakJsObject, shape::slot::SlotAttributes},
    property::PropertyKey,
};

use super::{SharedShape, TransitionKey};

#[test]
fn test_prune_property_on_counter_limit() {
    let shape = SharedShape::root();

    for i in 0..255 {
        assert_eq!(
            shape.forward_transitions().property_transitions_count(),
            (i, i as u8)
        );

        shape.insert_property_transition(TransitionKey {
            property_key: PropertyKey::Symbol(JsSymbol::new(None).unwrap()),
            attributes: SlotAttributes::all(),
        });
    }

    assert_eq!(
        shape.forward_transitions().property_transitions_count(),
        (255, 255)
    );

    boa_gc::force_collect();

    {
        shape.insert_property_transition(TransitionKey {
            property_key: PropertyKey::Symbol(JsSymbol::new(None).unwrap()),
            attributes: SlotAttributes::all(),
        });
    }

    assert_eq!(
        shape.forward_transitions().property_transitions_count(),
        (1, 0)
    );

    {
        shape.insert_property_transition(TransitionKey {
            property_key: PropertyKey::Symbol(JsSymbol::new(None).unwrap()),
            attributes: SlotAttributes::all(),
        });
    }

    assert_eq!(
        shape.forward_transitions().property_transitions_count(),
        (2, 1)
    );

    boa_gc::force_collect();

    assert_eq!(
        shape.forward_transitions().property_transitions_count(),
        (2, 1)
    );
}

#[test]
fn test_prune_prototype_on_counter_limit() {
    let shape = SharedShape::root();

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (0, 0)
    );

    // Keep the prototypes alive, otherwise their addresses can be reused by the
    // next allocation and the entries would be replaced instead of added.
    let mut kept_alive = Vec::with_capacity(256);

    for i in 0..255 {
        assert_eq!(
            shape.forward_transitions().prototype_transitions_count(),
            (i, i as u8)
        );

        let prototype = JsObject::with_null_proto();
        kept_alive.push(prototype.clone());
        shape.change_prototype_transition(Some(prototype));
    }

    boa_gc::force_collect();

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (255, 255)
    );

    // The prototypes are unreachable now, but they are pruned on the next
    // insertion, when the counter limit is reached.
    kept_alive.clear();
    boa_gc::force_collect();

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (255, 255)
    );

    let prototype = JsObject::with_null_proto();
    kept_alive.push(prototype.clone());
    shape.change_prototype_transition(Some(prototype));

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (1, 0)
    );

    let prototype = JsObject::with_null_proto();
    kept_alive.push(prototype.clone());
    shape.change_prototype_transition(Some(prototype));

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (2, 1)
    );

    boa_gc::force_collect();

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (2, 1)
    );
}

#[test]
fn test_prototype_transition_is_reused() {
    let shape = SharedShape::root();
    let prototype = JsObject::with_null_proto();

    let first = shape.change_prototype_transition(Some(prototype.clone()));
    let second = shape.change_prototype_transition(Some(prototype));

    assert_eq!(first.to_addr_usize(), second.to_addr_usize());
}

#[test]
fn test_prototype_transition_does_not_keep_prototype_alive() {
    let shape = SharedShape::root();

    let weak_prototype = {
        let prototype = JsObject::with_null_proto();
        let weak_prototype = WeakJsObject::new(&prototype);
        shape.change_prototype_transition(Some(prototype));
        weak_prototype
    };

    boa_gc::force_collect();

    assert!(
        weak_prototype.upgrade().is_none(),
        "the prototype transition should not keep the prototype alive"
    );
}
