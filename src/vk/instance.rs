use super::{debug, QueueInfo, SwapchainInfo};
use erupt::{vk, ExtendableFromConst};
use raw_window_handle::HasRawWindowHandle;

const VAL_LAYER: *const std::os::raw::c_char = erupt::cstr!("VK_LAYER_KHRONOS_validation");

pub trait Instance {
    fn instance(&self) -> &erupt::InstanceLoader;
    fn device(&self) -> &erupt::DeviceLoader;
    fn allocator(&self) -> &vk_alloc::Allocator;
    fn graphics_queue(&self) -> QueueInfo;

    fn wait_idle(&self) {
        unsafe { self.device().device_wait_idle().unwrap() }
    }
}

pub struct HeadlessInstance {
    entry: std::mem::ManuallyDrop<erupt::EntryLoader>,
    instance: std::mem::ManuallyDrop<erupt::InstanceLoader>,
    device: std::mem::ManuallyDrop<erupt::DeviceLoader>,
    allocator: vk_alloc::Allocator,

    messenger: Option<vk::DebugUtilsMessengerEXT>,
    physical_device: vk::PhysicalDevice,
    graphics_queue: QueueInfo,
}

pub struct WindowedInstance {
    entry: std::mem::ManuallyDrop<erupt::EntryLoader>,
    instance: std::mem::ManuallyDrop<erupt::InstanceLoader>,
    device: std::mem::ManuallyDrop<erupt::DeviceLoader>,
    allocator: vk_alloc::Allocator,

    messenger: Option<vk::DebugUtilsMessengerEXT>,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    graphics_queue: QueueInfo,
    present_queue: QueueInfo,
}

impl HeadlessInstance {
    pub fn new(debug_mode: bool) -> std::sync::Arc<Self> {
        let mut instance_extensions = Vec::new();
        let device_extensions = vec![vk::KHR_SWAPCHAIN_EXTENSION_NAME];
        let (instance_layers, device_layers) = if debug_mode {
            instance_extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION_NAME);
            (vec![VAL_LAYER], vec![VAL_LAYER])
        } else {
            (Vec::new(), Vec::new())
        };

        let (entry, instance, messenger) =
            create_entry_instance_messenger(&instance_extensions, &instance_layers, debug_mode);

        let (physical_device, graphics_family) =
            find_physical_device(&instance, |physical_device| {
                let queue_families = unsafe {
                    instance.get_physical_device_queue_family_properties(physical_device, None)
                };
                let graphics_family = match queue_families
                    .iter()
                    .position(|family| family.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                {
                    Some(index) => index as u32,
                    None => return None,
                };
                Some(graphics_family)
            })
            .unwrap();

        let (device, [graphics_queue]) = create_device(
            &instance,
            &device_extensions,
            &device_layers,
            physical_device,
            &[graphics_family],
        );

        let allocator =
            vk_alloc::Allocator::new(&instance, physical_device, &Default::default()).unwrap();

        std::sync::Arc::new(Self {
            entry,
            instance,
            device,
            allocator,

            messenger,
            physical_device,
            graphics_queue,
        })
    }
}

impl Instance for HeadlessInstance {
    fn instance(&self) -> &erupt::InstanceLoader {
        &self.instance
    }

    fn device(&self) -> &erupt::DeviceLoader {
        &self.device
    }

    fn allocator(&self) -> &vk_alloc::Allocator {
        &self.allocator
    }

    fn graphics_queue(&self) -> QueueInfo {
        self.graphics_queue
    }
}

impl Drop for HeadlessInstance {
    fn drop(&mut self) {
        unsafe {
            self.allocator.cleanup(&self.device);
            self.device.destroy_device(None);
            if let Some(messenger) = self.messenger {
                self.instance
                    .destroy_debug_utils_messenger_ext(Some(messenger), None)
            }
            self.instance.destroy_instance(None);
            std::mem::ManuallyDrop::drop(&mut self.device);
            std::mem::ManuallyDrop::drop(&mut self.instance);
            std::mem::ManuallyDrop::drop(&mut self.entry)
        }
    }
}

