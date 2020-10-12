use crate::{
  builtins::{function::make_builtin_fn, iterable::create_iter_result_object, Map, Value},
  object::ObjectData,
  property::{Attribute, DataDescriptor},
  BoaProfiler, Context, Result,
};
use gc::{Finalize, Trace};

#[derive(Debug, Clone, Finalize, Trace)]
pub enum MapIterationKind {
  Key,
  Value,
  KeyAndValue,
}

/// The Map Iterator object represents an iteration over a map. It implements the iterator protocol.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: TODO https://tc39.es/ecma262/#sec-array-iterator-objects
#[derive(Debug, Clone, Finalize, Trace)]
pub struct MapIterator {
  iterated_map: Value,
  map_next_index: u32,
  map_iteration_kind: MapIterationKind,
}

impl MapIterator {
  pub(crate) const NAME: &'static str = "MapIterator";

  fn new(map: Value, kind: MapIterationKind) -> Self {
    MapIterator {
      iterated_map: map,
      map_next_index: 0,
      map_iteration_kind: kind,
    }
  }

  /// Abstract operation CreateMapIterator( map, kind )
  ///
  /// Creates a new iterator over the given map.
  ///
  /// More information:
  ///  - [ECMA reference][spec]
  ///
  /// [spec]: https://www.ecma-international.org/ecma-262/11.0/index.html#sec-createmapiterator
  pub(crate) fn create_map_iterator(
    ctx: &Context,
    map: Value,
    kind: MapIterationKind,
  ) -> Result<Value> {
    let map_iterator = Value::new_object(Some(ctx.global_object()));
    map_iterator.set_data(ObjectData::MapIterator(Self::new(map, kind)));
    map_iterator
      .as_object_mut()
      .expect("map iterator object")
      .set_prototype_instance(ctx.iterator_prototypes().map_iterator().into());
    Ok(map_iterator)
  }


  /// %MapIteratorPrototype%.next( )
  ///
  /// Gets the next result in the map.
  ///
  /// More information:
  ///  - [ECMA reference][spec]
  ///
  /// [spec]: https://tc39.es/ecma262/#sec-%mapiteratorprototype%.next
  pub(crate) fn next(this: &Value, _args: &[Value], ctx: &mut Context) -> Result<Value> {
    if let Value::Object(ref object) = this {
      let mut object = object.borrow_mut();
      if let Some(map_iterator) = object.as_map_iterator_mut() {
        let m = &map_iterator.iterated_map;
        let index = map_iterator.map_next_index;
        let item_kind = &map_iterator.map_iteration_kind;

        if m.is_undefined() {
          return Ok(create_iter_result_object(ctx, Value::undefined(), true));
        }

        let entries = match m.get_data() {
          ObjectData::Map(data) => data,
          _ => {panic!("Trying to access map data on an Object that is not of ObjectData::Map type");}
        };

        let numEntries = entries.len();

        Ok(Value::undefined())
        /* while
        if map_iterator.map_next_index >= len {
          map_iterator.iterated_map = Value::undefined();
          return Ok(create_iter_result_object(ctx, Value::undefined(), true));
        }
        map_iterator.map_next_index = index + 1;
        match map_iterator.map_iteration_kind {
          MapIterationKind::Key => Ok(create_iter_result_object(ctx, index.into(), false)),
          MapIterationKind::Value => {
            let element_value = map_iterator.iterated_map.get_field(index);
            Ok(create_iter_result_object(ctx, element_value, false))
          }
          MapIterationKind::KeyAndValue => {
            let element_value = map_iterator.iterated_map.get_field(index);
            let result = Map::constructor(
              &Value::new_object(Some(ctx.global_object())),
              &[index.into(), element_value],
              ctx,
            )?;
            Ok(create_iter_result_object(ctx, result, false))
          }
        } */
      } else {
        ctx.throw_type_error("`this` is not an MapIterator")
      }
    } else {
      ctx.throw_type_error("`this` is not an MapIterator")
    }
  }

  /// Create the %MapIteratorPrototype% object
  ///
  /// More information:
  ///  - [ECMA reference][spec]
  ///
  /// [spec]: TODO https://tc39.es/ecma262/#sec-%arrayiteratorprototype%-object
  pub(crate) fn create_prototype(ctx: &mut Context, iterator_prototype: Value) -> Value {
    let global = ctx.global_object();
    let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

    // Create prototype
    let map_iterator = Value::new_object(Some(global));
    make_builtin_fn(Self::next, "next", &map_iterator, 0, ctx);
    map_iterator
      .as_object_mut()
      .expect("map iterator prototype object")
      .set_prototype_instance(iterator_prototype);

    let to_string_tag = ctx.well_known_symbols().to_string_tag_symbol();
    let to_string_tag_property = DataDescriptor::new("Map Iterator", Attribute::CONFIGURABLE);
    map_iterator.set_property(to_string_tag, to_string_tag_property);
    map_iterator
  }
}
