// #![windows_subsystem = "windows"]
///!https://github.com/ash-rs/ash/blob/master/examples/src/lib.rs
///!https://github.com/zX3no/vk/blob/main/src/main.rs
///!https://vkguide.dev/
///!https://vulkan-tutorial.com/
///!https://docs.rs/ash/latest/ash/index.html
use vk2::Vulkan;
use win_window::*;

fn main() {
    let vk = Vulkan::new(800, 600, true);
    mini::results!();

    let mut frame_number = 0.0;

    loop {
        unsafe { vk2::draw(&vk, &mut frame_number) };
        match event() {
            Event::Quit => break,
            _ => {}
        }
    }
}
