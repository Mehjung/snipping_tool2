use std::sync::Arc;
use windows::{
    core::*,
    Win32::Graphics::{Direct2D::*, DirectComposition::*, Dxgi::*},
};

pub struct ResourceManager {
    pub d2d_factory: ID2D1Factory2,
    pub dcomp_device: IDCompositionDevice,
    pub dxgi_factory: IDXGIFactory2,
}

impl ResourceManager {
    pub fn new() -> Result<Arc<Self>> {
        let d2d_factory: ID2D1Factory2 =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)? };

        let dxgi_factory: IDXGIFactory2 = unsafe { CreateDXGIFactory1()? };
        let dcomp_device: IDCompositionDevice = unsafe { DCompositionCreateDevice(None)? };

        Ok(Arc::new(Self {
            d2d_factory,
            dcomp_device,
            dxgi_factory,
        }))
    }
}
