//! Tests for native class inheritance through the unified `Class` trait.
//!
//! Mirrors the `Animal` -> `Dog` hierarchy cases from `boa-extends`.

use crate::{
    class::{with_class_data, with_class_data_mut, Class, ClassBuilder, NoParent},
    js_string, Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsValue, NativeFunction,
    Source,
};
use crate::object::FunctionObjectBuilder;
use crate::property::Attribute;
use boa_gc::{Finalize, Trace};

// ── Data layers ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Trace, Finalize, JsData)]
struct AnimalData {
    name: String,
    tricks: u32,
}

#[derive(Debug, Clone, Trace, Finalize, JsData)]
struct DogData {
    breed: String,
}

// ── Native classes ───────────────────────────────────────────────────

#[derive(Debug, Clone, Trace, Finalize, JsData)]
struct Animal;

impl Class for Animal {
    const NAME: &'static str = "Animal";
    const LENGTH: usize = 1;

    type Parent = NoParent;
    type Data = AnimalData;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        ctx: &mut Context,
    ) -> JsResult<Self::Data> {
        let name = args
            .get_or_undefined(0)
            .to_string(ctx)?
            .to_std_string_escaped();
        Ok(AnimalData { name, tricks: 0 })
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();
        let attr = Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE;
        class.accessor(
            js_string!("name"),
            Some(
                FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(animal_name_getter))
                    .name(js_string!("get name"))
                    .build(),
            ),
            None,
            attr,
        );
        class.accessor(
            js_string!("species"),
            Some(
                FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(animal_species_getter))
                    .name(js_string!("get species"))
                    .build(),
            ),
            None,
            attr,
        );
        class.accessor(
            js_string!("tricks"),
            Some(
                FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(animal_tricks_getter))
                    .name(js_string!("get tricks"))
                    .build(),
            ),
            None,
            attr,
        );
        class.method(
            js_string!("train"),
            0,
            NativeFunction::from_fn_ptr(animal_train),
        );
        class.method(
            js_string!("greet"),
            0,
            NativeFunction::from_fn_ptr(animal_greet),
        );
        Ok(())
    }
}

#[derive(Debug, Clone, Trace, Finalize, JsData)]
struct Dog;

impl Class for Dog {
    const NAME: &'static str = "Dog";
    const LENGTH: usize = 2;

    type Parent = Animal;
    type Data = DogData;

    fn data_constructor(
        _new_target: &JsValue,
        args: &[JsValue],
        ctx: &mut Context,
    ) -> JsResult<Self::Data> {
        let breed = args
            .get_or_undefined(1)
            .to_string(ctx)?
            .to_std_string_escaped();
        Ok(DogData { breed })
    }

    fn parent_args<'a>(
        _new_target: &JsValue,
        args: &'a [JsValue],
        _ctx: &mut Context,
    ) -> JsResult<&'a [JsValue]> {
        Ok(&args[..1.min(args.len())])
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        let realm = class.context().realm().clone();
        let attr = Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE;
        class.accessor(
            js_string!("breed"),
            Some(
                FunctionObjectBuilder::new(&realm, NativeFunction::from_fn_ptr(dog_breed_getter))
                    .name(js_string!("get breed"))
                    .build(),
            ),
            None,
            attr,
        );
        class.method(js_string!("bark"), 0, NativeFunction::from_fn_ptr(dog_bark));
        class.method(
            js_string!("greet"),
            0,
            NativeFunction::from_fn_ptr(dog_greet),
        );
        Ok(())
    }
}

// ── Methods / getters ────────────────────────────────────────────────

fn as_object(this: &JsValue) -> JsResult<JsObject> {
    this.as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("not an object").into())
}

fn animal_name_getter(this: &JsValue, _: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    let obj = as_object(this)?;
    with_class_data::<Animal, _>(&obj, |d| js_string!(d.name.as_str()).into())
}

fn animal_species_getter(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(js_string!("animal").into())
}

fn animal_tricks_getter(this: &JsValue, _: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    let obj = as_object(this)?;
    with_class_data::<Animal, _>(&obj, |d| JsValue::new(d.tricks))
}

