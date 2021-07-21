use super::{Instance, Renderable};
use crate::voxel::{ChunkCoord, Mesh, MeshFace};
use erupt::vk;

struct VoxelMeshBuffer {
    vertex_buffer: vk::Buffer,
    allocation: vk_alloc::Allocation,
    length: u32,
    mat: uv::Mat4,
}

pub struct VoxelMeshManager<T: Instance> {
    instance: std::sync::Arc<T>,
    meshes: std::collections::HashMap<ChunkCoord, VoxelMeshBuffer>,
}

pub struct VoxelMeshRenderer<T: Instance> {
    instance: std::sync::Arc<T>,
    layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    extent: vk::Extent2D,
    v_fov: f32,
}

impl VoxelMeshBuffer {
    pub fn new(instance: &impl Instance, mesh: &Mesh) -> Self {
        let device = instance.device();
        let allocator = instance.allocator();

        let buffer_info = vk::BufferCreateInfoBuilder::new()
            .size((std::mem::size_of::<MeshFace>() * mesh.faces.len()) as vk::DeviceSize)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let vertex_buffer = unsafe { device.create_buffer(&buffer_info, None) }.unwrap();

        let mut allocation = allocator
            .allocate_memory_for_buffer(device, vertex_buffer, vk_alloc::MemoryLocation::CpuToGpu)
            .unwrap();

        unsafe {
            device.bind_buffer_memory(vertex_buffer, allocation.device_memory, allocation.offset)
        }
        .unwrap();

        let slice = allocation.mapped_slice_mut().unwrap().unwrap();

        unsafe {
            std::ptr::copy_nonoverlapping(
                mesh.faces.as_ptr(),
                slice.as_mut_ptr().cast(),
                mesh.faces.len(),
            )
        };

        Self {
            vertex_buffer,
            allocation,
            length: mesh.faces.len() as u32,
            mat: mesh.coord.mat(),
        }
    }

    pub fn destroy(&self, instance: &impl Instance) {
        let device = instance.device();
        let allocator = instance.allocator();
        unsafe {
            allocator.deallocate(device, &self.allocation).unwrap();
            device.destroy_buffer(Some(self.vertex_buffer), None);
        }
    }
}

impl<T: Instance> VoxelMeshManager<T> {
    pub fn new(instance: std::sync::Arc<T>) -> Self {
        Self {
            instance: instance.clone(),
            meshes: std::collections::HashMap::new(),
        }
    }

    fn meshes(&self) -> impl Iterator<Item = &VoxelMeshBuffer> {
        self.meshes.values()
    }

    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    pub fn upload_mesh(&mut self, mesh: &Mesh) {
        let new_mesh = VoxelMeshBuffer::new(self.instance.as_ref(), mesh);
        if let Some(old_mesh) = self.meshes.insert(mesh.coord, new_mesh) {
            old_mesh.destroy(self.instance.as_ref())
        }
    }
}

impl<T: Instance> Drop for VoxelMeshManager<T> {
    fn drop(&mut self) {
        for mesh in self.meshes.values() {
            mesh.destroy(self.instance.as_ref())
        }
    }
}

impl<T: Instance> VoxelMeshRenderer<T> {
    const VOXEL_VERT_SPV_BYTES: &'static [u8] = include_shader!("voxel.vert");
    const VOXEL_FRAG_SPV_BYTES: &'static [u8] = include_shader!("voxel.frag");

