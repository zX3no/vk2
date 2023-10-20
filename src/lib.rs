use ash::{extensions::khr, *};
use win_window::*;

pub unsafe fn str_from_i8(slice: &[i8]) -> Result<&str, std::str::Utf8Error> {
    let (end, _) = slice
        .iter()
        .enumerate()
        .find(|(_, c)| **c == b'\0' as i8)
        .unwrap();
    std::str::from_utf8(std::mem::transmute(&slice[..end]))
}

pub unsafe fn create_surface(
    entry: &Entry,
    instance: &Instance,
    width: u32,
    height: u32,
) -> (Window, vk::SurfaceKHR) {
    let window = create_window(
        "test window",
        width as i32,
        height as i32,
        WS_VISIBLE | WS_OVERLAPPEDWINDOW,
    );
    let win32_surface_fn = khr::Win32Surface::new(&entry, &instance);
    let surface = win32_surface_fn
        .create_win32_surface(
            &vk::Win32SurfaceCreateInfoKHR::default()
                .hinstance(window.hinstance as isize)
                .hwnd(window.hwnd as isize),
            None,
        )
        .unwrap();

    (window, surface)
}

pub unsafe fn create_device(instance: &Instance) -> (vk::PhysicalDevice, Device, vk::Queue) {
    let devices = instance.enumerate_physical_devices().unwrap();
    let physical_device = &devices[0];
    let queue = instance.get_physical_device_queue_family_properties(*physical_device);
    let (index, _) = queue
        .iter()
        .enumerate()
        .find(|(_, info)| info.queue_flags.contains(vk::QueueFlags::GRAPHICS))
        .unwrap();

    let properties = instance.get_physical_device_properties(*physical_device);
    let name = str_from_i8(&properties.device_name).unwrap();
    minilog::info!("Physical Device: {}", name);

    let device = instance
        .create_device(
            *physical_device,
            &vk::DeviceCreateInfo::default()
                .queue_create_infos(&[vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(index as u32)
                    .queue_priorities(&[1.0])])
                .enabled_extension_names(&[khr::Swapchain::NAME.as_ptr()])
                .enabled_features(&vk::PhysicalDeviceFeatures {
                    shader_clip_distance: 1,
                    ..Default::default()
                }),
            None,
        )
        .unwrap();
    let queue = device.get_device_queue(index as u32, 0);

    (*physical_device, device, queue)
}

pub const SURFACE_FORMAT: vk::SurfaceFormatKHR = vk::SurfaceFormatKHR {
    format: vk::Format::B8G8R8A8_UNORM,
    color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
};

pub unsafe fn get_surface_info(
    entry: &Entry,
    instance: &Instance,
    surface: &vk::SurfaceKHR,
    physical_device: &vk::PhysicalDevice,
) -> u32 {
    let surface_fn = khr::Surface::new(&entry, &instance);
    let surface_formats = surface_fn
        .get_physical_device_surface_formats(*physical_device, *surface)
        .unwrap();
    if !surface_formats.contains(&SURFACE_FORMAT) {
        panic!(
            "Physical device does not support this format: {:?}",
            SURFACE_FORMAT
        );
    }
    let surface_capabilities = surface_fn
        .get_physical_device_surface_capabilities(*physical_device, *surface)
        .unwrap();

    //Usually this is 2 for front and back buffer.
    //Add one more to tripple buffer.
    surface_capabilities.min_image_count + 1
}

pub unsafe fn create_swapchain(
    instance: &Instance,
    surface: &vk::SurfaceKHR,
    device: &Device,
    min_image_count: u32,
    width: u32,
    height: u32,
) -> (vk::SwapchainKHR, Vec<vk::Image>, Vec<vk::ImageView>) {
    let swapchain_fn = khr::Swapchain::new(instance, device);
    let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
        .surface(*surface)
        .min_image_count(min_image_count)
        .image_color_space(SURFACE_FORMAT.color_space)
        .image_format(SURFACE_FORMAT.format)
        .image_extent(vk::Extent2D { width, height })
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(vk::PresentModeKHR::FIFO_RELAXED)
        .clipped(true)
        .image_array_layers(1);

    let swapchain = swapchain_fn
        .create_swapchain(&swapchain_create_info, None)
        .unwrap();

    let images = swapchain_fn.get_swapchain_images(swapchain).unwrap();
    let image_views: Vec<vk::ImageView> = images
        .iter()
        .map(|&image| {
            device
                .create_image_view(
                    &vk::ImageViewCreateInfo::default()
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(SURFACE_FORMAT.format)
                        .components(vk::ComponentMapping {
                            r: vk::ComponentSwizzle::R,
                            g: vk::ComponentSwizzle::G,
                            b: vk::ComponentSwizzle::B,
                            a: vk::ComponentSwizzle::A,
                        })
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        })
                        .image(image),
                    None,
                )
                .unwrap()
        })
        .collect();

    (swapchain, images, image_views)
}

pub enum ShaderType {
    Vertex,
    Fragment,
}

///https://vkguide.dev/docs/chapter-2/toggling_shaders/
pub unsafe fn create_shader(device: &Device, bytes: &[u8], shader_type: ShaderType) {
    const MAIN: *const i8 = b"main\0" as *const u8 as *const i8;

    let (_, code, _) = unsafe { bytes.align_to::<u32>() };
    let shader_info = vk::ShaderModuleCreateInfo::default().code(&code);
    let shader_module = device.create_shader_module(&shader_info, None).unwrap();

    let shader = vk::PipelineShaderStageCreateInfo {
        module: shader_module,
        p_name: MAIN,
        stage: match shader_type {
            ShaderType::Vertex => vk::ShaderStageFlags::VERTEX,
            ShaderType::Fragment => vk::ShaderStageFlags::FRAGMENT,
        },
        ..Default::default()
    };
}

pub unsafe fn create_graphics_pipeline() {}

pub struct Vulkan {
    pub entry: Entry,
    pub instance: Instance,
    pub window: Window,
    pub surface: vk::SurfaceKHR,
    pub device: Device,
    pub queue: vk::Queue,

    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_view: Vec<vk::ImageView>,
}

impl Vulkan {
    pub fn new(width: u32, height: u32) -> Self {
        unsafe {
            let entry = ash::Entry::linked();
            let instance = entry
                .create_instance(
                    &vk::InstanceCreateInfo::default().enabled_extension_names(&[
                        khr::Surface::NAME.as_ptr(),
                        khr::Win32Surface::NAME.as_ptr(),
                    ]),
                    None,
                )
                .unwrap();

            let (window, surface) = create_surface(&entry, &instance, width, height);
            let (physical_device, device, queue) = create_device(&instance);

            let min = get_surface_info(&entry, &instance, &surface, &physical_device);
            let (swapchain, images, image_view) =
                create_swapchain(&instance, &surface, &device, min, width, height);

            Vulkan {
                entry,
                instance,
                window,
                surface,
                device,
                queue,
                swapchain,
                images,
                image_view,
            }
        }
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        unsafe {
            let surface_fn = khr::Surface::new(&self.entry, &self.instance);
            let swapchain_fn = khr::Swapchain::new(&self.instance, &self.device);

            swapchain_fn.destroy_swapchain(self.swapchain, None);

            for image in std::mem::take(&mut self.image_view) {
                self.device.destroy_image_view(image, None);
            }

            self.device.destroy_device(None);
            surface_fn.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