fn animal_train(this: &JsValue, _: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    let obj = as_object(this)?;
    let new_count = with_class_data_mut::<Animal, _>(&obj, |d| {
        d.tricks += 1;
        d.tricks
    })?;
    Ok(JsValue::new(new_count))
}

fn animal_greet(this: &JsValue, _: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    let obj = as_object(this)?;
    let name = with_class_data::<Animal, _>(&obj, |d| d.name.clone())?;
    Ok(js_string!(format!("Hi, I'm {name}!")).into())
}

fn dog_breed_getter(this: &JsValue, _: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    let obj = as_object(this)?;
    with_class_data::<Dog, _>(&obj, |d| js_string!(d.breed.as_str()).into())
}

fn dog_bark(this: &JsValue, _: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    let obj = as_object(this)?;
    let name = with_class_data::<Animal, _>(&obj, |d| d.name.clone())?;
    Ok(js_string!(format!("{name} says: Woof!")).into())
}

fn dog_greet(this: &JsValue, _: &[JsValue], _ctx: &mut Context) -> JsResult<JsValue> {
    let obj = as_object(this)?;
    let name = with_class_data::<Animal, _>(&obj, |a| a.name.clone())?;
    let breed = with_class_data::<Dog, _>(&obj, |d| d.breed.clone())?;
    Ok(js_string!(format!("Woof! I'm {name}, a {breed}.")).into())
}

// ── Test harness ─────────────────────────────────────────────────────

fn ctx() -> Context {
    let mut ctx = Context::default();
    ctx.register_global_class::<Animal>().expect("register Animal");
    ctx.register_global_class::<Dog>().expect("register Dog");
    ctx
}

fn eval_str(ctx: &mut Context, src: &str) -> String {
    ctx.eval(Source::from_bytes(src))
        .unwrap_or_else(|e| panic!("eval failed: {e}\nsrc:\n{src}"))
        .to_string(ctx)
        .unwrap()
        .to_std_string_escaped()
}

fn eval_throws(ctx: &mut Context, src: &str) -> String {
    let wrapped = format!(
        r#"(function() {{ try {{ {src}; return "NO_THROW"; }} catch(e) {{ return String(e); }} }})()"#
    );
    ctx.eval(Source::from_bytes(&wrapped))
        .expect("wrapper failed")
        .to_string(ctx)
        .unwrap()
        .to_std_string_escaped()
}

// ── JS `new` path ────────────────────────────────────────────────────

#[test]
fn js_new_dog_populates_class_data() {
    let mut ctx = ctx();
    let val = ctx
        .eval(Source::from_bytes(
            r#"const d = new Dog("Rex", "Labrador"); ({ name: d.name, breed: d.breed, species: d.species, tricks: d.tricks })"#,
        ))
        .unwrap();
    let obj = val.as_object().unwrap();
    assert_eq!(
        obj.get(js_string!("name"), &mut ctx)
            .unwrap()
            .to_string(&mut ctx)
            .unwrap()
            .to_std_string_escaped(),
        "Rex"
    );
    assert_eq!(
        obj.get(js_string!("breed"), &mut ctx)
            .unwrap()
            .to_string(&mut ctx)
            .unwrap()
            .to_std_string_escaped(),
        "Labrador"
    );
    assert_eq!(
        obj.get(js_string!("species"), &mut ctx)
            .unwrap()
            .to_string(&mut ctx)
            .unwrap()
            .to_std_string_escaped(),
        "animal"
    );
    assert_eq!(
        obj.get(js_string!("tricks"), &mut ctx)
            .unwrap()
            .as_number()
            .unwrap(),
        0.0
    );
}

