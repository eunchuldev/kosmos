pub use kosmos_tile_shared::{Terrian, Tile, TileConstants};
use async_lock::Semaphore;
use std::sync::Arc;
use futures_lite::future;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct TilemapGpuIterator {
    device: Arc<wgpu::Device>,
    queue: wgpu::Queue,
    compute_pipeline: wgpu::ComputePipeline,
    bind_group_0_to_1: wgpu::BindGroup,
    bind_group_1_to_0: wgpu::BindGroup,
    storage_buffer_0: wgpu::Buffer,
    storage_buffer_1: wgpu::Buffer,
    staging_buffer: wgpu::Buffer,

    buffer_size: u64,
    window_len: usize,
    window_width: usize,
    frame_counter: Arc<AtomicUsize>,

    queue_lock: Arc<Semaphore>,
}

impl TilemapGpuIterator {
    pub fn new(window_width: usize) -> Self {
        assert!((window_width * std::mem::size_of::<Tile>()) % 8 == 0);
        let window_len = window_width.pow(3);
        let buffer_size = (window_len * std::mem::size_of::<Tile>()) as u64;

        let (device, queue) = future::block_on(async {
            wgpu::Instance::default()
                .request_adapter(&wgpu::RequestAdapterOptions::default())
                /*.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: None,
                })*/
                .await
                .expect("Failed to find an appropriate adapter")
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        features: wgpu::Features::SPIRV_SHADER_PASSTHROUGH
                        | wgpu::Features::PUSH_CONSTANTS,
                        limits: wgpu::Limits {
                            max_push_constant_size: 128,
                            ..Default::default()
                        },
                    },
                    None,
                ).await
                .expect("Failed to create device")
        });

        let shader_bytes: &[u8] = include_bytes!(env!("kosmos_tile_kernel.spv"));
        let spirv = std::borrow::Cow::Owned(wgpu::util::make_spirv_raw(shader_bytes).into_owned());
        let shader_binary = wgpu::ShaderModuleDescriptorSpirV {
            label: None,
            source: spirv,
        };

        let module = unsafe { device.create_shader_module_spirv(&shader_binary) };

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,//Some(NonZeroU64::new(1).unwrap()),
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,//Some(NonZeroU64::new(1).unwrap()),
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                    },
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: 0..std::mem::size_of::<TileConstants>() as u32,
            }],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: "tick",
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let storage_buffer_0 = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tile Tick Buffer 0"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let storage_buffer_1 = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tile Tick Buffer 1"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });


        let bind_group_0_to_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Tile Tick Bindgroup 0 To 1"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer_0.as_entire_binding(),
            }, wgpu::BindGroupEntry {
                binding: 1,
                resource: storage_buffer_1.as_entire_binding(),
            }],
        });

        let bind_group_1_to_0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Tile Tick Bindgroup 1 To 0"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer_1.as_entire_binding(),
            }, wgpu::BindGroupEntry {
                binding: 1,
                resource: storage_buffer_0.as_entire_binding(),
            }],
        });

        Self {
            device: Arc::new(device),
            queue,
            compute_pipeline,
            storage_buffer_0,
            storage_buffer_1,
            bind_group_0_to_1,
            bind_group_1_to_0,
            buffer_size,
            staging_buffer,
            window_len,
            window_width,
            queue_lock: Arc::new(Semaphore::new(1)),
            frame_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn tick(&self) {
        let frame_number = self.frame_counter.fetch_add(1, Ordering::SeqCst) as u32;
        let bind_group = if frame_number % 2 == 0 {
            &self.bind_group_0_to_1
        } else {
            &self.bind_group_1_to_0
        };

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
            });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_push_constants(
                0,
                bytemuck::bytes_of(&TileConstants {
                    width: self.window_width as u32,
                    height: self.window_width as u32,
                    depth: self.window_width as u32,
                    frame_number,
                }),
            );
            cpass.set_bind_group(0, bind_group, &[]);
            let x = self.window_width as u32 / 4;
            let y = self.window_width as u32 / 4;
            let z = self.window_width as u32 / 4;
            cpass.dispatch_workgroups(x, y, z);
        }

        /*let guard = self.queue_lock.acquire_arc().await;
        self.queue.on_submitted_work_done(move || {
            drop(guard);
        });*/
        self.queue.submit(Some(encoder.finish()));
    }
    pub fn poll(&self) -> bool {
        self.device.poll(wgpu::Maintain::Poll)
    }
    pub fn wait(&self) {
        self.device.poll(wgpu::Maintain::Wait);
    }
    pub fn upload(&self, window: &[Tile]) -> anyhow::Result<()> {
        let input = if self.frame_counter.load(Ordering::SeqCst) % 2 == 0 {
            &self.storage_buffer_0
        } else {
            &self.storage_buffer_1
        };

        self.queue.write_buffer(&input, 0, bytemuck::cast_slice(&window[0..self.window_len]));
        Ok(())
    }
    pub async fn download(&self) -> anyhow::Result<Vec<Tile>> {
        let (s, r) = async_channel::bounded(1);

        let output = if self.frame_counter.load(Ordering::SeqCst) % 2 == 0 {
            &self.storage_buffer_1
        } else {
            &self.storage_buffer_0
        };

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(
            &output,
            0,
            &self.staging_buffer,
            0,
            self.buffer_size,
        );
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = self.staging_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| { s.try_send(v).unwrap(); });

        let device = self.device.clone();
        blocking::unblock(move || device.poll(wgpu::Maintain::Wait)).await;

        r.recv().await??;
        let data = buffer_slice.get_mapped_range();
        let window: Vec<Tile> = bytemuck::checked::cast_slice::<u8, Tile>(&data).to_owned();
        drop(data);
        self.staging_buffer.unmap();
        Ok(window)
    }
}


pub struct Tilemap {
    iterator: TilemapGpuIterator,

    window: Vec<Tile>,
    window_width: usize,
    chunk_width: usize,
    chunk_radius: usize,
}

impl Tilemap {
    pub fn new(chunk_width: usize, chunk_radius: usize) -> Self {
        let window_width = chunk_width * (chunk_radius*2 + 1);
        let iterator = TilemapGpuIterator::new(window_width);
        let window = vec![Tile::default(); window_width.pow(3)];

        Self {
            chunk_width,
            chunk_radius,
            window,
            window_width,
            iterator,
        }
    }

    pub async fn tick(&mut self) -> anyhow::Result<()> {
        self.iterator.upload(&self.window)?;
        self.iterator.tick();
        self.window = self.iterator.download().await?;
        Ok(())
    }

    pub async fn tick_without_sync(&self) {
        self.iterator.tick();
    }
}

