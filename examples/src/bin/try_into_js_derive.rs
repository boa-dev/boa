use boa_engine::{
    js_string,
    value::{TryFromJs, TryIntoJs},
    Context, JsResult, JsValue, Source,
};

#[derive(TryFromJs, TryIntoJs, Debug, PartialEq, Eq)]
#[boa(rename_all = "camelCase")]
struct InnerTest {
    hello_world_how_are_you: u8,
}

#[derive(TryIntoJs)]
struct Test {
    x: i32,
    #[boa(rename = "y")]
    y_point: i32,
    #[allow(unused)]
    #[boa(skip)]
    tuple: (i32, u8, String),
    #[boa(rename = "isReadable")]
    #[boa(into_js_with = "readable_into_js")]
    is_readable: i8,

    inner: InnerTest,
}

#[derive(TryFromJs, Debug, PartialEq, Eq)]
struct ResultVerifier {
    x: i32,
    y: i32,
    #[boa(rename = "isReadable")]
    is_readable: bool,

    inner: InnerTest,
}

fn main() -> JsResult<()> {
    let js_code = r#"
    function pointShift(pointA, pointB) {
        if (pointA.isReadable === true
            && pointB.isReadable === true
            && pointA.inner.helloWorldHowAreYou !== undefined)
        {
            return {
                x: pointA.x + pointB.x,
                y: pointA.y + pointB.y,
                isReadable: true,
                inner: {
                    helloWorldHowAreYou: pointA.inner.helloWorldHowAreYou + pointB.inner.helloWorldHowAreYou
                }
            }
        }
        return undefined
    }
    "#;

    let mut context = Context::default();
    let context = &mut context;

    context.eval(Source::from_bytes(js_code))?;

    let point_shift = context
        .global_object()
        .get(js_string!("pointShift"), context)?;
    let point_shift = point_shift.as_callable().unwrap();

    let a = Test {
        x: 10,
        y_point: 20,
        tuple: (30, 40, "no matter".into()),
        is_readable: 1,
        inner: InnerTest {
            hello_world_how_are_you: 1,
        },
    };
    let b = Test {
        x: 2,
        y_point: 1,
        tuple: (30, 40, "no matter".into()),
        is_readable: 2,
        inner: InnerTest {
            hello_world_how_are_you: 2,
        },
    };
    let c = Test {
        x: 2,
        y_point: 1,
        tuple: (30, 40, "no matter".into()),
        is_readable: 0,
        inner: InnerTest {
            hello_world_how_are_you: 3,
        },
    };

    let result = point_shift.call(
        &JsValue::undefined(),
        &[a.try_into_js(context)?, b.try_into_js(context)?],
        context,
    )?;
    let verifier = ResultVerifier::try_from_js(&result, context)?;
    let expect = ResultVerifier {
        x: 10 + 2,
        y: 20 + 1,
        is_readable: true,
        inner: InnerTest {
            hello_world_how_are_you: 3,
        },
    };
    assert_eq!(verifier, expect);

    let result = point_shift.call(
        &JsValue::undefined(),
        &[a.try_into_js(context)?, c.try_into_js(context)?],
        context,
    )?;
    assert!(result.is_undefined());

    Ok(())
}

fn readable_into_js(value: &i8, _context: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::new(*value != 0))
}
