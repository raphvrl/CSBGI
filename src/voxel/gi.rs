use anyhow::Result;

use crate::{
    debug::VoxelDebugMode,
    graphics::{
        GraphicsContext, VoxelGpu,
        bindings::GpuVoxelTraceSettings,
        pass::{
            debug::{VoxelDebugPass, VoxelDebugPassInput},
            voxel::{LightingInput, LightingPass, MipHierarchy, MipPass, VoxelSampling},
        },
    },
    voxel::{SparseVoxelStorage, VOXEL_SIZE, VoxelGrid, voxelize_scene},
};

use crate::assets::data::SceneData;
use std::time::Instant;

pub struct VoxelGi {
    #[allow(dead_code)]
    grid: VoxelGrid,
    #[allow(dead_code)]
    storage: SparseVoxelStorage,
    #[allow(dead_code)]
    gpu: VoxelGpu,
    mip_hierarchy: MipHierarchy,
    sampling: VoxelSampling,
    lighting_pass: LightingPass,
    mip_pass: MipPass,
    debug_pass: VoxelDebugPass,
    debug_enabled: bool,
    occupied_debug_enabled: bool,
    debug_mode: VoxelDebugMode,
    mip_debug_enabled: bool,
    mip_level: u32,
    radiance_dirty: bool,
}

impl VoxelGi {
    pub(crate) fn new(
        graphics: &GraphicsContext,
        frame_layout: &wgpu::BindGroupLayout,
        scene_data: &SceneData,
        startup: &Instant,
    ) -> Result<Self> {
        let grid = VoxelGrid {
            origin: glam::Vec3::splat(-16.0),
            voxel_size: VOXEL_SIZE,
            dimensions: glam::UVec3::splat(256),
        };
        let mut storage = SparseVoxelStorage::new();
        voxelize_scene(&grid, &mut storage, scene_data)?;

        tracing::info!(
            stage = "voxelization",
            elapsed_ms = startup.elapsed().as_millis(),
            bricks = storage.brick_count(),
        );

        let gpu = VoxelGpu::new(graphics, &storage, &grid)?;

        tracing::info!(
            stage = "voxel_gpu_upload",
            elapsed_ms = startup.elapsed().as_millis(),
            bricks = gpu.brick_count(),
            occupied_voxels = gpu.occupied_voxel_count(),
            visible_faces = gpu.visible_face_count(),
        );

        let mip_hierarchy = MipHierarchy::new(graphics, &grid);
        let sampling = VoxelSampling::new(graphics, &gpu, &mip_hierarchy);
        let lighting_pass = LightingPass::new(graphics, frame_layout, gpu.layout(), gpu.lighting_layout());
        let mip_pass = MipPass::new(graphics, gpu.layout(), &gpu, &mip_hierarchy);
        let debug_pass = VoxelDebugPass::new(
            graphics,
            frame_layout,
            gpu.layout(),
            gpu.radiance_layout(),
            &mip_hierarchy,
        );

        tracing::info!(stage = "voxel_passes", elapsed_ms = startup.elapsed().as_millis());

        Ok(Self {
            grid,
            storage,
            gpu,
            mip_hierarchy,
            sampling,
            lighting_pass,
            mip_pass,
            debug_pass,
            debug_enabled: false,
            occupied_debug_enabled: false,
            debug_mode: VoxelDebugMode::WhiteEdges,
            mip_debug_enabled: false,
            mip_level: 0,
            radiance_dirty: true,
        })
    }

    pub(crate) fn debug_settings_mut(
        &mut self,
    ) -> (
        &mut bool,
        &mut bool,
        &mut VoxelDebugMode,
        &mut bool,
        &mut u32,
        &mut GpuVoxelTraceSettings,
    ) {
        let Self {
            debug_enabled,
            occupied_debug_enabled,
            debug_mode,
            mip_debug_enabled,
            mip_level,
            sampling,
            ..
        } = self;

        (
            debug_enabled,
            occupied_debug_enabled,
            debug_mode,
            mip_debug_enabled,
            mip_level,
            sampling.trace_settings_mut(),
        )
    }