    pub fn new(instance: std::sync::Arc<T>, surface: &impl Renderable) -> Self {
        let device = instance.device();
        let render_info = surface.render_info();

        let vert_code = erupt::utils::decode_spv(Self::VOXEL_VERT_SPV_BYTES).unwrap();
        let vert_shader_module_create_info =
            vk::ShaderModuleCreateInfoBuilder::new().code(&vert_code);
        let vert_shader_module =
            unsafe { device.create_shader_module(&vert_shader_module_create_info, None) }.unwrap();

        let frag_code = erupt::utils::decode_spv(Self::VOXEL_FRAG_SPV_BYTES).unwrap();
        let frag_shader_module_create_info =
            vk::ShaderModuleCreateInfoBuilder::new().code(&frag_code);
        let frag_shader_module =
            unsafe { device.create_shader_module(&frag_shader_module_create_info, None) }.unwrap();

        let entry_point = std::ffi::CString::new("main").unwrap();

        let stages = [
            vk::PipelineShaderStageCreateInfoBuilder::new()
                .stage(vk::ShaderStageFlagBits::VERTEX)
                .module(vert_shader_module)
                .name(&entry_point),
            vk::PipelineShaderStageCreateInfoBuilder::new()
                .stage(vk::ShaderStageFlagBits::FRAGMENT)
                .module(frag_shader_module)
                .name(&entry_point),
        ];

        let vertex_binding = vk::VertexInputBindingDescriptionBuilder::new()
            .binding(0)
            .stride(8)
            .input_rate(vk::VertexInputRate::INSTANCE);
        let vertex_attributes = [vk::VertexInputAttributeDescriptionBuilder::new()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32_UINT)
            .offset(0)];
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfoBuilder::new()
            .vertex_binding_descriptions(std::slice::from_ref(&vertex_binding))
            .vertex_attribute_descriptions(&vertex_attributes);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfoBuilder::new()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::ViewportBuilder::new()
            .x(0.0)
            .y(0.0)
            .width(render_info.extent.width as f32)
            .height(render_info.extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);
        let scissor = vk::Rect2DBuilder::new()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(render_info.extent);
        let viewport_state = vk::PipelineViewportStateCreateInfoBuilder::new()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let rasterization_state = vk::PipelineRasterizationStateCreateInfoBuilder::new()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::LINE)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .line_width(1.0);

        let multisample_state = vk::PipelineMultisampleStateCreateInfoBuilder::new()
            .rasterization_samples(vk::SampleCountFlagBits::_1)
            .sample_shading_enable(false)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfoBuilder::new()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentStateBuilder::new()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::all());
        let color_blend_state = vk::PipelineColorBlendStateCreateInfoBuilder::new()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment))
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let push_constant_range = vk::PushConstantRangeBuilder::new()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<uv::Mat4>() as u32);

        let layout_create_info = vk::PipelineLayoutCreateInfoBuilder::new()
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));

        let layout = unsafe { device.create_pipeline_layout(&layout_create_info, None) }.unwrap();

        let pipeline_create_info = vk::GraphicsPipelineCreateInfoBuilder::new()
            .stages(&stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .layout(layout)
            .render_pass(render_info.render_pass)
            .subpass(0);

        let pipeline =
            unsafe { device.create_graphics_pipelines(None, &[pipeline_create_info], None) }
                .unwrap()[0];

        unsafe {
            device.destroy_shader_module(Some(vert_shader_module), None);
            device.destroy_shader_module(Some(frag_shader_module), None);
        }

        Self {
            instance,
            layout,
            pipeline,
            extent: render_info.extent,
            v_fov: 45.0,
        }
    }

    fn perspective_mat(&self) -> uv::Mat4 {
        uv::projection::perspective_infinite_z_vk(
            self.v_fov,
            self.extent.width as f32 / self.extent.height as f32,
            0.1,
        )
    }

    pub fn render(
        &mut self,
        command_buffer: vk::CommandBuffer,
        manager: &VoxelMeshManager<impl Instance>,
        camera: &crate::camera::Camera,
    ) {
        let device = self.instance.device();
        unsafe {
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );
        }
        let projection_mat = self.perspective_mat() * camera.look_mat();
        for mesh in manager.meshes() {
            let transform_mat = projection_mat * mesh.mat;
            unsafe {
                device.cmd_push_constants(
                    command_buffer,
                    self.layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    std::mem::size_of::<uv::Mat4>() as u32,
                    transform_mat.as_ptr().cast(),
                );
                device.cmd_bind_vertex_buffers(command_buffer, 0, &[mesh.vertex_buffer], &[0]);
                device.cmd_draw(command_buffer, 6, mesh.length, 0, 0)
            }
        }
    }

    pub fn rebuild(&mut self, surface: &impl Renderable) {
        let instance = self.instance.clone();
        unsafe {
            std::mem::drop(std::ptr::read(self));
            std::ptr::write(self, Self::new(instance, surface))
        }
    }
}

impl<T: Instance> Drop for VoxelMeshRenderer<T> {
    fn drop(&mut self) {
        let device = self.instance.device();
        unsafe {
            device.destroy_pipeline_layout(Some(self.layout), None);
            device.destroy_pipeline(Some(self.pipeline), None)
        }
    }
}
