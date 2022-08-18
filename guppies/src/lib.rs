pub mod callback;
pub mod primitives;
mod setup;
pub use glam;
use glam::{Mat4, Vec2};
use primitives::DrawPrimitives;
use setup::Setup;
pub use winit;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
pub fn get_scale(size: PhysicalSize<u32>, svg_scale: Vec2) -> Mat4 {
    let ratio = f32::min(svg_scale.x, svg_scale.y) / f32::max(svg_scale.x, svg_scale.y);
    Mat4::from_scale(
        [
            2.0 * ratio / size.width as f32,
            -2.0 * ratio / size.height as f32,
            1.0,
        ]
        .into(),
    )
}

pub trait ViewModel: Send + Sync {
    fn on_redraw(&mut self) -> (Option<Vec<u8>>, Option<DrawPrimitives>);
    fn reset_mut_count(&mut self);
    fn on_event(&mut self, event: WindowEvent);
}

fn init(
    event_loop: &EventLoopWindowTarget<()>,
    draw_primitive: &DrawPrimitives,
    redraw: &mut Option<setup::Redraw>,
    window: &mut Option<winit::window::Window>,
) {
    *window = Some(
        WindowBuilder::new()
            .with_title("SVG-GUI")
            .build(&event_loop)
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
        &draw_primitive.0,
        &draw_primitive.1,
    ));
    let Setup {
        redraw: some_redraw,
        ..
    } = setup;
    *redraw = Some(some_redraw);
}

pub fn main<V: ViewModel + 'static>(mut view_model: V) {
    let event_loop = EventLoop::new();
    let draw_primitive = view_model
        .on_redraw()
        .1
        .expect("initial draw must not be none");
    let mut redraw = None;
    let mut window: Option<Window> = None;

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Poll;
        // FIXME: why does ios not redraw automatically without explicit call
        #[cfg(any(target_os = "ios", target_os = "android"))]
        if let Some(window) = window.as_mut() {
            window.request_redraw();
        }
        match event {
            #[cfg(target_os = "android")]
            Event::Resumed => init(event_loop, &draw_primitive, &mut redraw, &mut window),
            #[cfg(not(target_os = "android"))]
            Event::NewEvents(start_cause) => match start_cause {
                winit::event::StartCause::Init => {
                    init(event_loop, &draw_primitive, &mut redraw, &mut window);
                    let size = window.as_ref().unwrap().inner_size();
                    view_model.on_event(WindowEvent::Resized(size));
                }
                _ => (),
            },
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(p) => {
                        if let Some(redraw) = redraw.as_mut() {
                            Setup::resize(p, redraw);
                        }
                    }
                    _ => {}
                }
                view_model.on_event(event);
            }
            Event::RedrawRequested(_) => {
                if let (Some(redraw), Some(window)) = (redraw.as_mut(), window.as_mut()) {
                    if let (Some(mut texture), Some((vertices, indices))) = view_model.on_redraw() {
                        texture.resize(8192 * 16, 0);
                        Setup::redraw(redraw, &texture[..], &vertices, &indices);
                    }
                }
            }
            _ => {}
        }
    });
}
