#![allow(non_snake_case)]
use pam_client::{Context, Flag};
use pam_client::conv_cli::Conversation;
use nix::sys::ioctl;
use nix::ioctl_read_bad;
use core::ffi::c_ushort;
use nix::fcntl::open;

pub const VT_GETSTATE: u32 = 22019;
#[repr (C)]
pub struct vt_stat {
    v_active: c_ushort,
    v_signal: c_ushort,
    v_state: c_ushort
}

ioctl_read_bad!(getVTState, VT_GETSTATE, vt_stat);

fn startX() {
    //TODO Actually start X11
    println!("Starting X not actually supported");
}

fn authenticate() {
    //TODO work on conversation handler
    ()
}

fn findVirtualTerminal() {
    // TODO Use IOCTL to get the current active virtual terminal so we know where to start X
    let termPath = "/dev/tty0";
    let ttyFD = open(termPath, nix::fcntl::OFlag::O_RDONLY, nix::sys::stat::Mode::empty()).expect("Failed to open tty");
    println!("Looks like we opened the tty");
    let mut termInfo = vt_stat {v_active: 0,v_signal: 0,v_state: 0};
    let mut termPtr: *mut vt_stat = &mut termInfo;
    let err = unsafe { getVTState(ttyFD, termPtr)};
    match err {
        Err(e) => println!("Oofus, errored with code: {}", e),
        Ok(_) => println!("We got some terminfo it is v_active: {}, v_signal: {}, v_state: {}", termInfo.v_active, termInfo.v_signal, termInfo.v_state),
    }
    ()
}
fn main() {

    findVirtualTerminal();
    let mut username = String::new();
    let mut context  = Context::new(
        "neoslim",
        None,
        Conversation::new()
    ).expect("Failed to start PAM Authentication");

    match context.set_user_prompt(Some("Who cleanses the dark ones taint? ")) {
        Err(e) => println!("PAM didn't like the dark ones taint: {}", e),
        Ok(_) => (),
    };
    match context.authenticate(Flag::NONE) {
        Err(_) => println!("RIP, don't got no AUTH"),
        Ok(_) => match context.user() {
            Err(_) => println!("Do got some Auth for you <unknown user>"),
            Ok(name) => {
                username = name.clone();
                println!("Do got some AUTH for you {}", &username);
            },
        },
    };
    match context.acct_mgmt(Flag::NONE){
        Err(_) => println!("RIP, don't got no account validation"),
        Ok(_) => println!("Do got some account validation for you"),
    };

    for object in &context.envlist() {
        println!("VAR: {}", object);
    }
    let mut session = context.open_session(Flag::NONE).expect("RIP, don't got no open session");
}
