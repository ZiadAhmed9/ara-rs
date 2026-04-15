fn main() {
    cxx_build::bridge("src/lib.rs")
        .file("src/main.cpp")
        .std("c++17")
        .compile("cxx_bridge_example");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/main.cpp");
    println!("cargo:rerun-if-changed=src/cxx_client.h");
}
