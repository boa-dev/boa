use super::JsValue;
use crate::Context;

#[derive(Debug)]
pub struct TryFromJsValueError(pub String);

pub trait TryFromJsValue<T> {
    fn try_from_js_value(self, context: &mut Context) -> Result<T, TryFromJsValueError>;
}

impl TryFromJsValue<bool> for JsValue {
    fn try_from_js_value(self, _: &mut Context) -> Result<bool, TryFromJsValueError> {
        match self.as_boolean() {
            Some(value) => Ok(value),
            None => Err(TryFromJsValueError("JsValue is not a boolean".to_string()))
        }
    }
}

impl TryFromJsValue<f64> for JsValue {
    fn try_from_js_value(self, _: &mut Context) -> Result<f64, TryFromJsValueError> {
        match self.as_number() {
            Some(value) => Ok(value),
            None => Err(TryFromJsValueError("JsValue is not a number".to_string()))
        }
    }
}

impl TryFromJsValue<f32> for JsValue {
    fn try_from_js_value(self, _: &mut Context) -> Result<f32, TryFromJsValueError> {
        match self.as_number() {
            Some(value) => Ok(value as f32),
            None => Err(TryFromJsValueError("JsValue is not a number".to_string()))
        }
    }
}

impl TryFromJsValue<i128> for JsValue {
    fn try_from_js_value(self, _: &mut Context) -> Result<i128, TryFromJsValueError> {
        match self.as_bigint() {
            Some(value) => {
                let value_i128_result = value.to_string().parse::<i128>();

                match value_i128_result {
                    Ok(value_i128) => Ok(value_i128),
                    Err(_) => Err(TryFromJsValueError("Could not parse bigint to i128".to_string()))
                }
            },
            None => Err(TryFromJsValueError("JsValue is not a bigint".to_string()))
        }
    }
}

// TODO this might break since i64 may (will) not be a bigint
impl TryFromJsValue<i64> for JsValue {
    fn try_from_js_value(self, _: &mut Context) -> Result<i64, TryFromJsValueError> {
        match self.as_bigint() {
            Some(value) => {
                let value_i64_result = value.to_string().parse::<i64>();

                match value_i64_result {
                    Ok(value_i64) => Ok(value_i64),
                    Err(_) => Err(TryFromJsValueError("Could not parse bigint to i64".to_string()))
                }
            },
            None => Err(TryFromJsValueError("JsValue is not a bigint".to_string()))
        }
    }
}

impl TryFromJsValue<i32> for JsValue {
    fn try_from_js_value(self, _: &mut Context) -> Result<i32, TryFromJsValueError> {
        match self.as_number() {
            Some(value) => Ok(value as i32),
            None => Err(TryFromJsValueError("JsValue is not a number".to_string()))
        }
    }
}

impl TryFromJsValue<i16> for JsValue {
    fn try_from_js_value(self, _: &mut Context) -> Result<i16, TryFromJsValueError> {
        match self.as_number() {
            Some(value) => Ok(value as i16),
            None => Err(TryFromJsValueError("JsValue is not a number".to_string()))
        }
    }
}

impl TryFromJsValue<i8> for JsValue {
    fn try_from_js_value(self, _: &mut Context) -> Result<i8, TryFromJsValueError> {
        match self.as_number() {
            Some(value) => Ok(value as i8),
            None => Err(TryFromJsValueError("JsValue is not a number".to_string()))
        }
    }
}

impl<T> TryFromJsValue<Option<T>> for JsValue where JsValue: TryFromJsValue<T> {
    fn try_from_js_value(self, context: &mut Context) -> Result<Option<T>, TryFromJsValueError> {
        if self.is_null() {
            Ok(None)
        }
        else {
            match self.try_from_js_value(context) {
                Ok(value) => Ok(Some(value)),
                Err(err) => Err(err)
            }
        }
    }
}

impl TryFromJsValue<String> for JsValue {
    fn try_from_js_value(self, _: &mut Context) -> Result<String, TryFromJsValueError> {
        match self.as_string() {
            Some(value) => Ok(value.to_string()),
            None => Err(TryFromJsValueError("JsValue is not a string".to_string()))
        }
    }
}

impl TryFromJsValue<u128> for JsValue {
    fn try_from_js_value(self, _: &mut Context) -> Result<u128, TryFromJsValueError> {
        match self.as_bigint() {
            Some(value) => {
                let value_u128_result = value.to_string().parse::<u128>();

                match value_u128_result {
                    Ok(value_u128) => Ok(value_u128),
                    Err(_) => Err(TryFromJsValueError("Could not parse bigint to u128".to_string()))
                }
            },
            None => Err(TryFromJsValueError("JsValue is not a bigint".to_string()))
        }
    }
}

// TODO this might break since i64 may (will) not be a bigint
impl TryFromJsValue<u64> for JsValue {
    fn try_from_js_value(self, _: &mut Context) -> Result<u64, TryFromJsValueError> {
        match self.as_bigint() {
            Some(value) => {
                let value_u64_result = value.to_string().parse::<u64>();

                match value_u64_result {
                    Ok(value_u64) => Ok(value_u64),
                    Err(_) => Err(TryFromJsValueError("Could not parse bigint to u64".to_string()))
                }
            },
            None => Err(TryFromJsValueError("JsValue is not a bigint".to_string()))
        }
    }
}

impl TryFromJsValue<u32> for JsValue {
    fn try_from_js_value(self, _: &mut Context) -> Result<u32, TryFromJsValueError> {
        match self.as_number() {
            Some(value) => Ok(value as u32),
            None => Err(TryFromJsValueError("JsValue is not a number".to_string()))
        }
    }
}

impl TryFromJsValue<u16> for JsValue {
    fn try_from_js_value(self, _: &mut Context) -> Result<u16, TryFromJsValueError> {
        match self.as_number() {
            Some(value) => Ok(value as u16),
            None => Err(TryFromJsValueError("JsValue is not a number".to_string()))
        }
    }
}

impl TryFromJsValue<u8> for JsValue {
    fn try_from_js_value(self, _: &mut Context) -> Result<u8, TryFromJsValueError> {
        match self.as_number() {
            Some(value) => Ok(value as u8),
            None => Err(TryFromJsValueError("JsValue is not a number".to_string()))
        }
    }
}

impl<T> TryFromJsValue<Vec<T>> for JsValue where JsValue: TryFromJsValue<T> {
    fn try_from_js_value(self, context: &mut Context) -> Result<Vec<T>, TryFromJsValueError> {
        match self.as_object() {
            Some(js_object) => {
                if js_object.is_array() {
                    let mut processing: bool = true;
                    let mut index: usize = 0;
        
                    let mut result = vec![];
        
                    while processing == true {
                        match js_object.get(index, context) {
                            Ok(js_value) => {
                                if js_value.is_undefined() {
                                    processing = false;
                                }
                                else {
                                    match js_value.try_from_js_value(context) {
                                        Ok(value) => {
                                            result.push(value);
                                            index += 1;
                                        }
                                        Err(err) => {
                                            return Err(err);
                                        }
                                    }
                                }
                            },
                            Err(_) => {
                                return Err(TryFromJsValueError("Item at array index does not exist".to_string()))
                            }
                        }
                    }
        
                    Ok(result)
                }
                else {
                    Err(TryFromJsValueError("JsObject is not an array".to_string()))
                }
            },
            None => Err(TryFromJsValueError("JsValue is not an object".to_string()))
        }
    }
}