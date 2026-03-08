use boa_engine::{
    Context, JsData, JsNativeError, JsObject, JsResult, boa_class, class::Class, realm::Realm,
    value::Convert,
};
use boa_gc::{Finalize, Trace};
use futures_lite::future::block_on;

#[derive(Trace, Finalize, JsData, Default)]
pub struct Instance {
    #[unsafe_ignore_trace]
    pub inner: wgpu::Instance,
}

#[boa_class(rename = "GPU")]
#[boa(rename_all = "camelCase")]
impl Instance {
    fn request_adapter(&self, context: &mut Context) -> JsResult<JsObject> {
        let adapter = block_on(
            self.inner
                .request_adapter(&wgpu::RequestAdapterOptions::default()),
        )
        .expect("no adapter");
        Class::from_data(Adapter { inner: adapter }, context)
    }
}

#[derive(Trace, Finalize, JsData)]
pub struct Adapter {
    #[unsafe_ignore_trace]
    pub inner: wgpu::Adapter,
}

#[boa_class(rename = "GPUAdapter")]
#[boa(rename_all = "camelCase")]
impl Adapter {
    #[boa(constructor)]
    fn contructor() -> JsResult<Self> {
        Err(JsNativeError::typ().into())
    }

    fn request_device(&self, context: &mut Context) -> JsResult<JsObject> {
        let (device, queue) = block_on(
            self.inner
                .request_device(&wgpu::DeviceDescriptor::default()),
        )
        .expect("no device");
        let queue = Queue { inner: queue };
        let queue_object = Class::from_data(queue.clone(), context)?;
        Class::from_data(
            Device {
                inner: device,
                queue,
                queue_object,
            },
            context,
        )
    }
}

#[derive(Trace, Finalize, JsData)]
pub struct Device {
    #[unsafe_ignore_trace]
    pub inner: wgpu::Device,
    pub queue: Queue,
    pub queue_object: JsObject,
}

#[boa_class(rename = "GPUDevice")]
#[boa(rename_all = "camelCase")]
impl Device {
    #[boa(constructor)]
    fn contructor() -> JsResult<Self> {
        Err(JsNativeError::typ().into())
    }

    #[boa(getter)]
    fn queue(&self) -> JsObject {
        self.queue_object.clone()
    }

    fn create_buffer(&self, context: &mut Context) -> JsResult<JsObject> {
        let buffer = self.inner.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Class::from_data(Buffer { inner: buffer }, context)
    }
}

#[derive(Trace, Finalize, JsData, Clone)]
pub struct Queue {
    #[unsafe_ignore_trace]
    pub inner: wgpu::Queue,
}

#[boa_class(rename = "GPUQueue")]
#[boa(rename_all = "camelCase")]
impl Queue {
    #[boa(constructor)]
    fn contructor() -> JsResult<Self> {
        Err(JsNativeError::typ().into())
    }

    fn write_buffer(&self, buffer: JsObject) -> JsResult<()> {
        let Some(buffer) = buffer.downcast_ref::<Buffer>() else {
            return Err(JsNativeError::typ().into());
        };
        self.inner.write_buffer(&buffer.inner, 0, &[1, 2, 3, 4]);
        Ok(())
    }
}

#[derive(Trace, Finalize, JsData)]
pub struct Buffer {
    #[unsafe_ignore_trace]
    pub inner: wgpu::Buffer,
}

#[boa_class(rename = "GPUBuffer")]
#[boa(rename_all = "camelCase")]
impl Buffer {
    #[boa(constructor)]
    fn contructor() -> JsResult<Self> {
        Err(JsNativeError::typ().into())
    }
}

pub fn register(context: &mut Context) -> JsResult<()> {
    context.register_global_class::<Instance>()?;
    context.register_global_class::<Adapter>()?;
    context.register_global_class::<Device>()?;
    context.register_global_class::<Queue>()?;
    context.register_global_class::<Buffer>()?;

    // Register the global `navigator` object and attach the `gpu` instance to it.
    // This is required to expose the WebGPU entry point in the JavaScript environment,
    // allowing scripts to access `navigator.gpu` as per the WebGPU specification.

    let instance = Instance::default();
    let gpu_obj = Class::from_data(instance, context)?;

    let navigator = boa_engine::object::ObjectInitializer::new(context)
        .property(
            boa_engine::js_string!("gpu"),
            gpu_obj,
            boa_engine::property::Attribute::all(),
        )
        .build();

    context.register_global_property(
        boa_engine::js_string!("navigator"),
        navigator,
        boa_engine::property::Attribute::all(),
    )?;

    Ok(())
}
