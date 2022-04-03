use crate::{
    Context,
    object::JsArray,
    value::JsValue
};

pub trait IntoJsValue {
    fn into_js_value(self, context: &mut Context) -> JsValue;
}

impl IntoJsValue for bool {
    fn into_js_value(self, _: &mut Context) -> JsValue {
        self.into()
    }
}

impl IntoJsValue for f64 {
    fn into_js_value(self, _: &mut Context) -> JsValue {
        self.into()
    }
}

impl IntoJsValue for f32 {
    fn into_js_value(self, _: &mut Context) -> JsValue {
        self.into()
    }
}

impl IntoJsValue for i128 {
    fn into_js_value(self, _: &mut Context) -> JsValue {
        self.into()
    }
}

impl IntoJsValue for i64 {
    fn into_js_value(self, _: &mut Context) -> JsValue {
        self.into()
    }
}

impl IntoJsValue for i32 {
    fn into_js_value(self, _: &mut Context) -> JsValue {
        self.into()
    }
}

impl IntoJsValue for i16 {
    fn into_js_value(self, _: &mut Context) -> JsValue {
        self.into()
    }
}

impl IntoJsValue for i8 {
    fn into_js_value(self, _: &mut Context) -> JsValue {
        self.into()
    }
}

impl<T: IntoJsValue> IntoJsValue for Option<T> {
    fn into_js_value(self, context: &mut Context) -> JsValue {
        match self {
            Some(value) => value.into_js_value(context),
            None => JsValue::Null
        }
    }
}

impl IntoJsValue for String {
    fn into_js_value(self, _: &mut Context) -> JsValue {
        self.into()
    }
}

impl IntoJsValue for u128 {
    fn into_js_value(self, _: &mut Context) -> JsValue {
        self.into()
    }
}

impl IntoJsValue for u64 {
    fn into_js_value(self, _: &mut Context) -> JsValue {
        self.into()
    }
}

impl IntoJsValue for u32 {
    fn into_js_value(self, _: &mut Context) -> JsValue {
        self.into()
    }
}

impl IntoJsValue for u16 {
    fn into_js_value(self, _: &mut Context) -> JsValue {
        self.into()
    }
}

impl IntoJsValue for u8 {
    fn into_js_value(self, _: &mut Context) -> JsValue {
        self.into()
    }
}

// TODO consider that each type might need its own explicit impl for Vec
// TODO the derive attribute might need to be used in that case
impl<T: IntoJsValue> IntoJsValue for Vec<T> {
    fn into_js_value(self, context: &mut Context) -> JsValue {
        let js_values = self.into_iter().map(|item| item.into_js_value(context)).collect::<Vec<JsValue>>();
        JsArray::from_iter(js_values, context).into()
    }
}

// TODO I would like to create typed arrays for u8 etc in addition to the impl for Vec<T>
// impl IntoJsValue for Vec<u8> {
//     fn into_js_value(self, context: &mut Context) -> JsValue {
//         let js_values = self.into_iter().map(|item| item.into_js_value(context)).collect::<Vec<JsValue>>();

//         Array::create_array_from_list(
//             js_values,
//             context
//         ).into()
//     }
// }