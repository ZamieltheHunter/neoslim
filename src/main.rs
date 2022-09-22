use pam_client::{Context, Flag};
use pam_client::conv_cli::Conversation;

fn startX() {
    println!("Starting X not actually supported");

    //TODO Actually start X11
}

fn authenticate() {
    //TODO work on conversation handler
    ()
}

fn main() {
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

    println!("Hello, world!");
}
