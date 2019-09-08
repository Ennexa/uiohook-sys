extern crate bindgen;

use std::env;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
	let target = env::var("TARGET").expect("TARGET was not set");

    println!("cargo:rerun-if-changed=build.rs");

    if cfg!(feature = "with-bindgen") {
    	let bindings = bindgen::Builder::default()
            .header("vendor/include/uiohook.h")
            .whitelist_var("_event_type_EVENT.*")
            .whitelist_var("_log_level_LOG_LEVEL.*")
            .whitelist_function("logger_proc")
            .whitelist_function("hook_set_logger_proc")
            .whitelist_function("hook_post_event")
            .whitelist_function("hook_set_dispatch_proc")
            .whitelist_function("hook_run")
            .whitelist_function("hook_stop")
            .whitelist_function("hook_create_screen_info")
            .whitelist_function("hook_get_auto_repeat_rate")
            .whitelist_function("hook_get_auto_repeat_delay")
            .whitelist_function("hook_get_pointer_acceleration_multiplier")
            .whitelist_function("hook_get_pointer_acceleration_threshold")
            .whitelist_function("hook_get_pointer_sensitivity")
            .whitelist_function("hook_get_multi_click_time")
            .trust_clang_mangling(false)
            .generate()
			.expect("Unable to generate bindings");

		let out_path = PathBuf::from("src").join("bindings.rs");

		let data = bindings
			.to_string()
			.replace("_event_type_EVENT", "EVENT")
			.replace("_log_level_LOG_LEVEL", "LOG_LEVEL")
		;

        let mut file = File::create(out_path).expect("couldn't open file!");
        file.write_all(data.as_bytes())
            .expect("couldn't write bindings.rs!");
	}

    if cfg!(feature = "static") {
        if !Path::new("vendor/.git").exists() {
            let _ = Command::new("git")
                .args(&["submodule", "update", "--init"])
                .status();
        }

	    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());
	    let include = dst.join("include");
	    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

        fs::create_dir_all(&include).unwrap();
        fs::copy("vendor/include/uiohook.h", include.join("uiohook.h")).unwrap();

        if !Path::new("vendor/.libs").exists() {
            let mut args = vec!["--enable-demo=no"];

            if target.contains("musl") {
                env::set_var("CC", "musl-gcc");
            }

            if target.contains("windows-gnu") {
                if target.contains("x86_64") {
                    args.push("--host=x86_64-w64-mingw32");
                } else {
                    args.push("--host=i686-w64-mingw32");
                }
            }

            Command::new("autoreconf")
                .args(&mut vec![
                	"--install",
                	"--verbose",
                	"--force",
                	"./vendor"
                ])
                .output()
                .unwrap();

            Command::new(&format!("{}/vendor/configure", crate_dir))
                .args(&args)
                .current_dir(&dst)
                .output()
                .unwrap();

            Command::new("make").current_dir(&dst).output().unwrap();
        }

        let libdir = dst.join(".libs");

	    println!("cargo:rustc-link-search={}", libdir.display());
	    println!("cargo:rustc-link-lib=static=uiohook");

	    if target.contains("darwin") {
		    println!("cargo:rustc-link-lib=framework=IOKit");
		    println!("cargo:rustc-link-lib=framework=Carbon");
	    }
	    // println!("cargo:root={}", env::var("OUT_DIR").unwrap());
    } else {
        println!("cargo:rustc-link-lib=uiohook");
    }

}
