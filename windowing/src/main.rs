mod setup;
use setup::Setup;

use tesselation::glam::{Mat4, Vec2, Vec4};
use tesselation::init;

use winit::dpi::PhysicalSize;
use winit::window::{self, WindowBuilder};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
fn get_scale(size: PhysicalSize<u32>, svg_scale: Vec2) -> Mat4 {
    let ratio = f32::min(1200. as f32, 1600. as f32) / f32::max(svg_scale.x, svg_scale.y);
    let scale = Mat4::from_scale(
        [
            2.0 * ratio / size.width as f32,
            -2.0 * ratio / size.height as f32,
            1.0,
        ]
        .into(),
    );
    scale
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let (svg_draw_primitives, (_translate, svg_scale)) = init();
    let win_size = window.inner_size();
    let mut translate = Mat4::from_translation([-1., 1.0, 0.0].into());
    let mut scale = get_scale(win_size, svg_scale);
    let mut transform: Mat4 = translate * scale;
    let Setup {
        instance,
        adapter,
        surface,
        device,
        queue,
        mut config,
        render_pipeline,
        shader,
        pipeline_layout,
        bind_group,
        buffer,
    } = Setup::new(&window, transform).await;
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
                    translate = translate
                        * Mat4::from_translation(
                            [-p.x as f32 * 1.5 / 1600., -p.y as f32 * 1.5 / 1600., 0.].into(),
                        );
                }
                winit::event::MouseScrollDelta::LineDelta(_, _) => todo!(),
            },
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                Setup::resize(size, &device, &surface, &mut config);
                scale = get_scale(size, svg_scale);
            }
            Event::RedrawRequested(_) => {
                let transform = translate * scale;
                Setup::redraw(
                    svg_draw_primitives.0.as_ref(),
                    svg_draw_primitives.1.as_ref(),
                    &transform,
                    &device,
                    &surface,
                    &render_pipeline,
                    &queue,
                    &config,
                    &bind_group,
                    &buffer,
                );
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

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("SVG-GUI")
        .build(&event_loop)
        .unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Temporarily avoid srgb formats for the surface on the web
        pollster::block_on(run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}
