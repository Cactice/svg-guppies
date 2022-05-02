mod setup;
use setup::Setup;

use usvg_layout::glam::{Mat3, Vec2};
use usvg_layout::init;
use usvg_layout::iterator::Vertex;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

async fn run(event_loop: EventLoop<()>, window: Window) {
    let (svg_draw_primitives, (_translate, scale)) = init();
    let transform: Mat3 = Mat3::from_scale(Vec2::new(1.0 / (scale.x), 1.0 / (scale.y)));
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
    } = Setup::new(&window, transform).await;
    let _vertices = vec![
        Vertex {
            position: [-1.0, 1.0, 0.0],
            color: [1.0, 0.0, 0.0],
            ..Default::default()
        },
        Vertex {
            position: [1.0, -1.0, 0.0],
            color: [0.0, 1.0, 0.0],
            ..Default::default()
        },
        Vertex {
            position: [1.0, 1.0, 0.0],
            color: [0.0, 0.0, 1.0],
            ..Default::default()
        },
    ];

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter, &shader, &pipeline_layout);

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => Setup::resize(size, &device, &surface, &mut config),
            Event::RedrawRequested(_) => {
                Setup::redraw(
                    svg_draw_primitives.0.as_ref(),
                    &device,
                    &surface,
                    &render_pipeline,
                    &queue,
                    svg_draw_primitives.1.as_ref(),
                    &config,
                    &bind_group,
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
    let window = winit::window::Window::new(&event_loop).unwrap();
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
