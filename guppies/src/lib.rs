pub mod primitives;
mod setup;
pub use bytemuck;
use bytemuck::{Pod, Zeroable};
pub use glam;
use pollster;
use primitives::{Triangles, Vertex};
use setup::{Redraw, RedrawMachine};
use std::array;
use std::fmt::Debug;
use std::sync::Arc;

pub use wgpu;
pub use winit;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

pub fn exec_futures<T: std::future::Future<Output = ()> + 'static>(future: T) {
    #[cfg(not(target_arch = "wasm32"))]
    pollster::block_on(future);
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(future);
}
fn init_window(event_loop: &ActiveEventLoop) -> winit::window::Window {
    let attributes = Window::default_attributes().with_title("SVG-GUI");
    let window = event_loop.create_window(attributes).unwrap();
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
        exec_futures(render_loop(self.functions));
    }
}

use winit::application::ApplicationHandler;

pub struct GuppyApp<const COUNT: usize, Vert>
where
    Vert: Pod + Zeroable + Debug + Clone + Default,
{
    redraw_machine: Option<RedrawMachine<'static>>,
    window: Option<Arc<Window>>,
    gpu_redraw: Option<[GpuRedraw<Vert>; COUNT]>,
    redraws: Option<[Redraw; COUNT]>,
    functions: Vec<Box<dyn FnMut(&Event<()>, &mut [GpuRedraw<Vert>; COUNT])>>,
}

impl<const COUNT: usize, Vert> ApplicationHandler for GuppyApp<COUNT, Vert>
where
    Vert: Pod + Zeroable + Debug + Clone + Default,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win = init_window(event_loop);
            let win = Arc::new(win);
            self.window = Some(win.clone());

            #[cfg(not(target_arch = "wasm32"))]
            {
                // Unsafe hack to satisfy RedrawMachine<'static>
                // We ensure 'win' lives as long as 'redraw_machine' because they are both in GuppyApp
                // and 'redraw_machine' is dropped before 'window' (field order matters in Rust drop,
                // but we can also manually drop if needed, though simpler here since we leak slightly or trust the struct)
                // Actually to make it truly 'static, we might need to leak the window or use unsafe change of lifetime.
                let _win_ref: &'static Window = unsafe { std::mem::transmute(win.as_ref()) };

                // We reconstruct RedrawMachine to take a static reference if possible,
                // but RedrawMachine::new takes Arc<Window>.
                // RedrawMachine definition: pub struct RedrawMachine<'a> { surface: Surface<'a> ... }
                // setup::new takes (window: Arc<Window>) and does create_surface(window).
                // To get Surface<'static>, we generally need the target to be static.

                // Let's rely on the fact that we can cast the lifetime of RedrawMachine if we are careful.
                let machine = pollster::block_on(RedrawMachine::new(win.clone()));
                self.redraw_machine = Some(unsafe { std::mem::transmute(machine) });
                let machine_ref = self.redraw_machine.as_ref().unwrap();
                self.redraws = Some(array::from_fn(|_| Redraw::new(machine_ref)));
                self.gpu_redraw = Some([(); COUNT].map(|_| GpuRedraw::default()));

                // Initial resize trigger
                let size = win.inner_size();
                let synthetic_event = Event::WindowEvent {
                    window_id: WindowId::dummy(),
                    event: WindowEvent::Resized(size),
                };
                self.functions.iter_mut().for_each(|func| {
                    func(&synthetic_event, self.gpu_redraw.as_mut().unwrap());
                });
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Reconstruct the Event::WindowEvent for user callbacks
        // Note: We can only provide a reference to our synthetic event.
        let synthetic_event = Event::WindowEvent {
            window_id,
            event: event.clone(),
        };

        if let (Some(ref mut gpu_redraw), Some(ref mut machine)) =
            (self.gpu_redraw.as_mut(), self.redraw_machine.as_mut())
        {
            self.functions.iter_mut().for_each(|func| {
                func(&synthetic_event, gpu_redraw);
            });

            if let Some(ref mut redraws) = self.redraws {
                redraws
                    .iter_mut()
                    .zip(gpu_redraw.iter_mut())
                    .for_each(|(redraw, new_redraw)| {
                        if let Some(shader) = new_redraw.shader.take() {
                            redraw.update_shader(&shader, machine);
                        }
                    });
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(p) => {
                if let Some(machine) = self.redraw_machine.as_mut() {
                    machine.resize(p);
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(gpu_redraw), Some(redraws), Some(machine), Some(window)) = (
                    self.gpu_redraw.as_mut(),
                    self.redraws.as_mut(),
                    self.redraw_machine.as_mut(),
                    self.window.as_ref(),
                ) {
                    let mut frame = machine.get_frame();
                    machine.redraw(gpu_redraw, redraws, &mut frame);
                    machine.submit(frame);
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

pub async fn render_loop<const COUNT: usize, Vert>(
    render_loop_fn: Vec<Box<dyn FnMut(&Event<()>, &mut [GpuRedraw<Vert>; COUNT])>>,
) where
    Vert: Pod + Zeroable + Debug + Clone + Default,
{
    let event_loop = EventLoop::new().unwrap();

    let mut app = GuppyApp {
        window: None,
        redraw_machine: None,
        gpu_redraw: None,
        redraws: None,
        functions: render_loop_fn,
    };

    let _ = event_loop.run_app(&mut app);
}
