use crate::{JsObject, JsSymbol, object::shape::slot::SlotAttributes, property::PropertyKey};

use super::{SharedShape, TransitionKey};

#[test]
fn test_prune_property_on_counter_limit() {
    let shape = SharedShape::root(&boa_gc::MutationContext::global());

    for i in 0..255 {
        assert_eq!(
            shape.forward_transitions().property_transitions_count(),
            (i, i as u8)
        );

        shape.insert_property_transition(
            &boa_gc::MutationContext::global(),
            TransitionKey {
                property_key: PropertyKey::Symbol(JsSymbol::new(None).unwrap()),
                attributes: SlotAttributes::all(),
            },
        );
    }

    assert_eq!(
        shape.forward_transitions().property_transitions_count(),
        (255, 255)
    );

    boa_gc::force_collect();

    {
        shape.insert_property_transition(
            &boa_gc::MutationContext::global(),
            TransitionKey {
                property_key: PropertyKey::Symbol(JsSymbol::new(None).unwrap()),
                attributes: SlotAttributes::all(),
            },
        );
    }

    assert_eq!(
        shape.forward_transitions().property_transitions_count(),
        (1, 0)
    );

    {
        shape.insert_property_transition(
            &boa_gc::MutationContext::global(),
            TransitionKey {
                property_key: PropertyKey::Symbol(JsSymbol::new(None).unwrap()),
                attributes: SlotAttributes::all(),
            },
        );
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
    let shape = SharedShape::root(&boa_gc::MutationContext::global());

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (0, 0)
    );

    for i in 0..255 {
        assert_eq!(
            shape.forward_transitions().prototype_transitions_count(),
            (i, i as u8)
        );

        shape.change_prototype_transition(
            &boa_gc::MutationContext::global(),
            Some(JsObject::with_null_proto(&unsafe {
                boa_gc::MutationContext::global()
            })),
        );
    }

    boa_gc::force_collect();

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (255, 255)
    );

    {
        shape.change_prototype_transition(
            &boa_gc::MutationContext::global(),
            Some(JsObject::with_null_proto(&unsafe {
                boa_gc::MutationContext::global()
            })),
        );
    }

    assert_eq!(
        shape.forward_transitions().prototype_transitions_count(),
        (1, 0)
    );

    {
        shape.change_prototype_transition(
            &boa_gc::MutationContext::global(),
            Some(JsObject::with_null_proto(&unsafe {
                boa_gc::MutationContext::global()
            })),
        );
    }

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