impl WindowedInstance {
    pub fn new(window: &impl HasRawWindowHandle, debug_mode: bool) -> std::sync::Arc<Self> {
        let mut instance_extensions =
            erupt::utils::surface::enumerate_required_extensions(window).unwrap();
        let device_extensions = vec![vk::KHR_SWAPCHAIN_EXTENSION_NAME];
        let (instance_layers, device_layers) = if debug_mode {
            instance_extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION_NAME);
            (vec![VAL_LAYER], vec![VAL_LAYER])
        } else {
            (Vec::new(), Vec::new())
        };

        let (entry, instance, messenger) =
            create_entry_instance_messenger(&instance_extensions, &instance_layers, debug_mode);

        let surface =
            unsafe { erupt::utils::surface::create_surface(&instance, window, None) }.unwrap();

        let (physical_device, (graphics_family, present_family)) =
            find_physical_device(&instance, |physical_device| {
                let queue_families = unsafe {
                    instance.get_physical_device_queue_family_properties(physical_device, None)
                };
                let present_family = match (0..queue_families.len()).find(|index| {
                    unsafe {
                        instance.get_physical_device_surface_support_khr(
                            physical_device,
                            *index as u32,
                            surface,
                        )
                    }
                    .unwrap()
                }) {
                    Some(index) => index as u32,
                    None => return None,
                };
                let graphics_family = match queue_families
                    .iter()
                    .position(|family| family.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                {
                    Some(index) => index as u32,
                    None => return None,
                };
                Some((graphics_family, present_family))
            })
            .unwrap();

        let (device, [graphics_queue, present_queue]) = create_device(
            &instance,
            &device_extensions,
            &device_layers,
            physical_device,
            &[graphics_family, present_family],
        );

        let allocator =
            vk_alloc::Allocator::new(&instance, physical_device, &Default::default()).unwrap();

        std::sync::Arc::new(Self {
            entry,
            instance,
            device,
            allocator,

            messenger,
            surface,
            physical_device,
            graphics_queue,
            present_queue,
        })
    }

    pub fn present_queue(&self) -> QueueInfo {
        self.present_queue
    }

    pub fn swapchain_info(&self, (width, height): (u32, u32)) -> SwapchainInfo {
        let surface_caps = unsafe {
            self.instance
                .get_physical_device_surface_capabilities_khr(self.physical_device, self.surface)
        }
        .unwrap();

        let surface_formats = unsafe {
            self.instance.get_physical_device_surface_formats_khr(
                self.physical_device,
                self.surface,
                None,
            )
        }
        .unwrap();
        let first_surface_format = surface_formats[0];
        let surface_format = surface_formats
            .into_iter()
            .find(|surface_format| {
                surface_format.format == vk::Format::B8G8R8A8_SRGB
                    && surface_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR_KHR
            })
            .unwrap_or(first_surface_format);

        let present_modes = unsafe {
            self.instance.get_physical_device_surface_present_modes_khr(
                self.physical_device,
                self.surface,
                None,
            )
        }
        .unwrap();
        let present_mode = present_modes
            .into_iter()
            .find(|present_mode| *present_mode == vk::PresentModeKHR::MAILBOX_KHR)
            .unwrap_or(vk::PresentModeKHR::FIFO_KHR);

        let extent = vk::Extent2D {
            width: width.clamp(
                surface_caps.min_image_extent.width,
                surface_caps.max_image_extent.width,
            ),
            height: height.clamp(
                surface_caps.min_image_extent.height,
                surface_caps.max_image_extent.height,
            ),
        };

        SwapchainInfo {
            surface: self.surface,
            surface_caps,
            surface_format,
            present_mode,
            extent,
        }
    }
}

impl Instance for WindowedInstance {
    fn instance(&self) -> &erupt::InstanceLoader {
        &self.instance
    }

    fn device(&self) -> &erupt::DeviceLoader {
        &self.device
    }

    fn allocator(&self) -> &vk_alloc::Allocator {
        &self.allocator
    }

    fn graphics_queue(&self) -> QueueInfo {
        self.graphics_queue
    }
}

impl Drop for WindowedInstance {
    fn drop(&mut self) {
        unsafe {
            self.allocator.cleanup(&self.device);
            self.device.destroy_device(None);
            self.instance.destroy_surface_khr(Some(self.surface), None);
            if let Some(messenger) = self.messenger {
                self.instance
                    .destroy_debug_utils_messenger_ext(Some(messenger), None)
            }
            self.instance.destroy_instance(None);
            std::mem::ManuallyDrop::drop(&mut self.device);
            std::mem::ManuallyDrop::drop(&mut self.instance);
            std::mem::ManuallyDrop::drop(&mut self.entry)
        }
    }
}

