mod setup;
use setup::Setup;
pub use tesselation;
use tesselation::glam::{Mat4, Vec2};
use tesselation::{init, Callback};
use winit::dpi::PhysicalSize;
use winit::window::WindowBuilder;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

fn get_scale(size: PhysicalSize<u32>, svg_scale: Vec2) -> Mat4 {
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

async fn run(event_loop: EventLoop<()>, window: Window, callback: Callback<'_>) {
    let ((vertices, indices), (_translate, svg_scale)) = init(callback);
    let win_size = window.inner_size();
    let mut translate = Mat4::from_translation([-1., 1.0, 0.0].into());
    let mut scale = get_scale(win_size, svg_scale);
    let transform: Mat4 = translate * scale;
    let Setup {
        mut redraw,
        adapter,
        instance,
        pipeline_layout,
        shader,
    } = Setup::new(&window, transform, &vertices, &indices).await;
    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter, &shader, &pipeline_layout);

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => match delta {
                winit::event::MouseScrollDelta::PixelDelta(p) => {
                    translate *= Mat4::from_translation(
                        [-p.x as f32 * 1.5 / 1600., -p.y as f32 * 1.5 / 1600., 0.].into(),
                    );
                }
                winit::event::MouseScrollDelta::LineDelta(_, _) => todo!(),
            },
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                Setup::resize(size, &redraw.device, &redraw.surface, &mut redraw.config);
                scale = get_scale(size, svg_scale);
            }
            Event::RedrawRequested(_) => {
                redraw.transform = translate * scale;
                Setup::redraw(&redraw);
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

pub fn main(callback: Callback) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("SVG-GUI")
        .build(&event_loop)
        .unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Temporarily avoid srgb formats for the surface on the web
        pollster::block_on(run(event_loop, window, callback));
    }
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
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window, callback));
    }
}
