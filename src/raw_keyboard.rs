use std::mem;
use std::mem::MaybeUninit;
use std::ptr::{null, null_mut};
use winapi::shared::minwindef::{PUINT, UINT};
use winapi::shared::windef::HWND;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winuser::{GetRawInputDeviceList, PCRAWINPUTDEVICE, RAWINPUTDEVICE, RAWINPUTDEVICELIST};


fn register() {
    let a = RAWINPUTDEVICE {
        usUsagePage: 0,
        usUsage: 0,
        dwFlags: 0,
        hwndTarget: 0 as HWND
    };

    let mut b = 0;
    unsafe {
        let count = GetRawInputDeviceList(null_mut(), &mut b, mem::size_of::<RAWINPUTDEVICELIST>() as UINT);
        if count as i32 == -1 { println!("dum"); }
        let mut raw_input_devices: [RAWINPUTDEVICELIST; 50] = mem::zeroed();
        GetRawInputDeviceList(raw_input_devices.as_mut_ptr(), &mut b, mem::size_of::<RAWINPUTDEVICELIST>() as UINT);
        println!("{}, {}", b, count as i32);
        let d = GetLastError();
        println!("{d:?}");

    }
}

pub fn init() {
    register();
}
