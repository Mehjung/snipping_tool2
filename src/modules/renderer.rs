use std::mem::ManuallyDrop;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::Common::*, Direct2D::*, Direct3D::*, Direct3D11::*, DirectComposition::*,
            Dxgi::Common::*, Dxgi::*, Gdi::*,
        },
        UI::WindowsAndMessaging::*,
    },
};

pub struct Render {
    pub d2d_context: ID2D1DeviceContext,
    pub swap_chain: IDXGISwapChain1,
    pub dcomp_device: IDCompositionDevice,
    pub dcomp_target: IDCompositionTarget,
    pub dcomp_visual: IDCompositionVisual,
    pub d2d_factory: ID2D1Factory2,
    pub dpi_x: f32,
    pub dpi_y: f32,
}

impl Render {
    pub fn new(hwnd: HWND) -> Result<Self> {
        unsafe {
            let mut d3d_device: Option<ID3D11Device> = None;
            let mut d3d_context: Option<ID3D11DeviceContext> = None;
            let mut dxgi_device: Option<IDXGIDevice> = None;
            let mut dxgi_adapter: Option<IDXGIAdapter> = None;
            let mut dxgi_factory: Option<IDXGIFactory2> = None;

            let feature_levels = [D3D_FEATURE_LEVEL_11_0];
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                Some(&feature_levels),
                D3D11_SDK_VERSION,
                Some(&mut d3d_device),
                None,
                Some(&mut d3d_context),
            )
            .unwrap();

            dxgi_device = Some(d3d_device.as_ref().unwrap().cast::<IDXGIDevice>().unwrap());
            dxgi_adapter = Some(dxgi_device.as_ref().unwrap().GetAdapter().unwrap());
            dxgi_factory = Some(dxgi_adapter.as_ref().unwrap().GetParent().unwrap());

            let mut rect = RECT::default();
            GetClientRect(hwnd, &mut rect);
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: width as u32,
                Height: height as u32,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                Stereo: BOOL(0),
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                Scaling: DXGI_SCALING_STRETCH,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
                AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
                Flags: 0,
            };

            let swap_chain = dxgi_factory
                .as_ref()
                .unwrap()
                .CreateSwapChainForComposition(
                    dxgi_device.as_ref().unwrap(),
                    &swap_chain_desc,
                    None::<&IDXGIOutput>,
                )
                .unwrap();

            let d2d_factory: ID2D1Factory2 =
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).unwrap();
            let d2d_device = d2d_factory
                .CreateDevice(dxgi_device.as_ref().unwrap())
                .unwrap();
            let d2d_context1 = d2d_device
                .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
                .unwrap();
            let d2d_context: ID2D1DeviceContext = d2d_context1.cast().unwrap();

            let dcomp_device: IDCompositionDevice =
                DCompositionCreateDevice(dxgi_device.as_ref().unwrap()).unwrap();

            let dcomp_target = dcomp_device.CreateTargetForHwnd(hwnd, true).unwrap();
            let dcomp_visual = dcomp_device.CreateVisual().unwrap();

            dcomp_visual.SetContent(&swap_chain).unwrap();
            dcomp_target.SetRoot(&dcomp_visual).unwrap();
            dcomp_device.Commit().unwrap();

            let (dpi_x, dpi_y) = get_dpi_for_window(hwnd);
            println!("DPI: ({}, {})", dpi_x, dpi_y);
            d2d_context.SetDpi(dpi_x, dpi_y);

            Ok(Render {
                d2d_context,
                swap_chain,
                dcomp_device,
                dcomp_target,
                dcomp_visual,
                d2d_factory,
                dpi_x,
                dpi_y,
            })
        }
    }

    pub fn with_render_context<F>(&self, render_fn: F) -> Result<()>
    where
        F: FnOnce(&ID2D1DeviceContext) -> Result<()>,
    {
        unsafe {
            let d2d_context = &self.d2d_context;
            let swap_chain = &self.swap_chain;

            let dxgi_back_buffer = swap_chain.GetBuffer::<IDXGISurface>(0).unwrap();

            let bitmap_properties = D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: self.dpi_x,
                dpiY: self.dpi_y,
                bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                colorContext: ManuallyDrop::new(None),
            };

            let target_bitmap = d2d_context
                .CreateBitmapFromDxgiSurface(&dxgi_back_buffer, Some(&bitmap_properties))
                .unwrap();

            d2d_context.SetTarget(&target_bitmap);

            d2d_context.BeginDraw();

            d2d_context.Clear(Some(&D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }));

            render_fn(d2d_context)?;

            d2d_context.EndDraw(None, None).unwrap();
            swap_chain.Present(1, 0).unwrap();
        }
        Ok(())
    }
}

fn get_dpi_for_window(hwnd: HWND) -> (f32, f32) {
    let hdc = unsafe { GetDC(hwnd) };
    let dpi_x = unsafe { GetDeviceCaps(hdc, LOGPIXELSX) } as f32;
    let dpi_y = unsafe { GetDeviceCaps(hdc, LOGPIXELSY) } as f32;
    unsafe { ReleaseDC(hwnd, hdc) };
    (dpi_x, dpi_y)
}
