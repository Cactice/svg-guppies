mod setup;
pub use pollster;
use setup::Setup;
pub use tesselation;
use tesselation::geometry::{SvgSet};
pub use tesselation::glam;
use tesselation::glam::{Mat4, Vec2};
pub use winit;
use winit::dpi::PhysicalSize;
use winit::window::WindowBuilder;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

pub fn get_scale(size: PhysicalSize<u32>, svg_scale: Vec2) -> Mat4 {
    let ratio = f32::min(1200_f32, 1600_f32) / f32::max(svg_scale.x, svg_scale.y);
    Mat4::from_scale(
        [
            2.0 * ratio / size.width as f32,
            -2.0 * ratio / size.height as f32,
            1.0,
        ]
        .into(),
    )
}

pub trait ViewModel {
    fn into_bytes(&self) -> Option<Vec<u8>>;
    fn into_texts(&self) -> Option<Vec<(String, String)>>;
    fn reset_mut_count(&mut self);
    fn on_event(&mut self, svg_set: &SvgSet, event: WindowEvent);
}

pub fn main<V: ViewModel + 'static>(svg_set: SvgSet<'static>, mut view_model: V) {
    let event_loop = EventLoop::new();
    let vertices = svg_set.geometry_set.get_vertices();
    let indices = svg_set.geometry_set.get_indices();
    let mut redraw = None;
    let mut window = None;

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(start_cause) => match start_cause {
                winit::event::StartCause::Init => {
                    window = Some(
                        WindowBuilder::new()
                            .with_title("SVG-GUI")
                            .build(event_loop)
                            .unwrap(),
                    );
                    let window = window.as_ref().expect("Window is None");
                    #[cfg(target_arch = "wasm32")]
                    {
                        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                        use winit::platform::web::WindowExtWebSys;

                        web_sys::window()
                            .and_then(|win| win.document())
                            .and_then(|doc| doc.body())
                            .and_then(|body| {
                                body.remove_child(&body.last_element_child().unwrap())
                                    .unwrap();
                                body.append_child(&web_sys::Element::from(window.canvas()))
                                    .ok()
                            })
                            .expect("Couldn't append canvas to document body");
                    }
                    let setup = pollster::block_on(Setup::new(
                        window,
                        Mat4::IDENTITY,
                        &vertices,
                        &indices,
                    ));
                    let Setup {
                        redraw: some_redraw,
                        adapter: _,
                        instance: _,
                        pipeline_layout: _,
                        shader: _,
                    } = setup;
                    redraw = Some(some_redraw);

                    let _win_size = window.inner_size();
                }
                _ => {}
            },
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
                view_model.on_event(&svg_set, event);
            }
            Event::RedrawRequested(_) => {
                if let (Some(redraw), Some(window)) = (redraw.as_mut(), window.as_mut()) {
                    if let Some(mut texture) = view_model.into_bytes() {
                        texture.resize(8192 * 16, 0);
                        Setup::redraw(redraw, &texture[..]);
                        window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    });
}
