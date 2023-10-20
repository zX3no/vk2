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

pub unsafe fn create_surface(entry: &Entry, instance: &Instance) -> (Window, vk::SurfaceKHR) {
    let window = create_window("test window", 600, 400, WS_VISIBLE | WS_OVERLAPPEDWINDOW);
    let surface_fn = khr::Win32Surface::new(&entry, &instance);
    let surface = surface_fn
        .create_win32_surface(
            &vk::Win32SurfaceCreateInfoKHR::default()
                .hinstance(window.hinstance as isize)
                .hwnd(window.hwnd as isize),
            None,
        )
        .unwrap();

    (window, surface)
}

pub unsafe fn create_device(instance: &Instance) -> Device {
    let devices = instance.enumerate_physical_devices().unwrap();
    let device = &devices[0];
    let queue = instance.get_physical_device_queue_family_properties(*device);
    //Check if the gpu supports graphics.
    let (index, _) = queue
        .iter()
        .enumerate()
        .find(|(_, info)| info.queue_flags.contains(vk::QueueFlags::GRAPHICS))
        .unwrap();

    let properties = instance.get_physical_device_properties(*device);

    let name = str_from_i8(&properties.device_name).unwrap();
    minilog::info!("Physical Device: {}", name);

    instance
        .create_device(
            *device,
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
        .unwrap()
}

pub struct Vulkan {
    pub entry: Entry,
    pub instance: Instance,
    pub window: Window,
    pub surface: vk::SurfaceKHR,
    pub device: Device,
}

impl Vulkan {
    pub fn new() -> Self {
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

            let (window, surface) = create_surface(&entry, &instance);
            let device = create_device(&instance);

            Vulkan {
                entry,
                instance,
                window,
                surface,
                device,
            }
        }
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        unsafe {
            let surface_fn = khr::Surface::new(&self.entry, &self.instance);
            surface_fn.destroy_surface(self.surface, None);
        }
    }
}
