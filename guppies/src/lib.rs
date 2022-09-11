pub mod callback;
pub mod primitives;
mod setup;
pub use glam;
use glam::{Mat4, Vec2};
use primitives::Triangles;
use setup::Redraw;
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

fn init(
    event_loop: &EventLoopWindowTarget<()>,
    triangles: &Triangles,
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
    *redraw = Some(pollster::block_on(Redraw::new(
        window,
        Mat4::IDENTITY,
        &triangles.vertices,
        &triangles.indices,
    )));
}

#[derive(Debug, Default)]
pub struct GpuRedraw {
    texture: Vec<u8>,
    triangles: Triangles,
}
impl GpuRedraw {
    pub fn update_texture(&mut self, textures: Vec<u8>) {
        self.texture = textures;
    }
    pub fn update_triangles(&mut self, triangles: Triangles, offset: usize) {
        let v_i = {
            if offset > 0 {
                self.triangles.indices[offset + 1] as usize
            } else {
                0 as usize
            }
        };
        self.triangles.indices.splice(offset.., triangles.indices);
        self.triangles.vertices.splice(v_i.., triangles.vertices);
    }
}

pub fn init_main_loop<F: FnMut(WindowEvent, &mut GpuRedraw) + 'static>(mut main_loop: F) {
    let event_loop = EventLoop::new();
    let mut redraw = None;
    // Type definition is required for android build
    let mut window: Option<Window> = None;
    let mut gpu_redraw = GpuRedraw::default();

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Poll;
        // FIXME: why do some OS not redraw automatically without explicit call
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
                    init(event_loop, &gpu_redraw.triangles, &mut redraw, &mut window);
                    let size = window.as_ref().unwrap().inner_size();
                    main_loop(WindowEvent::Resized(size), &mut gpu_redraw);
                }
                _ => (),
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(p) => {
                    if let Some(redraw) = redraw.as_mut() {
                        redraw.resize(p);
                    }
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                if let (Some(redraw), Some(window)) = (redraw.as_mut(), window.as_mut()) {
                    gpu_redraw.texture.resize(8192 * 16, 0);
                    redraw.redraw(
                        &gpu_redraw.texture[..],
                        &gpu_redraw.triangles.vertices,
                        &gpu_redraw.triangles.indices,
                    );
                    window.request_redraw();
                }
            }
            _ => {}
        }
    });
}
