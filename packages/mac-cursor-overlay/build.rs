extern crate napi_build;

fn main() {
  napi_build::setup();
  println!("cargo:rerun-if-changed=src/overlay.m");

  cc::Build::new()
    .file("src/overlay.m")
    .flag("-fobjc-arc")
    .flag("-fblocks")
    .compile("mac_cursor_overlay_objc");

  println!("cargo:rustc-link-lib=framework=AppKit");
  println!("cargo:rustc-link-lib=framework=Foundation");
  println!("cargo:rustc-link-lib=framework=QuartzCore");
  println!("cargo:rustc-link-lib=framework=CoreGraphics");
  println!("cargo:rustc-link-lib=objc");
}
