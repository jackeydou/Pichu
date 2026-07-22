extern crate napi_build;

fn main() {
    napi_build::setup();
    println!("cargo:rustc-link-lib=framework=CoreGraphics");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    println!("cargo:rustc-link-lib=framework=ApplicationServices");
    println!("cargo:rustc-link-lib=framework=ImageIO");
    // Needed for NSWorkspace.shared.frontmostApplication / NSRunningApplication.
    println!("cargo:rustc-link-lib=framework=AppKit");
    // Objective-C runtime for objc_msgSend / objc_getClass / sel_registerName.
    println!("cargo:rustc-link-lib=objc");
}
