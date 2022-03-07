use crate::{Context, JsValue};

#[derive(Debug, Clone)]
pub struct JobCallback {
    this: Box<JsValue>,
    callback: Box<JsValue>,
}

impl JobCallback {
    fn new(this: JsValue, callback: JsValue) -> Self {
        Self {
            this: Box::new(this),
            callback: Box::new(callback),
        }
    }

    fn make_job_callback(this: JsValue, callback: JsValue) -> Self {
        Self::new(this, callback)
    }

    /// TODO: determine how to get rid of context
    pub fn call_job_callback(
        &self,
        v: Box<JsValue>,
        argument_list: Vec<JsValue>,
        context: &mut Context,
    ) {
        let callback = match *self.callback {
            JsValue::Object(ref object) if object.is_callable() => object.clone(),
            _ => panic!("Callback is not a callable object"),
        };

        callback.__call__(&v, &argument_list, context);
    }

    pub fn run(&self, context: &mut Context) {
        let callback = match *self.callback {
            JsValue::Object(ref object) if object.is_callable() => object.clone(),
            _ => panic!("Callback is not a callable object"),
        };

        callback.__call__(&self.this, &[], context);
    }
}
