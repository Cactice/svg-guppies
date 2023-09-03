use crate::{
    primitives::{Indices, Vertex, Vertices},
    GpuRedraw,
};
use glam::Mat4;
use std::borrow::Cow;
use wgpu::{
    util::{make_spirv, DeviceExt},
    BindGroup, Buffer, CommandEncoder, Device, Extent3d, PipelineLayout, RenderPipeline, Surface,
    SurfaceConfiguration, SurfaceTexture, Texture, TextureFormat, TextureView,
};
use winit::{dpi::PhysicalSize, window::Window};

const TRANSFORM_TEXTURE_SIZE: Extent3d = Extent3d {
    width: 4 * 2048,
    height: 1,
    depth_or_array_layers: 1,
};

#[derive(Debug)]
pub struct Redraw {
    pub transform: Mat4,
    pub render_pipeline: RenderPipeline,
    pub bind_group: BindGroup,
    pub uniform_buffer: Buffer,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub transform_texture: Texture,
    pub pipeline_layout: PipelineLayout,
}

pub struct RedrawMachine {
    pub queue: wgpu::Queue,
    pub device: Device,
    pub surface: Surface,
    pub surface_format: TextureFormat,
    pub config: SurfaceConfiguration,
}
pub struct Reframe {
    pub view: TextureView,
    pub frame: SurfaceTexture,
    pub encoder: CommandEncoder,
}
impl RedrawMachine {
    pub fn redraw(&self, gpu_redraws: &mut [GpuRedraw], redraws: &[Redraw], reframe: &mut Reframe) {
        let Reframe {
            view,
            frame,
            encoder,
        } = reframe;
        let RedrawMachine {
            queue,
            device,
            surface,
            surface_format,
            config,
        } = self;
        let msaa_texture = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Multisampled frame descriptor"),
                size: wgpu::Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format: config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: Default::default(),
            })
            .create_view(&wgpu::TextureViewDescriptor::default());
        let x = redraws
            .iter()
            .zip(gpu_redraws.iter_mut())
            .for_each(|(redraw, gpu_redraw)| {
                gpu_redraw.texture.resize(8192 * 16, 0);
                let Redraw {
                    transform,
                    render_pipeline,
                    bind_group,
                    uniform_buffer,
                    transform_texture,
                    ..
                } = redraw;
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&gpu_redraw.triangles.indices),
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                });
                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("SVG-GUI Vertex Buffer"),
                    contents: (bytemuck::cast_slice(&gpu_redraw.triangles.vertices)),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &msaa_texture,
                            resolve_target: Some(view),
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(render_pipeline);
                    rpass.set_bind_group(0, bind_group, &[]);
                    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    rpass.draw_indexed(0..(gpu_redraw.triangles.indices.len() as u32), 0, 0..1);
                }
                queue.write_buffer(
                    uniform_buffer,
                    0,
                    bytemuck::cast_slice(&[Uniform {
                        transform: *transform,
                    }]),
                );
                queue.write_texture(
                    transform_texture.as_image_copy(),
                    &gpu_redraw.texture,
                    wgpu::ImageDataLayout::default(),
                    TRANSFORM_TEXTURE_SIZE,
                );
            });
    }
    pub fn get_frame(&self) -> Reframe {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        Reframe {
            view,
            frame,
            encoder,
        }
    }

    pub fn submit(&self, reframe: Reframe) {
        self.queue.submit(Some(reframe.encoder.finish()));
        reframe.frame.present();
    }
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        // Reconfigure the surface with the new size
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = unsafe {
            instance
                .create_surface(&window)
                .expect("Surface creation failed")
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats.first().unwrap().clone();
        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("SVG-GUI DeviceDescriptor"),
                    features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the surface.
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Default::default(),
        };
        RedrawMachine {
            device,
            surface,
            queue,
            config,
            surface_format,
        }
    }
}

const SAMPLE_COUNT: u32 = 4;

fn get_uniform_buffer(
    device: &Device,
    contents: &[u8],
) -> (
    wgpu::Buffer,
    wgpu::BindGroup,
    wgpu::BindGroupLayout,
    wgpu::Texture,
) {
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let transform_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("transform texture"),
        size: TRANSFORM_TEXTURE_SIZE,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D1,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: Default::default(),
    });
    let uniform_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D1,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
            label: Some("uniform_bind_group_layout"),
        });

    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(
                    &transform_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
        ],
        label: Some("uniform_bind_group"),
    });
    (
        uniform_buffer,
        uniform_bind_group,
        uniform_bind_group_layout,
        transform_texture,
    )
}

impl Redraw {
    pub fn update_shader(&mut self, spirv_shader: &Vec<u8>, redraw_machine: &RedrawMachine) {
        let RedrawMachine {
            device,
            surface_format,
            ..
        } = redraw_machine;
        let default_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });
        let custom_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: make_spirv(&spirv_shader),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&self.pipeline_layout),
            vertex: wgpu::VertexState {
                module: &default_shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Uint32, 2 => Float32x4],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &custom_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: *surface_format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: SAMPLE_COUNT,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        self.render_pipeline = render_pipeline;
    }

    pub fn new(
        redraw_machine: &RedrawMachine,
        vertices: &Vertices,
        indices: &Indices,
        index: usize,
    ) -> Self {
        let RedrawMachine {
            queue,
            device,
            surface,
            config,
            surface_format,
        } = redraw_machine;
        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let (uniform_buffer, uniform_bind_group, uniform_bind_group_layout, transform_texture) =
            get_uniform_buffer(
                &device,
                bytemuck::cast_slice(&[Uniform {
                    transform: Mat4::IDENTITY,
                }]),
            );
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&uniform_bind_group_layout],
            ..Default::default()
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Uint32, 2 => Float32x4],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: *surface_format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: SAMPLE_COUNT,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

        surface.configure(&device, &config);
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SVG-GUI Vertex Buffer"),
            contents: (bytemuck::cast_slice(vertices)),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        Redraw {
            render_pipeline,
            bind_group: uniform_bind_group,
            uniform_buffer,
            vertex_buffer,
            index_buffer,
            transform: Mat4::IDENTITY,
            transform_texture,
            pipeline_layout,
        }
    }
}

// Default scene has all values set to zero
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform {
    pub transform: Mat4,
}
