use super::{RenderInfo, WindowedInstance};
use crate::vk::Instance;
use erupt::vk;

pub trait Renderable {
    fn render_info(&self) -> RenderInfo;
}

const MAX_FRAMES_IN_FLIGHT: u32 = 2;

pub struct Swapchain {
    instance: std::sync::Arc<WindowedInstance>,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,

    swapchain: vk::SwapchainKHR,
    swapchain_framebuffers: Vec<SwapchainFramebuffer>,
    depth_image: vk::Image,
    depth_allocation: vk_alloc::Allocation,
    depth_view: vk::ImageView,

    command_pool: vk::CommandPool,
    sync_objects: Vec<RenderSyncObject>,
    current_frame: usize,
}

#[derive(Debug, Default, Copy, Clone)]
struct SwapchainFramebuffer {
    view: vk::ImageView,
    framebuffer: vk::Framebuffer,
    fence: vk::Fence,
}

#[derive(Debug, Default, Copy, Clone)]
struct RenderSyncObject {
    in_flight: vk::Fence,
    image_available: vk::Semaphore,
    render_finished: vk::Semaphore,
    command_buffer: vk::CommandBuffer,
}

impl Swapchain {
    pub fn new(instance: std::sync::Arc<WindowedInstance>, size: (u32, u32)) -> Self {
        let device = instance.device();
        let allocator = instance.allocator();
        let swapchain_info = instance.swapchain_info(size);
        let (graphics, present) = (instance.graphics_queue(), instance.present_queue());

        let attachments = [
            vk::AttachmentDescriptionBuilder::new()
                .format(swapchain_info.surface_format.format)
                .samples(vk::SampleCountFlagBits::_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR),
            vk::AttachmentDescriptionBuilder::new()
                .format(vk::Format::D32_SFLOAT)
                .samples(vk::SampleCountFlagBits::_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
        ];
        let depth_stencil_attachment = vk::AttachmentReferenceBuilder::new()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        let color_attachment = vk::AttachmentReferenceBuilder::new()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        let subpass = vk::SubpassDescriptionBuilder::new()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment))
            .depth_stencil_attachment(&depth_stencil_attachment);
        let dependency = vk::SubpassDependencyBuilder::new()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            );
        let render_pass_create_info = vk::RenderPassCreateInfoBuilder::new()
            .attachments(&attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));
        let render_pass =
            unsafe { device.create_render_pass(&render_pass_create_info, None) }.unwrap();

        let (sharing_mode, queue_families) = if graphics.family == present.family {
            (vk::SharingMode::EXCLUSIVE, Vec::new())
        } else {
            (
                vk::SharingMode::CONCURRENT,
                vec![graphics.family, present.family],
            )
        };
        let min_image_count = (swapchain_info.surface_caps.min_image_count + 1)
            .min(swapchain_info.surface_caps.max_image_count);

        let swapchain_create_info = vk::SwapchainCreateInfoKHRBuilder::new()
            .surface(swapchain_info.surface)
            .min_image_count(min_image_count)
            .image_format(swapchain_info.surface_format.format)
            .image_color_space(swapchain_info.surface_format.color_space)
            .image_extent(swapchain_info.extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(sharing_mode)
            .queue_family_indices(&queue_families)
            .pre_transform(swapchain_info.surface_caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagBitsKHR::OPAQUE_KHR)
            .present_mode(swapchain_info.present_mode)
            .clipped(true);
        let swapchain =
            unsafe { device.create_swapchain_khr(&swapchain_create_info, None) }.unwrap();

        let image_create_info = vk::ImageCreateInfoBuilder::new()
            .image_type(vk::ImageType::_2D)
            .format(vk::Format::D32_SFLOAT)
            .extent(vk::Extent3D {
                width: swapchain_info.extent.width,
                height: swapchain_info.extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlagBits::_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);
        let depth_image = unsafe { device.create_image(&image_create_info, None) }.unwrap();
        let depth_image_allocation = allocator
            .allocate_memory_for_image(&device, depth_image, vk_alloc::MemoryLocation::GpuOnly)
            .unwrap();
        unsafe {
            device.bind_image_memory(
                depth_image,
                depth_image_allocation.device_memory,
                depth_image_allocation.offset,
            )
        }
        .unwrap();
        let image_view_create_info = vk::ImageViewCreateInfoBuilder::new()
            .image(depth_image)
            .view_type(vk::ImageViewType::_2D)
            .format(vk::Format::D32_SFLOAT)
            .components(vk::ComponentMapping::default())
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });
        let depth_image_view =
            unsafe { device.create_image_view(&image_view_create_info, None) }.unwrap();

        let images = unsafe { device.get_swapchain_images_khr(swapchain, None) }.unwrap();
        let swapchain_images = images
            .into_iter()
            .map(|image| {
                let view_create_info = vk::ImageViewCreateInfoBuilder::new()
                    .image(image)
                    .view_type(vk::ImageViewType::_2D)
                    .format(swapchain_info.surface_format.format)
                    .components(vk::ComponentMapping::default())
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                let view = unsafe { device.create_image_view(&view_create_info, None) }.unwrap();
                let attachments = [view, depth_image_view];
                let framebuffer_create_info = vk::FramebufferCreateInfoBuilder::new()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(swapchain_info.extent.width)
                    .height(swapchain_info.extent.height)
                    .layers(1);
                let framebuffer =
                    unsafe { device.create_framebuffer(&framebuffer_create_info, None) }.unwrap();
                SwapchainFramebuffer {
                    view,
                    framebuffer,
                    fence: vk::Fence::null(),
                }
            })
            .collect();

        let command_pool_create_info = vk::CommandPoolCreateInfoBuilder::new()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(graphics.family);
        let command_pool =
            unsafe { device.create_command_pool(&command_pool_create_info, None) }.unwrap();

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfoBuilder::new()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT);
        let command_buffers =
            unsafe { device.allocate_command_buffers(&command_buffer_allocate_info) }.unwrap();
        let semaphore_create_info = vk::SemaphoreCreateInfoBuilder::new();
        let fence_create_info =
            vk::FenceCreateInfoBuilder::new().flags(vk::FenceCreateFlags::SIGNALED);
        let render_sync_objects = command_buffers
            .into_iter()
            .map(|command_buffer| {
                let in_flight = unsafe { device.create_fence(&fence_create_info, None) }.unwrap();
                let image_available =
                    unsafe { device.create_semaphore(&semaphore_create_info, None) }.unwrap();
                let render_finished =
                    unsafe { device.create_semaphore(&semaphore_create_info, None) }.unwrap();
                RenderSyncObject {
                    in_flight,
                    image_available,
                    render_finished,
                    command_buffer,
                }
            })
            .collect();

        Self {
            instance,
            render_pass,
            extent: swapchain_info.extent,

            swapchain,
            swapchain_framebuffers: swapchain_images,
            depth_image,
            depth_allocation: depth_image_allocation,
            depth_view: depth_image_view,

            command_pool,
            sync_objects: render_sync_objects,
            current_frame: 0,
        }
    }

    pub fn rebuild(&mut self, size: (u32, u32)) {
        let instance = self.instance.clone();
        unsafe {
            std::mem::drop(std::ptr::read(self));
            std::ptr::write(self, Self::new(instance, size))
        }
    }

    pub fn render(&mut self, record: impl FnOnce(vk::CommandBuffer)) -> bool {
        let device = self.instance.device();
        let (graphics, present) = (
            self.instance.graphics_queue(),
            self.instance.present_queue(),
        );
        self.current_frame = (self.current_frame + 1) % self.sync_objects.len();
        let sync = &self.sync_objects[self.current_frame];

        unsafe { device.wait_for_fences(&[sync.in_flight], true, u64::MAX) }.unwrap();

        let image_acquired = unsafe {
            device.acquire_next_image_khr(
                self.swapchain,
                u64::MAX,
                Some(sync.image_available),
                None,
            )
        };
        let index = match image_acquired.result() {
            Ok(x) => x as usize,
            Err(vk::Result::SUBOPTIMAL_KHR | vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                return false;
            }
            Err(e) => panic!("{}", e),
        };

        if !self.swapchain_framebuffers[index].fence.is_null() {
            unsafe {
                device.wait_for_fences(&[self.swapchain_framebuffers[index].fence], true, u64::MAX)
            }
            .unwrap()
        };
        self.swapchain_framebuffers[index].fence = sync.in_flight;

        let render_pass_begin_info = vk::RenderPassBeginInfoBuilder::new()
            .render_pass(self.render_pass)
            .framebuffer(self.swapchain_framebuffers[index].framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.extent,
            })
            .clear_values(&[
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.0, 0.0, 0.0, 0.0],
                    },
                },
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                },
            ]);
        let command_buffer_begin_info = vk::CommandBufferBeginInfoBuilder::new()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            device
                .begin_command_buffer(sync.command_buffer, &command_buffer_begin_info)
                .unwrap();
            device.cmd_begin_render_pass(
                sync.command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            record(sync.command_buffer);
            device.cmd_end_render_pass(sync.command_buffer);
            device.end_command_buffer(sync.command_buffer).unwrap()
        }

        unsafe { device.reset_fences(&[sync.in_flight]) }.unwrap();

        let submit_info = vk::SubmitInfoBuilder::new()
            .wait_semaphores(std::slice::from_ref(&sync.image_available))
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(std::slice::from_ref(&sync.command_buffer))
            .signal_semaphores(std::slice::from_ref(&sync.render_finished));
        unsafe { device.queue_submit(graphics.queue, &[submit_info], Some(sync.in_flight)) }
            .unwrap();

        let image_index = index as u32;
        let present_info = vk::PresentInfoKHRBuilder::new()
            .wait_semaphores(std::slice::from_ref(&sync.render_finished))
            .swapchains(std::slice::from_ref(&self.swapchain))
            .image_indices(std::slice::from_ref(&image_index));
        let presented = unsafe { device.queue_present_khr(present.queue, &present_info) };
        match presented.result() {
            Ok(()) => true,
            Err(vk::Result::SUBOPTIMAL_KHR | vk::Result::ERROR_OUT_OF_DATE_KHR) => false,
            Err(e) => panic!("{}", e),
        }
    }
}

impl Renderable for Swapchain {
    fn render_info(&self) -> RenderInfo {
        RenderInfo {
            render_pass: self.render_pass,
            extent: self.extent,
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        let device = self.instance.device();
        let allocator = self.instance.allocator();

        unsafe {
            for render_sync_object in &self.sync_objects {
                device.destroy_fence(Some(render_sync_object.in_flight), None);
                device.destroy_semaphore(Some(render_sync_object.image_available), None);
                device.destroy_semaphore(Some(render_sync_object.render_finished), None)
            }
            device.destroy_command_pool(Some(self.command_pool), None);
            for swapchain_image in &self.swapchain_framebuffers {
                device.destroy_framebuffer(Some(swapchain_image.framebuffer), None);
                device.destroy_image_view(Some(swapchain_image.view), None)
            }
            device.destroy_image_view(Some(self.depth_view), None);
            allocator
                .deallocate(device, &self.depth_allocation)
                .unwrap();
            device.destroy_image(Some(self.depth_image), None);
            device.destroy_swapchain_khr(Some(self.swapchain), None);
            device.destroy_render_pass(Some(self.render_pass), None)
        }
    }
}
