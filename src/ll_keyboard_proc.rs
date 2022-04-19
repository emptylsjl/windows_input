
use std::cell::{Ref, RefCell};
use std::collections::{HashMap, VecDeque};
use std::lazy::Lazy;
use std::os::raw::{c_int, c_long};
use std::process;
use std::ptr::null_mut;

use winapi::shared::minwindef::{DWORD, HINSTANCE, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HHOOK, HWND, POINT};
use winapi::um::winuser::{CallNextHookEx, GetMessageA, HOOKPROC, KBDLLHOOKSTRUCT, MSG, SetWindowsHookExA, UnhookWindowsHookEx};

use crate::wm_vkey::{WM_VKEY, WM_VKEY::*, WM_KEYSTATUS, WM_KEYSTATUS::*, KeyStatus};

const WH_KEYBOARD_LL: c_int = 13;

static mut LLKP: Lazy<RefCell<LLKeyboard>> = Lazy::new(|| { RefCell::new(LLKeyboard::new()) });
static mut LLKP_HOOK: Option<HHOOK> = None;

#[derive(Debug, Default, Clone)]
struct Keystroke {
    key: WM_VKEY,
    status: KeyStatus,
    count: u32,
}

impl From<u32> for Keystroke {
    fn from(key: u32) -> Self {
        Keystroke{
            key: WM_VKEY::from(key & 0xff),
            status: KeyStatus::from(key & 0x100),
            count: key >> 9
        }
    }
}

impl PartialEq for Keystroke {
    fn eq(&self, other: &Self) -> bool {
        (self.key == other.key) & (self.status == other.status)
    }
}

struct KeyEvent {
    consume: bool,
    event: Box<dyn Fn(Ref<LLKeyboard>) + 'static>,
}

impl Default for KeyEvent {
    fn default() -> Self {
        KeyEvent {
            consume: false,
            event: Box::new(|_| {})
        }
    }
}

struct LLKeyboard {
    events: HashMap<WM_VKEY, KeyEvent>,
    records: VecDeque<Keystroke>,
    records_u128: u128,
}

// impl Clone for LLKeyboard {
//     fn clone(&self) -> Self {
//         LLKeyboard {
//             hhk: None,
//             events: ,
//             records: Default::default(),
//             records_u128: 0
//         }
//     }
// }

impl LLKeyboard {

    fn new() -> LLKeyboard {
        LLKeyboard {
            events: HashMap::with_capacity(0xff),
            records: VecDeque::from(vec![Keystroke::default(); 10]),
            records_u128: 0
        }
    }

    fn add_event<F: Fn(Ref<LLKeyboard>) + 'static>(&mut self, key: WM_VKEY, event: F, consume: bool) {
        let event = Box::new(event);
        self.events.insert(
            key,
            KeyEvent { consume, event }
        );
    }

    fn remove_event(&mut self, key: WM_VKEY) -> Option<KeyEvent> {
        self.events.remove(&key)
    }

    fn record_key(&mut self, key: WM_VKEY, status: KeyStatus) {
        let key = Keystroke {key, status, count: 0};
        if key == self.records[0] {
            self.records[0].count += 1;
        } else {
            self.records.pop_back();
            self.records.push_front(key);
        }
    }

    fn get_records(&self) -> Vec<Keystroke>{
        Vec::from(self.records.clone())
    }

    fn record_key_u128(&mut self, vkCode: u32, status: u32) {
        let cur = (self.records_u128 & 0xffff) as u32;
        let key = ((status % 2) << 8) + vkCode;
        let repeat = (cur & 0x1ff == key) as u32;
        let limit = (cur & 0xfe00 != 0xfe00) as u32;
        self.records_u128 <<= 16 * (repeat ^ 1);
        self.records_u128 += (((repeat * limit) << 9) + key * (repeat ^ 1)) as u128;
    }

    fn get_records_u128(&self) -> Vec<Keystroke>{
        let mut record = self.records_u128;
        let mut records: Vec<Keystroke> = Vec::with_capacity(8);
        for i in 0..8 {
            records.push(Keystroke::from((record & 0xffff) as u32));
            record >>= 16;
        };
        records
    }
}

unsafe fn llkp_hook(keyboard_proc: HOOKPROC) {
    LLKP_HOOK = Some(SetWindowsHookExA(WH_KEYBOARD_LL, keyboard_proc, 0 as HINSTANCE, 0 as DWORD));
    // UnhookWindowsHookEx(self.hhk.unwrap());
}

unsafe fn llkp_unhook() {
    if !LLKP_HOOK.is_none() {
        UnhookWindowsHookEx(LLKP_HOOK.unwrap());
    }
    LLKP_HOOK = None;
}

fn get_message() {
    unsafe {
        let mut msg: MSG = MSG {
            hwnd : 0 as HWND,
            message : 0 as UINT,
            wParam : 0 as WPARAM,
            lParam : 0 as LPARAM,
            time : 0 as DWORD,
            pt : POINT { x: 0 as c_long, y: 0 as c_long, },
        };
        GetMessageA(&mut msg, 0 as HWND, 0 as UINT, 0 as UINT);
    }
}

unsafe extern "system" fn ll_keyboard_proc(code: c_int, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let KBDLLHOOKSTRUCT {
        vkCode, scanCode, flags, time, dwExtraInfo
    } = *(l_param as *const KBDLLHOOKSTRUCT);

    let key = WM_VKEY::from(vkCode);
    let status = KeyStatus::from(w_param as u32);
    println!("{key:?}");
    LLKP.borrow_mut().record_key(key, status);

    // key_event = LLKP.borrow_mut().events.get(&key).unwrap_or(key_event);
    // let llkp = ;
    // (key_event.event)();
    // if key_event.consume { return 1; }
    // let mut key_event = &KeyEvent::default();
    // let key_event = LLKP.borrow_mut().events.get(&key).unwrap_or(key_event);
    // {
    //     let a = LLKP.borrow_mut();
    //     let b = a.events.get(&key).unwrap_or(key_event);
    //     key_event = (*b).
    // }

    // let event = ;
    if let Some(key_event) = LLKP.borrow().events.get(&key) {
        (key_event.event)(LLKP.borrow());
        if key_event.consume { return 1; }
    }


    CallNextHookEx(null_mut(), code, w_param, l_param)
}


pub fn init() {
    unsafe {
        let x = 5;
        LLKP.borrow_mut().add_event(KEY_A,move |_| { println!("{}", x);}, true);
        LLKP.borrow_mut().add_event(KEY_B, |llkp| { llkp.get_records().iter().for_each(|x| { println!("{:?}-{:?} ", x.key, x.status);});}, true);
        LLKP.borrow_mut().add_event(RSHIFT, |_| { llkp_unhook(); process::exit(0); }, true);

        llkp_hook(Some(ll_keyboard_proc));
        get_message();
    }
}
