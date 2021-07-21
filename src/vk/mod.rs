pub use instance::{HeadlessInstance, Instance, WindowedInstance};
pub use renderable::{Renderable, Swapchain};
use types::*;
pub use voxel_mesh::{VoxelMeshManager, VoxelMeshRenderer};

macro_rules! include_shader {
    ($filename:expr) => {
        include_bytes!(concat!(env!("OUT_DIR"), "/", $filename, ".spv"))
    };
}

mod instance;
mod renderable;
mod voxel_mesh;

mod debug {
    use erupt::vk;

    pub fn create_info() -> vk::DebugUtilsMessengerCreateInfoEXTBuilder<'static> {
        vk::DebugUtilsMessengerCreateInfoEXTBuilder::new()
            .message_severity(message_severity())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .pfn_user_callback(Some(callback))
    }

    fn message_severity() -> vk::DebugUtilsMessageSeverityFlagsEXT {
        match log::max_level() {
            log::LevelFilter::Off => vk::DebugUtilsMessageSeverityFlagsEXT::empty(),
            log::LevelFilter::Error => vk::DebugUtilsMessageSeverityFlagsEXT::ERROR_EXT,
            log::LevelFilter::Warn => {
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR_EXT
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING_EXT
            }
            log::LevelFilter::Info => {
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR_EXT
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING_EXT
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO_EXT
            }
            log::LevelFilter::Debug => vk::DebugUtilsMessageSeverityFlagsEXT::all(),
            log::LevelFilter::Trace => vk::DebugUtilsMessageSeverityFlagsEXT::all(),
        }
    }

    unsafe extern "system" fn callback(
        severity: vk::DebugUtilsMessageSeverityFlagBitsEXT,
        _: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _: *mut std::ffi::c_void,
    ) -> vk::Bool32 {
        let level = if severity >= vk::DebugUtilsMessageSeverityFlagBitsEXT::ERROR_EXT {
            log::Level::Error
        } else if severity >= vk::DebugUtilsMessageSeverityFlagBitsEXT::WARNING_EXT {
            log::Level::Warn
        } else if severity >= vk::DebugUtilsMessageSeverityFlagBitsEXT::INFO_EXT {
            log::Level::Info
        } else if severity >= vk::DebugUtilsMessageSeverityFlagBitsEXT::VERBOSE_EXT {
            log::Level::Debug
        } else {
            log::Level::Trace
        };
        let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message).to_string_lossy();
        log::log!(level, "{}", message);
        vk::FALSE
    }
}

mod types {
    use erupt::vk;

    #[derive(Debug, Default, Copy, Clone)]
    pub struct SwapchainInfo {
        pub(super) surface: vk::SurfaceKHR,
        pub(super) surface_caps: vk::SurfaceCapabilitiesKHR,
        pub(super) surface_format: vk::SurfaceFormatKHR,
        pub(super) present_mode: vk::PresentModeKHR,
        pub(super) extent: vk::Extent2D,
    }

    #[derive(Debug, Default, Copy, Clone)]
    pub struct QueueInfo {
        pub(super) family: u32,
        pub(super) queue: vk::Queue,
    }

    #[derive(Debug, Default, Copy, Clone)]
    pub struct RenderInfo {
        pub(super) render_pass: vk::RenderPass,
        pub(super) extent: vk::Extent2D,
    }
}
