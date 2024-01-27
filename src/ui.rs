use log::info;
use uefi::Result;
use uefi::prelude::BootServices;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::table::boot::{OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol};
use uefi_services::println;

pub fn set_resolution(
    boot_services: &BootServices,
    desired_resolution: (usize, usize),
) -> Result<ScopedProtocol<GraphicsOutput>> {
    println!("trying to get protos");
    let gop_handle = boot_services.get_handle_for_protocol::<GraphicsOutput>()?;
    // PT: open_protocol_exclusive just hangs forever, so ask more politely
    let mut gop = unsafe {
        boot_services.open_protocol::<GraphicsOutput>(
            OpenProtocolParams {
                handle: gop_handle,
                agent: boot_services.image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
    }?;

    let mut switched_to_desired_resolution = false;
    for mode in gop.modes(boot_services) {
        let res = mode.info().resolution();
        info!("Found supported resolution {:?}", res);
        if res == desired_resolution {
            gop.set_mode(&mode).expect("Failed to set desired resolution");
            switched_to_desired_resolution = true;
            break;
        }
    }
    if !switched_to_desired_resolution {
        panic!("Failed to switch to the desired resolution");
    }

    Ok(gop)
}