fn create_entry_instance_messenger(
    instance_extensions: &[*const std::os::raw::c_char],
    instance_layers: &[*const std::os::raw::c_char],
    debug_mode: bool,
) -> (
    std::mem::ManuallyDrop<erupt::EntryLoader>,
    std::mem::ManuallyDrop<erupt::InstanceLoader>,
    Option<vk::DebugUtilsMessengerEXT>,
) {
    let entry = erupt::EntryLoader::new().unwrap();

    let application_info =
        vk::ApplicationInfoBuilder::new().api_version(vk::make_api_version(0, 1, 2, 0));
    if debug_mode {
        let messenger_create_info = debug::create_info();
        let instance_create_info = vk::InstanceCreateInfoBuilder::new()
            .extend_from(&messenger_create_info)
            .application_info(&application_info)
            .enabled_layer_names(instance_layers)
            .enabled_extension_names(instance_extensions);
        let instance =
            unsafe { erupt::InstanceLoader::new(&entry, &instance_create_info, None) }.unwrap();
        let messenger =
            unsafe { instance.create_debug_utils_messenger_ext(&messenger_create_info, None) }
                .unwrap();
        (
            std::mem::ManuallyDrop::new(entry),
            std::mem::ManuallyDrop::new(instance),
            Some(messenger),
        )
    } else {
        let instance_create_info = vk::InstanceCreateInfoBuilder::new()
            .application_info(&application_info)
            .enabled_layer_names(instance_layers)
            .enabled_extension_names(instance_extensions);
        let instance =
            unsafe { erupt::InstanceLoader::new(&entry, &instance_create_info, None) }.unwrap();
        (
            std::mem::ManuallyDrop::new(entry),
            std::mem::ManuallyDrop::new(instance),
            None,
        )
    }
}

fn find_physical_device<T>(
    instance: &erupt::InstanceLoader,
    filter: impl Fn(vk::PhysicalDevice) -> Option<T>,
) -> Option<(vk::PhysicalDevice, T)> {
    let physical_devices = unsafe { instance.enumerate_physical_devices(None) }.unwrap();
    physical_devices
        .into_iter()
        .filter_map(|physical_device| filter(physical_device).map(|x| (physical_device, x)))
        .min_by_key(|(physical_device, _)| {
            let properties = unsafe { instance.get_physical_device_properties(*physical_device) };
            match properties.device_type {
                vk::PhysicalDeviceType::DISCRETE_GPU => 0,
                vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
                _ => 2,
            }
        })
}

fn create_device<const N: usize>(
    instance: &erupt::InstanceLoader,
    device_extensions: &[*const std::os::raw::c_char],
    device_layers: &[*const std::os::raw::c_char],
    physical_device: vk::PhysicalDevice,
    queue_families: &[u32; N],
) -> (std::mem::ManuallyDrop<erupt::DeviceLoader>, [QueueInfo; N]) {
    let unique_queues = queue_families
        .iter()
        .collect::<std::collections::HashSet<_>>();
    let queue_create_infos = unique_queues
        .into_iter()
        .map(|family| {
            vk::DeviceQueueCreateInfoBuilder::new()
                .queue_family_index(*family)
                .queue_priorities(&[1.0])
        })
        .collect::<Vec<_>>();
    let features = vk::PhysicalDeviceFeaturesBuilder::new().fill_mode_non_solid(true);
    let device_create_info = vk::DeviceCreateInfoBuilder::new()
        .queue_create_infos(&queue_create_infos)
        .enabled_layer_names(device_layers)
        .enabled_extension_names(device_extensions)
        .enabled_features(&features);
    let device =
        unsafe { erupt::DeviceLoader::new(instance, physical_device, &device_create_info, None) }
            .unwrap();
    //TODO: simplify when array_map stabilizes
    let mut queues = [QueueInfo::default(); N];
    for i in 0..N {
        queues[i] = QueueInfo {
            family: queue_families[i],
            queue: unsafe { device.get_device_queue(queue_families[i], 0) },
        }
    }
    (std::mem::ManuallyDrop::new(device), queues)
}
