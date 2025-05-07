
use std::path::PathBuf;
use std::fs;
use std::env;
use std::sync::LazyLock;


//mod setinstance;
//mod lazy;

mod dev;
mod dist;
mod common_helperfuncs;
mod init;
 





static INSTANCE_NAME: LazyLock<String> = LazyLock::new(|| {
    let name = env::var("NIFTY_INSTANCE").expect("NIFTY_INSTANCE env not set");
    name.to_uppercase()
});


static CONFIGS_PATH: LazyLock<String> = LazyLock::new(|| {
    let name = env::var("NIFTY_CONFIGS_DIR").expect("NIFTY_CONFIGS_DIR env not set");
    name.to_lowercase()
});


static MAIN_CLIENT_PATH: LazyLock<String> = LazyLock::new(|| {
    env::var("NIFTYCLIENT_DIR").expect("NIFTYCLIENT_DIR env not set")
});


static MAIN_SERVER_PATH: LazyLock<String> = LazyLock::new(|| {
    env::var("NIFTYSERVER_DIR").expect("NIFTYSERVER_DIR env not set")
});


static INSTANCE_SERVER_PATH: LazyLock<String> = LazyLock::new(|| {
    let n = INSTANCE_NAME.clone();
    let env_var_name = format!("NIFTY_INSTANCE_{}_SERVER_DIR", n);
    let error_str = format!("{} env var not found", env_var_name);
    env::var(env_var_name).expect(error_str.as_str())
});


static INSTANCE_CLIENT_PATH: LazyLock<String> = LazyLock::new(|| {
    let n = INSTANCE_NAME.clone();
    let env_var_name = format!("NIFTY_INSTANCE_{}_CLIENT_DIR", n);
    let error_str = format!("{} env var not found", env_var_name);
    env::var(env_var_name).expect(error_str.as_str())
});


static TMP_PATH: LazyLock<String> = LazyLock::new(|| "/tmp/niftybuildit/".to_string());


static HTTP_PORT: LazyLock<String> = LazyLock::new(|| {
    let n = INSTANCE_NAME.clone().to_uppercase();
    let env_var_name = format!("NIFTY_INSTANCE_{}_PORT", n);
    let error_str = format!("{} env var port not found", env_var_name);
    env::var(env_var_name).expect(error_str.as_str())
});


static CHROME_OVERRIDES_PATH: LazyLock<String> = LazyLock::new(|| "/Users/dave/Code/chrome_overrides/".to_string());


/*
static OFFLINEDATE_DIR: LazyLock<String> = LazyLock::new(|| {
    let name = env::var("NIFTY_OFFLINEDATE_DIR").unwrap_or(String::from(""));
    name // can test == "" to see if set or not 
});
*/


static DEVAPPVERSION: LazyLock<u32> = LazyLock::new(|| {
    let devappversion_content = fs::read_to_string("/tmp/niftybuildit/devappversion.txt").unwrap_or(String::from("0"));
    devappversion_content.trim().parse::<u32>().unwrap_or(0)
});




fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("No arguments provided");
        return;
    }

    let primary_action = &args[1];
    let primary_action_aux  = if args.len() >= 3 { &args[2] } else { "" };

    match primary_action.as_str() {

        "alldev" =>                  { let _ = dev::alldev();   },

        "core" =>                    { let _ = dev::handle_core();   },

        "corelazy" =>                { let _ = dev::handle_corelazy();   },

        "thirdparty" =>              { let _ = dev::thirdparty::runit();   },

        "media" =>                   { let _ = dev::media::runit();   },

        "iconsfont" =>               { let _ = dev::media::iconsfont();   },

        "server" =>                  { let _ = dev::server::runit();   },

        "dist" =>                    { let _ = dist::runit();   },

        "file" =>                    { let x = PathBuf::from(primary_action_aux); let _ = dev::handle_file_changed(&x);   },

        "copy_chrome_css_changes" => { let _ = dev::handle_copy_chrome_css_changes();   },

        "init"                    => { let _ = init::initit(primary_action_aux);   },

        "devappversion"           => { let _ = dev::handle_set_devappversion(primary_action_aux);   },

        _ =>            {   println!("Invalid command line argument");   }
    }
    
}

