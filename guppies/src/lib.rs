pub mod primitives;
mod setup;
use std::array;

pub use glam;
use primitives::Triangles;
use setup::{Redraw, RedrawMachine};
pub use wgpu;
pub use winit;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder, WindowId};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

fn init_window(event_loop: &EventLoopWindowTarget<()>) -> winit::window::Window {
    let window = WindowBuilder::new()
        .with_title("SVG-GUI")
        .build(&event_loop)
        .unwrap();
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
    window
}

#[derive(Debug, Default)]
pub struct GpuRedraw {
    texture: Vec<u8>,
    triangles: Triangles,
    shader: Option<Vec<u8>>,
}

impl GpuRedraw {
    pub fn update_spirv_shader(&mut self, shader: Vec<u8>) {
        self.shader = Some(shader);
    }
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

pub fn render_loop<const COUNT: usize, F: FnMut(&Event<()>, &mut [GpuRedraw; COUNT]) + 'static>(
    mut render_loop: F,
) {
    let event_loop = EventLoop::new();

    // Type definition is required for android build
    let mut window: Option<Window> = None;
    let mut gpu_redraw: Option<[GpuRedraw; COUNT]> = None;
    let mut redraws: Option<[Redraw; COUNT]> = None;
    let mut redraw_machine: Option<RedrawMachine> = None;

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Poll;
        // FIXME: why do some OS not redraw automatically without explicit call
        #[cfg(any(target_os = "ios", target_os = "android"))]
        if let Some(window) = window.as_mut() {
            window.request_redraw();
        }
        if let (Some(ref mut gpu_redraw), Some(redraws), Some(redraw_machine)) = (
            gpu_redraw.as_mut(),
            redraws.as_mut(),
            redraw_machine.as_ref(),
        ) {
            render_loop(&event, gpu_redraw);
            redraws
                .iter_mut()
                .zip(gpu_redraw.iter_mut())
                .for_each(|(redraw, new_redraw)| {
                    if let Some(shader) = new_redraw.shader.take() {
                        redraw.update_shader(&shader, redraw_machine);
                    }
                });
        }
        match event {
            #[cfg(target_os = "android")]
            Event::Resumed => init_window(event_loop, &draw_primitive, &mut redraws, &mut window),
            #[cfg(not(target_os = "android"))]
            Event::NewEvents(start_cause) => match start_cause {
                winit::event::StartCause::Init => {
                    let new_window = init_window(event_loop);
                    let new_redraw_machine = pollster::block_on(RedrawMachine::new(&new_window));
                    redraws = Some(array::from_fn(|i| {
                        Redraw::new(
                            &new_redraw_machine,
                            &Default::default(),
                            &Default::default(),
                            i,
                        )
                    }));
                    redraw_machine = Some(new_redraw_machine);
                    gpu_redraw = Some([(); COUNT].map(|_| GpuRedraw::default()));
                    window = Some(new_window);

                    // I think below is necessary when running on mobile...
                    // I forgot and don't want to test now.
                    let size = window.as_ref().unwrap().inner_size();
                    if let Some(gpu_redraw) = gpu_redraw.as_mut() {
                        render_loop(
                            &Event::WindowEvent {
                                window_id: unsafe { WindowId::dummy() },
                                event: WindowEvent::Resized(size),
                            },
                            gpu_redraw,
                        );
                    }
                }
                _ => (),
            },
            Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(p) => match redraw_machine.as_mut() {
                    Some(redraw_machine) => redraw_machine.resize(p),
                    _ => {}
                },
                _ => {}
            },
            Event::RedrawRequested(_) => {
                if let (Some(window), Some(gpu_redraw), Some(redraws), Some(redraw_machine)) = (
                    window.as_mut(),
                    gpu_redraw.as_mut(),
                    redraws.as_mut(),
                    redraw_machine.as_mut(),
                ) {
                    let mut frame = redraw_machine.get_frame();
                    redraw_machine.redraw(gpu_redraw, redraws, &mut frame);
                    redraw_machine.submit(frame);
                    window.request_redraw();
                }
            }
            _ => {}
        }
    });
}
