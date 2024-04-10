use alloc::format;
use alloc::vec::Vec;
use uefi::{CString16};
use uefi::fs::{FileSystem};
use uefi::prelude::BootServices;

pub fn read_file(boot_services: &BootServices, path: &str) -> Vec<u8> {
    let path_as_cstr16 = CString16::try_from(path).expect("Path should only contain UCS2-compatible characters.");
    let sfs = boot_services.get_image_file_system(boot_services.image_handle()).unwrap();
    let mut fs = FileSystem::new(sfs);
    fs.read(path_as_cstr16.as_ref()).expect(&format!("Should be able to read file \"{path}\""))
}
