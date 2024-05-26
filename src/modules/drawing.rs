use crate::modules::renderer::Render;
use crate::modules::resource_manager::ResourceManager;
use std::mem::ManuallyDrop;


use windows::core::Result;
use windows::Foundation::Numerics::Matrix3x2;
use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;
use windows::Win32::Graphics::Direct2D::{ID2D1Brush, ID2D1Geometry, D2D1_COMBINE_MODE_EXCLUDE};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, HDC, PAINTSTRUCT};
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::{Direct2D::Common::*, Direct2D::*},
    },
};

#[derive(Clone)]

pub struct Drawing {
    render: Render,
}

impl Drawing {
    pub fn new(hwnd: HWND, resource_manager: &ResourceManager) -> Self {
        let render = Render::new(hwnd, resource_manager).unwrap();
        Drawing { render }
    }

    fn provide_env<F>(&self, hwnd: HWND, render_fn: F) -> Result<()>
    where
        F: FnOnce(&HDC) -> Result<()>,
    {
        let mut ps = PAINTSTRUCT::default();
        let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
        if hdc.0 == 0 {
            return Err(windows::core::Error::from_win32());
        }

        let result = render_fn(&hdc);

        let _ = unsafe { EndPaint(hwnd, &ps) };

        result
    }

    pub fn draw_overlay(&self, hwnd: HWND, rect: Option<D2D_RECT_F>) -> Result<()> {
        self.provide_env(hwnd, |_hdc| {
            self.render.with_render_context(|d2d_context| {
                if let Some(rect) = rect {
                    unsafe {
                        // Hier fügt man die Zeichenlogik ein, die das Overlay zeichnet
                        let brush_color = D2D1_COLOR_F {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        };
                        let brush = d2d_context
                            .CreateSolidColorBrush(&brush_color, None)
                            .unwrap();

                        let size = d2d_context.GetSize();

                        let full_rect_geometry =
                            self.render
                                .d2d_factory
                                .CreateRectangleGeometry(&D2D_RECT_F {
                                    left: 0.0,
                                    top: 0.0,
                                    right: size.width,
                                    bottom: size.height,
                                })?;

                        let excluded_rect_geometry =
                            self.render.d2d_factory.CreateRectangleGeometry(&rect)?;

                        let combined_geometry = self.render.d2d_factory.CreatePathGeometry()?;
                        let sink = combined_geometry.Open()?;
                        sink.SetFillMode(D2D1_FILL_MODE_WINDING);
                        full_rect_geometry.CombineWithGeometry(
                            &excluded_rect_geometry,
                            D2D1_COMBINE_MODE_EXCLUDE,
                            None,
                            0.001,
                            &sink,
                        )?;
                        sink.Close()?;

                        let geom_mask = ManuallyDrop::new(Some(
                            combined_geometry.cast::<ID2D1Geometry>().unwrap(),
                        ));
                        let layer_parameters = D2D1_LAYER_PARAMETERS1 {
                            contentBounds: D2D_RECT_F {
                                left: 0.0,
                                top: 0.0,
                                right: size.width,
                                bottom: size.height,
                            },
                            geometricMask: geom_mask,
                            maskAntialiasMode: D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
                            maskTransform: Matrix3x2::identity(),
                            opacity: 0.6,
                            opacityBrush: ManuallyDrop::new(Some(
                                brush.cast::<ID2D1Brush>().unwrap(),
                            )),
                            layerOptions: D2D1_LAYER_OPTIONS1_NONE,
                        };

                        let layer = d2d_context.CreateLayer(Some(&size))?;

                        d2d_context.PushLayer(&layer_parameters, &layer);
                        d2d_context.FillRectangle(
                            &D2D_RECT_F {
                                left: 0.0,
                                top: 0.0,
                                right: size.width,
                                bottom: size.height,
                            },
                            &brush,
                        );
                        d2d_context.PopLayer();
                    }
                }

                Ok(())
            })
        })
    }
    pub fn fill_background(&self, hwnd: HWND, color: D2D1_COLOR_F) -> Result<()> {
        self.provide_env(hwnd, |_hdc| {
            self.render.with_render_context(|d2d_context| {
                unsafe {
                    let brush = d2d_context.CreateSolidColorBrush(&color, None).unwrap();

                    let size = d2d_context.GetSize();

                    d2d_context.FillRectangle(
                        &D2D_RECT_F {
                            left: 0.0,
                            top: 0.0,
                            right: size.width,
                            bottom: size.height,
                        },
                        &brush,
                    );
                }

                Ok(())
            })
        })
    }
}
