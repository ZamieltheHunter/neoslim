#![allow(non_snake_case)]
use std::env;
use pam_client::{Context, Flag,conv_cli::Conversation};
use nix::{sys::{ioctl,wait::waitpid,stat::Mode},ioctl_read_bad,fcntl::{OFlag,open},unistd::{execv, fork, ForkResult, write}};
use core::ffi::c_ushort;
use std::ffi::{CString, CStr};

// VT_GETSTATE and vt_stat are defined in /usr/include/linux/vt.h
// Repr C ensures that the memory layout is the same as C's
pub const VT_GETSTATE: u32 = 22019;
#[repr (C)]
pub struct vt_stat {
    v_active: c_ushort,
    v_signal: c_ushort,
    v_state: c_ushort
}

// Nix helper macro to generate the ioctl function call
ioctl_read_bad!(getVTState, VT_GETSTATE, vt_stat);

fn authenticate() {
    //TODO work on conversation handler
    ()
}

fn startPAMAuthentication() -> Result<(), pam_client::ErrorWith<pam_client::ErrorCode>>{
    let mut username = String::new();
    let mut context  = Context::new(
        "neoslim",
        None,
        Conversation::new()
    ).expect("Failed to start PAM Authentication");

    match context.set_user_prompt(Some("Who cleanses the dark ones taint? ")) {
        Err(e) => println!("PAM didn't like the dark ones taint: {}", e),
        Ok(_) => println!("Prompt set"),
    };
    match context.authenticate(Flag::NONE) {
        Err(e) => println!("RIP, don't got no AUTH: {}", e),
        Ok(_) => match context.user() {
            Err(_) => println!("Do got some Auth for you <unknown user>"),
            Ok(name) => {
                username = name.clone();
                println!("Do got some AUTH for you {}", &username);
            },
        },
    };
    match context.acct_mgmt(Flag::NONE){
        Err(e) => println!("RIP, don't got no account validation: {}", e),
        Ok(_) => println!("Do got some account validation for you"),
    };

    for object in &context.envlist() {
        println!("VAR: {}", object);
    }
    let mut session = context.open_session(Flag::NONE).expect("RIP, don't got no open session");
    Ok(())
}

// This function will fork and then exec the server
// If the -t flag is given on the command line we are in testing mode and will run Xephyr
fn startServer(testing: bool, vtNum: c_ushort) -> Result<(), nix::Error> {
    //TODO Start a Xephyr server for testing purposes (Thank you Gulshan Singh
    //https://www.gulshansingh.com/posts/how-to-write-a-display-manager)
    match unsafe {fork()} {
        Ok(ForkResult::Parent {child, ..}) => {
            println!("Parent here!");
            Ok(())
        },
        Ok(ForkResult::Child) => {
            println!("Child here!");
            if testing {
                let command = CString::new("/usr/bin/Xephyr").expect("Failed to make cstring");
                let args = &[CString::new("-ac").unwrap(),CString::new("-br").unwrap(),CString::new("-noreset").unwrap(),CString::new("-screen").unwrap(),CString::new("802x600").unwrap(), CString::new(":1").unwrap()];
                println!("Start Xephyr here");
                execv(&command, args).expect("Failed to exec :(");
                std::process::exit(1);
            } else {
                println!("Start Xorg here");
            }
            std::process::exit(0);
            Ok(())
        },
        Err(e) => {
            println!("Failed to fork");
            Err(e)
        },
    }
}

// Runs the VT_GETSTATE ioctl call to fetch the virtual terminal number of the current virtual
// terminal to pass along to Xorg so it doesn't fail.
fn findVirtualTerminal() -> Result<c_ushort, nix::Error> {
    let termPath = "/dev/tty0";
    let ttyFD = open(termPath, OFlag::O_RDONLY, Mode::empty()).expect("Failed to open tty");
    println!("Looks like we opened the tty");

    //Create the destination struct for the data
    let mut termInfo = vt_stat {v_active: 0,v_signal: 0,v_state: 0};
    let termPtr: *mut vt_stat = &mut termInfo;

    // Actually runs the syscall that the ioctl_read_bad macro defined for us.
    let err = unsafe { getVTState(ttyFD, termPtr)};
    match err {
        Err(e) => Err(e),
        Ok(_) => {
            println!("We got some terminfo it is v_active: {}, v_signal: {}, v_state: {}", termInfo.v_active, termInfo.v_signal, termInfo.v_state);
            Ok(termInfo.v_active)
        },
    }
}

// Checks for testing flag, then calls findVirtualTerminal for setup before sending the info to
// startServer
fn main() {
    let args: Vec<String> = env::args().collect();
    let testing = args.iter().any(|arg| arg == "-t");
    match findVirtualTerminal(){
        Err(e) => {
            panic!("Failed to get the virtual terminal info with error: {}", e);
        },
        Ok(v_active) => {
            match startServer(testing, v_active) {
                Err(e) => panic!("Failed to start X server {}", e),
                Ok(_) => {
                    startPAMAuthentication().unwrap();
                }
            };
        },
    };
}