    pub(crate) fn upload_trace_settings(&self, queue: &wgpu::Queue) {
        self.sampling.upload_trace_settings(queue);
    }

    pub(crate) fn sampling_bind_group(&self) -> &wgpu::BindGroup {
        self.sampling.bind_group()
    }

    pub(crate) fn sampling_layout(&self) -> &wgpu::BindGroupLayout {
        self.sampling.layout()
    }

    pub(crate) fn mark_radiance_dirty(&mut self, light_changed: bool) {
        self.radiance_dirty |= light_changed;
    }

    fn mip_instance_count(&self) -> u32 {
        self.mip_hierarchy.level_count(self.mip_level)
    }

    pub(crate) fn uses_solid_debug(&self) -> bool {
        (self.occupied_debug_enabled
            && matches!(
                self.debug_mode,
                VoxelDebugMode::AlbedoFaces | VoxelDebugMode::NormalFaces | VoxelDebugMode::RadianceFaces
            ))
            || self.mip_debug_enabled
    }

    pub(crate) fn needs_shadow_pass(&self) -> bool {
        let radiance_debug = (self.occupied_debug_enabled
            && self.debug_mode == VoxelDebugMode::RadianceFaces)
            || self.mip_debug_enabled;

        !self.uses_solid_debug() || (radiance_debug && self.radiance_dirty)
    }

    pub(crate) fn record_lighting(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_bind_group: &wgpu::BindGroup,
    ) {
        if !self.radiance_dirty {
            return;
        }

        self.lighting_pass.record(
            encoder,
            LightingInput {
                frame_bind_group,
                voxel_bind_group: self.gpu.bind_group(),
                lighting_bind_group: self.gpu.lighting_bind_group(),
                occupied_voxel_count: self.gpu.occupied_voxel_count(),
            },
        );
        self.mip_pass.record(encoder);
        self.radiance_dirty = false;
    }

    fn debug_pass_input<'a>(
        gpu: &'a VoxelGpu,
        color_view: &'a wgpu::TextureView,
        resolve_view: &'a wgpu::TextureView,
        depth_view: &'a wgpu::TextureView,
        frame_bind_group: &'a wgpu::BindGroup,
    ) -> VoxelDebugPassInput<'a> {
        VoxelDebugPassInput {
            color_view,
            resolve_view,
            depth_view,
            frame_bind_group,
            voxel_bind_group: gpu.bind_group(),
            radiance_bind_group: gpu.radiance_bind_group(),
            brick_count: gpu.brick_count(),
            occupied_voxel_count: gpu.occupied_voxel_count(),
            visible_face_count: gpu.visible_face_count(),
        }
    }

    pub(crate) fn record_debug(
        &mut self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        resolve_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        frame_bind_group: &wgpu::BindGroup,
    ) {
        if self.debug_enabled {
            let input =
                Self::debug_pass_input(&self.gpu, color_view, resolve_view, depth_view, frame_bind_group);
            self.debug_pass.record(encoder, input);
        }

        if self.occupied_debug_enabled {
            let input =
                Self::debug_pass_input(&self.gpu, color_view, resolve_view, depth_view, frame_bind_group);

            match self.debug_mode {
                VoxelDebugMode::WhiteEdges => self.debug_pass.record_occupied_voxels(encoder, input),
                VoxelDebugMode::AlbedoFaces => self.debug_pass.record_occupied_voxel_faces(encoder, input),
                VoxelDebugMode::NormalFaces => self.debug_pass.record_occupied_voxel_normals(encoder, input),
                VoxelDebugMode::RadianceFaces => {
                    self.debug_pass.record_occupied_voxel_radiance(encoder, input)
                }
            }
        }

        if self.mip_debug_enabled {
            self.debug_pass.update_level(queue, self.mip_level);
            let input =
                Self::debug_pass_input(&self.gpu, color_view, resolve_view, depth_view, frame_bind_group);
            self.debug_pass
                .record_mips(encoder, input, self.mip_instance_count());
        }
    }
}
