extern crate napi_build;

fn main() {
  napi_build::setup();
  println!("cargo:rustc-link-lib=framework=CoreGraphics");
  println!("cargo:rustc-link-lib=framework=CoreFoundation");
}
