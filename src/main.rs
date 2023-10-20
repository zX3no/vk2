use vk2::Vulkan;
use win_window::*;

fn main() {
    let _vk = Vulkan::new(800, 600);

    loop {
        match event() {
            Event::Quit => break,
            _ => {}
        }
    }
}
