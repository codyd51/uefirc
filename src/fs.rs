use alloc::format;
use alloc::vec::Vec;
use uefi::{CStr16, cstr16};
use uefi::fs::FileSystem;
use uefi::prelude::BootServices;

pub fn read_file(boot_services: &BootServices, path: &str) -> Vec<u8> {
    let mut input_as_u16s = path
        .chars()
        .map(|c| u16::try_from(c as u32))
        .collect::<Result<Vec<u16>, _>>().expect("Failed to convert path characters to u16s");
    // Add a null byte at the end, as CStr16 needs one
    input_as_u16s.push(0);
    let path_as_cstr16 = unsafe {
        CStr16::from_u16_with_nul_unchecked(&input_as_u16s)
    };

    let mut sfs = boot_services.get_image_file_system(boot_services.image_handle()).unwrap();
    let mut fs = FileSystem::new(sfs);
    fs.read(path_as_cstr16).expect(&format!("Failed to read file \"{path}\""))
}
