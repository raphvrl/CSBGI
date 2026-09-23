use std::sync::mpsc::{self, Receiver, Sender};

use crate::{debug::ProfileStage, graphics::GraphicsContext};

const SLOT_COUNT: usize = 3;
const QUERY_COUNT: u32 = (ProfileStage::COUNT as u32 - ProfileStage::RENDER_START as u32) * 2;
const RESOLVED_SIZE: u64 = QUERY_COUNT as u64 * wgpu::QUERY_SIZE as u64;

struct ReadbackSlot {
    resolve: wgpu::Buffer,
    readback: wgpu::Buffer,
    busy: bool,
}

pub struct GpuProfiler {
    query_set: Option<wgpu::QuerySet>,
    slots: Vec<ReadbackSlot>,
    timestamp_period: f32,
    active_slot: Option<usize>,
    sender: Sender<(usize, bool)>,
    receiver: Receiver<(usize, bool)>,
    latest: Option<[f32; ProfileStage::COUNT]>,
}

impl GpuProfiler {
    pub fn new(graphics: &GraphicsContext) -> Self {
        let (sender, receiver) = mpsc::channel();
        if !graphics.timestamp_queries_supported() {
            return Self {
                query_set: None,
                slots: Vec::new(),
                timestamp_period: 0.0,
                active_slot: None,
                sender,
                receiver,
                latest: None,
            };
        }

        let device = graphics.device();
        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("Profiler timestamp queries"),
            ty: wgpu::QueryType::Timestamp,
            count: QUERY_COUNT,
        });
        let slots = (0..SLOT_COUNT)
            .map(|_| ReadbackSlot {
                resolve: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Profiler query resolve buffer"),
                    size: RESOLVED_SIZE,
                    usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                }),
                readback: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Profiler query readback buffer"),
                    size: RESOLVED_SIZE,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }),
                busy: false,
            })
            .collect();

        Self {
            query_set: Some(query_set),
            slots,
            timestamp_period: graphics.queue().get_timestamp_period(),
            active_slot: None,
            sender,
            receiver,
            latest: None,
        }
    }

    pub fn supported(&self) -> bool {
        self.query_set.is_some()
    }

    pub fn poll(&mut self, device: &wgpu::Device) {
        let _ = device.poll(wgpu::PollType::Poll);
        while let Ok((slot_index, mapped)) = self.receiver.try_recv() {
            let slot = &mut self.slots[slot_index];
            if mapped {
                if let Ok(view) = slot.readback.slice(..).get_mapped_range() {
                    let timestamps: Vec<u64> = view
                        .chunks_exact(std::mem::size_of::<u64>())
                        .map(|bytes| {
                            u64::from_ne_bytes([
                                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
                                bytes[6], bytes[7],
                            ])
                        })
                        .collect();
                    let mut values_ms = [0.0; ProfileStage::COUNT];
                    for stage in ProfileStage::ALL {
                        if (stage as usize) < ProfileStage::RENDER_START as usize {
                            continue;
                        }
                        let pair = (stage as usize - ProfileStage::RENDER_START as usize) * 2;
                        let elapsed = timestamps[pair + 1].wrapping_sub(timestamps[pair]);
                        values_ms[stage as usize] =
                            (elapsed as f64 * self.timestamp_period as f64 / 1_000_000.0) as f32;
                    }
                    drop(view);
                    self.latest = Some(values_ms);
                }
                slot.readback.unmap();
            }
            slot.busy = false;
        }
    }

    pub fn take_latest(&mut self) -> Option<[f32; ProfileStage::COUNT]> {
        self.latest.take()
    }

    pub fn begin_frame(&mut self) {
        self.active_slot = self.slots.iter().position(|slot| !slot.busy);
    }

    pub fn write_start(&self, encoder: &mut wgpu::CommandEncoder, stage: ProfileStage) {
        if (stage as usize) < ProfileStage::RENDER_START as usize {
            return;
        }
        if let (Some(query_set), Some(_)) = (&self.query_set, self.active_slot) {
            let query_index = (stage as u32 - ProfileStage::RENDER_START as u32) * 2;
            encoder.write_timestamp(query_set, query_index);
        }
    }

    pub fn write_end(&self, encoder: &mut wgpu::CommandEncoder, stage: ProfileStage) {
        if (stage as usize) < ProfileStage::RENDER_START as usize {
            return;
        }
        if let (Some(query_set), Some(_)) = (&self.query_set, self.active_slot) {
            let query_index = (stage as u32 - ProfileStage::RENDER_START as u32) * 2 + 1;
            encoder.write_timestamp(query_set, query_index);
        }
    }

    pub fn resolve(&mut self, encoder: &mut wgpu::CommandEncoder) -> Option<usize> {
        let slot_index = self.active_slot.take()?;
        let query_set = self.query_set.as_ref()?;
        let slot = &mut self.slots[slot_index];
        encoder.resolve_query_set(query_set, 0..QUERY_COUNT, &slot.resolve, 0);
        encoder.copy_buffer_to_buffer(&slot.resolve, 0, &slot.readback, 0, RESOLVED_SIZE);
        slot.busy = true;
        Some(slot_index)
    }

    pub fn submitted(&self, slot_index: Option<usize>) {
        let Some(slot_index) = slot_index else {
            return;
        };
        let sender = self.sender.clone();
        self.slots[slot_index]
            .readback
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send((slot_index, result.is_ok()));
            });
    }
}
