use std::borrow::Borrow;

use super::{Display, JsBigInt, JsObject, JsString, JsSymbol, JsValue, Profiler};
use crate::{
    Context,
    builtins::Array
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
            None => JsValue::Null.into()
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

        Array::create_array_from_list(
            js_values,
            context
        ).into()
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

// TODO do not unwrap, use the term try for function name instead
pub trait FromJsValue<T> {
    fn from_js_value(self, context: &mut Context) -> T;
}

impl FromJsValue<bool> for JsValue {
    fn from_js_value(self, _: &mut Context) -> bool {
        self.as_boolean().unwrap()
    }
}

impl FromJsValue<f64> for JsValue {
    fn from_js_value(self, _: &mut Context) -> f64 {
        self.as_number().unwrap()
    }
}

impl FromJsValue<f32> for JsValue {
    fn from_js_value(self, _: &mut Context) -> f32 {
        self.as_number().unwrap() as f32
    }
}

impl FromJsValue<i128> for JsValue {
    fn from_js_value(self, _: &mut Context) -> i128 {
        self.as_bigint().unwrap().to_string().parse::<i128>().unwrap()
    }
}

impl FromJsValue<i64> for JsValue {
    fn from_js_value(self, _: &mut Context) -> i64 {
        self.as_bigint().unwrap().to_string().parse::<i64>().unwrap()
    }
}

impl FromJsValue<i32> for JsValue {
    fn from_js_value(self, _: &mut Context) -> i32 {
        self.as_number().unwrap() as i32
    }
}

impl FromJsValue<i16> for JsValue {
    fn from_js_value(self, _: &mut Context) -> i16 {
        self.as_number().unwrap() as i16
    }
}

impl FromJsValue<i8> for JsValue {
    fn from_js_value(self, _: &mut Context) -> i8 {
        self.as_number().unwrap() as i8
    }
}

impl<T> FromJsValue<Option<T>> for JsValue where JsValue: FromJsValue<T> {
    fn from_js_value(self, context: &mut Context) -> Option<T> {
        if self.is_null() {
            None
        }
        else {
            Some(self.from_js_value(context))
        }
    }
}

impl FromJsValue<String> for JsValue {
    fn from_js_value(self, _: &mut Context) -> String {
        self.as_string().unwrap().to_string()
    }
}

impl FromJsValue<u128> for JsValue {
    fn from_js_value(self, _: &mut Context) -> u128 {
        self.as_bigint().unwrap().to_string().parse::<u128>().unwrap()
    }
}

impl FromJsValue<u64> for JsValue {
    fn from_js_value(self, _: &mut Context) -> u64 {
        self.as_bigint().unwrap().to_string().parse::<u64>().unwrap()
    }
}

impl FromJsValue<u32> for JsValue {
    fn from_js_value(self, _: &mut Context) -> u32 {
        self.as_number().unwrap() as u32
    }
}

impl FromJsValue<u16> for JsValue {
    fn from_js_value(self, _: &mut Context) -> u16 {
        self.as_number().unwrap() as u16
    }
}

impl FromJsValue<u8> for JsValue {
    fn from_js_value(self, _: &mut Context) -> u8 {
        self.as_number().unwrap() as u8
    }
}

impl<T> FromJsValue<Vec<T>> for JsValue where JsValue: FromJsValue<T> {
    fn from_js_value(self, context: &mut Context) -> Vec<T> {
        let js_object = self.as_object().unwrap();

        // TODO there must be a better way to do this
        // TODO why is there no as_array?
        // TODO possibly implement as_array
        if js_object.is_array() {
            let mut processing: bool = true;
            let mut index: usize = 0;

            let mut result = vec![];

            while processing == true {
                let js_value = js_object.get(index, context).unwrap();
            
                if js_value.is_undefined() {
                    processing = false;
                }
                else {
                    result.push(js_value.from_js_value(context));
                    index += 1;
                }
            }

            result
        }
        else {
            panic!("Cannot happen");
        }
    }
}