#[test]
fn js_new_animal_works() {
    let mut ctx = ctx();
    let val = ctx
        .eval(Source::from_bytes(r#"const a = new Animal("Polly"); a.name + "|" + a.species + "|" + a.tricks"#))
        .unwrap();
    assert_eq!(
        val.to_string(&mut ctx).unwrap().to_std_string_escaped(),
        "Polly|animal|0"
    );
}

#[test]
fn js_train_increments_tricks() {
    let mut ctx = ctx();
    let val = ctx
        .eval(Source::from_bytes(
            r#"const d = new Dog("Rex", "Lab"); d.train(); d.train(); d.train(); d.tricks"#,
        ))
        .unwrap();
    assert_eq!(val.as_number().unwrap(), 3.0);
}

// ── Prototype chain ──────────────────────────────────────────────────

#[test]
fn dog_is_instanceof_animal() {
    let mut ctx = ctx();
    let val = ctx
        .eval(Source::from_bytes(
            r#"const d = new Dog("Rex", "Lab"); d instanceof Dog && d instanceof Animal"#,
        ))
        .unwrap();
    assert!(val.as_boolean().unwrap());
}

#[test]
fn dog_inherits_animal_methods_via_prototype() {
    let mut ctx = ctx();
    let val = ctx
        .eval(Source::from_bytes(
            r#"const d = new Dog("Rex", "Lab"); d.train(); d.tricks === 1"#,
        ))
        .unwrap();
    assert!(val.as_boolean().unwrap());
}

#[test]
fn dog_greet_overrides_animal_greet() {
    let mut ctx = ctx();
    let val = ctx
        .eval(Source::from_bytes(r#"const d = new Dog("Rex", "Labrador"); d.greet()"#))
        .unwrap();
    assert_eq!(
        val.to_string(&mut ctx).unwrap().to_std_string_escaped(),
        "Woof! I'm Rex, a Labrador."
    );
}

// ── Rust `from_data` path (root class only in this first cut) ────────

#[test]
fn from_data_builds_animal() {
    let mut ctx = ctx();
    let animal = <Animal as Class>::from_data(
        AnimalData {
            name: "Polly".into(),
            tricks: 0,
        },
        &mut ctx,
    )
    .unwrap();
    let name = with_class_data::<Animal, _>(&animal, |d| d.name.clone()).unwrap();
    assert_eq!(name, "Polly");
}

// ── with_class_data / with_class_data_mut ────────────────────────────

#[test]
fn with_class_data_mut_updates_tricks() {
    let mut ctx = ctx();
    let args = vec![
        JsValue::from(js_string!("Rex")),
        JsValue::from(js_string!("Lab")),
    ];
    let dog = Dog::construct(&JsValue::from(js_string!("Dog")), &args, &mut ctx).unwrap();
    let before = with_class_data::<Animal, _>(&dog, |d| d.tricks).unwrap();
    assert_eq!(before, 0);
    let after = with_class_data_mut::<Animal, _>(&dog, |d| {
        d.tricks = 42;
        d.tricks
    })
    .unwrap();
    assert_eq!(after, 42);
    let again = with_class_data::<Animal, _>(&dog, |d| d.tricks).unwrap();
    assert_eq!(again, 42);
}

// ── Error handling ───────────────────────────────────────────────────

#[test]
fn new_without_new_throws_type_error() {
    let mut ctx = ctx();
    let val = ctx
        .eval(Source::from_bytes(r#"try { Dog("Rex", "Lab"); false } catch (e) { true }"#))
        .unwrap();
    assert!(val.as_boolean().unwrap());
}

#[test]
fn class_data_missing_yields_type_error() {
    let ctx = ctx();
    let obj = JsObject::with_object_proto(&ctx.intrinsics());
    let err = with_class_data::<Animal, _>(&obj, |_| ()).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("native class data"), "got: {msg}");
}

// ── Class identity ───────────────────────────────────────────────────

#[test]
fn class_names_are_set() {
    assert_eq!(Animal::NAME, "Animal");
    assert_eq!(Dog::NAME, "Dog");
}

// ── JS subclass extends native type ──────────────────────────────────

#[test]
fn js_subclass_super_drives_native_data() {
    let mut ctx = ctx();
    assert_eq!(
        eval_str(
            &mut ctx,
            r#"
                class GuardDog extends Dog {
                    constructor(name, breed, zone) { super(name, breed); this.zone = zone; }
                }
                const g = new GuardDog("Axel", "Shepherd", "north");
                g.name + "," + g.breed + "," + g.zone
            "#,
        ),
        "Axel,Shepherd,north",
    );
}

#[test]
fn js_subclass_super_greet_hits_native_override() {
    let mut ctx = ctx();
    assert_eq!(
        eval_str(
            &mut ctx,
            r#"
                class GuardDog extends Dog {
                    constructor(n, b) { super(n, b); }
                    greet() { return "[" + super.greet() + "]"; }
                }
                new GuardDog("Rex", "Lab").greet()
            "#,
        ),
        "[Woof! I'm Rex, a Lab.]",
    );
}

#[test]
fn js_subclass_can_shadow_native_method() {
    let mut ctx = ctx();
    assert_eq!(
        eval_str(
            &mut ctx,
            r#"
                class Mute extends Dog {
                    constructor(n, b) { super(n, b); }
                    bark() { return "..."; }
                }
                new Mute("Rex", "Lab").bark()
            "#,
        ),
        "...",
    );
}

// ── Reflect.construct with alternate new_target ──────────────────────

#[test]
fn reflect_construct_with_alt_prototype_keeps_class_data() {
    let mut ctx = ctx();
    let got = eval_str(
        &mut ctx,
        r#"
            class Wolf {}
            const d = Reflect.construct(Dog, ["Ghost", "Dire"], Wolf);
            (d instanceof Wolf) + "," + (d instanceof Dog) + "," + (d.name === undefined)
        "#,
    );
    assert_eq!(got, "true,false,true");
}

// ── this binding / wrong receiver ────────────────────────────────────

#[test]
fn native_method_on_plain_object_throws() {
    let mut ctx = ctx();
    let msg = eval_throws(&mut ctx, r#"Dog.prototype.bark.call({})"#);
    assert_ne!(msg, "NO_THROW", "expected a throw");
    assert!(msg.contains("native class data"), "got {msg:?}");
}

#[test]
fn native_getter_on_plain_object_throws() {
    let mut ctx = ctx();
    let msg = eval_throws(
        &mut ctx,
        r#"Object.getOwnPropertyDescriptor(Dog.prototype, "name").get.call({})"#,
    );
    assert_ne!(msg, "NO_THROW", "expected a throw");
}

// ── Argument coercion boundary ──────────────────────────────────────

#[test]
fn symbol_arg_throws_during_build() {
    let mut ctx = ctx();
    let msg = eval_throws(&mut ctx, r#"new Dog(Symbol(), "x")"#);
    assert_ne!(msg, "NO_THROW");
    assert!(msg.to_lowercase().contains("symbol"), "got {msg:?}");
}

// ── Cross-instance isolation ─────────────────────────────────────────

#[test]
fn class_data_not_shared_across_instances() {
    let mut ctx = ctx();
    assert_eq!(
        eval_str(
            &mut ctx,
            r#"
                const a = new Dog("A", "Lab");
                const b = new Dog("B", "Poodle");
                a.train(); a.train(); a.train();
                a.tricks + "," + b.tricks
            "#,
        ),
        "3,0",
    );
}

// ── Missing super in JS subclass ─────────────────────────────────────

#[test]
fn js_subclass_omitting_super_throws() {
    let mut ctx = ctx();
    let msg = eval_throws(
        &mut ctx,
        r#"
            class Bad extends Dog { constructor() { this.x = 1; } }
            new Bad()
        "#,
    );
    assert_ne!(msg, "NO_THROW");
    assert!(msg.to_lowercase().contains("super"), "got {msg:?}");
}

// ── Calling without new ──────────────────────────────────────────────

#[test]
fn calling_without_new_throws() {
    let mut ctx = ctx();
    let msg = eval_throws(&mut ctx, r#"Dog("Rex", "Lab")"#);
    assert_ne!(msg, "NO_THROW");
    assert!(msg.contains("without new"), "got {msg:?}");
}
