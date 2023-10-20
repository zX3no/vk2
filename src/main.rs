use vk2::Vulkan;
use win_window::*;

fn main() {
    let _vk = Vulkan::new();

    loop {
        match event() {
            Event::Quit => break,
            _ => {}
        }
    }
}
