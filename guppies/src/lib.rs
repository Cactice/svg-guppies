pub mod primitives;
mod setup;
pub use bytemuck;
use bytemuck::{Pod, Zeroable};
pub use glam;
use primitives::{Triangles, Vertex};
use setup::{Redraw, RedrawMachine};
use std::array;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;
pub use wgpu;
pub use winit;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder, WindowId};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

fn init_window(event_loop: &EventLoopWindowTarget<()>) -> winit::window::Window {
    let window = WindowBuilder::new()
        .with_title("SVG-GUI")
        .build(&event_loop)
        .unwrap();
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        use console_log::log;
        use winit::platform::web::WindowExtWebSys;
        console_log::init();

        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.remove_child(&body.last_element_child().unwrap())
                    .unwrap();
                body.append_child(&web_sys::Element::from(window.canvas().unwrap()))
                    .ok()
            })
            .expect("Couldn't append canvas to document body");
    }
    window
}

#[derive(Debug, Default)]
pub struct GpuRedraw<T: Pod + Zeroable + Debug + Clone + Default = Vertex> {
    texture: Vec<u8>,
    triangles: Triangles<T>,
    shader: Option<Vec<u32>>,
}

impl GpuRedraw {
    pub fn update_spirv_shader(&mut self, shader: Vec<u32>) {
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

pub struct Guppy<const COUNT: usize, Vert>
where
    Vert: Pod + Zeroable + Debug + Clone + Default,
{
    init: [GpuRedraw<Vert>; COUNT],
    functions: Vec<Box<dyn FnMut(&Event<()>, &mut [GpuRedraw<Vert>; COUNT])>>,
}

impl<const COUNT: usize, Vert: Pod + Zeroable + Debug + Clone + Default> Guppy<COUNT, Vert> {
    pub fn register<F: FnMut(&Event<()>, &mut [GpuRedraw<Vert>; COUNT]) + 'static>(
        &mut self,
        f: F,
    ) {
        self.functions.push(Box::new(f));
    }
    pub fn new(init: [GpuRedraw<Vert>; COUNT]) -> Self {
        Self {
            init,
            functions: Vec::default(),
        }
    }
    pub fn start(self) {
        render_loop(self.functions);
    }
}

pub fn render_loop<const COUNT: usize, Vert>(
    mut render_loop_fn: Vec<Box<dyn FnMut(&Event<()>, &mut [GpuRedraw<Vert>; COUNT])>>,
) where
    Vert: Pod + Zeroable + Debug + Clone + Default,
{
    let event_loop = EventLoop::new();

    // Type definition is required for android build
    let mut window: Option<Arc<Window>> = None;
    let mut gpu_redraw: Option<[GpuRedraw<Vert>; COUNT]> = None;
    let mut redraws: Option<[Redraw; COUNT]> = None;
    let mut redraw_machine: Option<RedrawMachine> = None;
    #[cfg(not(target_arch = "wasm32"))]
    let mut last_frame_inst = Instant::now();
    #[cfg(not(target_arch = "wasm32"))]
    let (mut frame_count, mut accum_time) = (0, 0.0);
    let _ = event_loop
        .expect("event loop initialization failed")
        .run(move |event, event_loop| {
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
                render_loop_fn.iter_mut().for_each(|func| {
                    func(&event, gpu_redraw);
                });
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
                Event::Resumed => {
                    init_window(event_loop);
                }
                #[cfg(not(target_os = "android"))]
                Event::NewEvents(start_cause) => match start_cause {
                    winit::event::StartCause::Init => {
                        let new_window = Arc::new(init_window(event_loop));
                        let new_redraw_machine =
                            pollster::block_on(RedrawMachine::new(new_window.clone()));
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

                        // Below is necessary when running on mobile...
                        if let Some(gpu_redraw) = gpu_redraw.as_mut() {
                            render_loop_fn.iter_mut().for_each(|func| {
                                let size = window.as_ref().unwrap().inner_size();
                                func(
                                    &Event::WindowEvent {
                                        window_id: unsafe { WindowId::dummy() },
                                        event: WindowEvent::Resized(size),
                                    },
                                    gpu_redraw,
                                );
                            });
                        }
                    }
                    _ => (),
                },
                Event::WindowEvent {
                    event: window_event,
                    ..
                } => match window_event {
                    WindowEvent::CloseRequested => {
                        event_loop.exit();
                    }
                    WindowEvent::Resized(p) => match redraw_machine.as_mut() {
                        Some(redraw_machine) => redraw_machine.resize(p),
                        _ => {}
                    },
                    WindowEvent::RedrawRequested => {
                        if let (
                            Some(window),
                            Some(gpu_redraw),
                            Some(redraws),
                            Some(redraw_machine),
                        ) = (
                            window.as_mut(),
                            gpu_redraw.as_mut(),
                            redraws.as_mut(),
                            redraw_machine.as_mut(),
                        ) {
                            let mut frame = redraw_machine.get_frame();
                            redraw_machine.redraw(gpu_redraw, redraws, &mut frame);
                            redraw_machine.submit(frame);
                            window.request_redraw();
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                accum_time += last_frame_inst.elapsed().as_secs_f32();
                                last_frame_inst = Instant::now();
                                frame_count += 1;
                                if frame_count == 100 {
                                    // println!(
                                    //     "Avg frame time {}ms",
                                    //     accum_time * 1000.0 / frame_count as f32
                                    // );
                                    accum_time = 0.0;
                                    frame_count = 0;
                                }
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        });
}
