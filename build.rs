use std::{env, fs};
use std::path::Path;

fn main() {
    if cfg!(feature = "docgen") {
        let out_dir = env::var_os("OUT_DIR").unwrap();
        let filename = "COMMAND_REFERENCE_GEN.md".to_string();
        let filepath = Path::new(&out_dir)
            .parent().unwrap()
            .parent().unwrap()
            .parent().unwrap()
            .join(&filename);
        match fs::remove_file(filepath.clone()) {
            Ok(_) => (),
            Err(e) => println!("Could not delete {:?}: {}", filepath.to_str(), e) 
        }
        println!("redismodule_cmd_procmacro build.rs completed") 
    }
}