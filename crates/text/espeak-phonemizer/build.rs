use cmake;

fn main() {
    println!("cargo:rerun-if-changed=../../../deps/espeak-ng/src");
    println!("cargo:rustc-link-lib=static=espeak-ng");
    println!("cargo:rustc-link-lib=static=ucd");

    let build_dir = cmake::Config::new("../../../deps/espeak-ng")
        .configure_arg("-DUSE_ASYNC:BOOL=OFF")
        .configure_arg("-DUSE_MBROLA:BOOL=OFF")
        .configure_arg("-DUSE_LIBSONIC:BOOL=OFF")
        .configure_arg("-DUSE_LIBPCAUDIO:BOOL=OFF")
        .configure_arg("-DUSE_KLATT:BOOL=OFF")
        .configure_arg("-DUSE_SPEECHPLAYER:BOOL=OFF")
        .configure_arg("-DBUILD_SHARED_LIBS:BOOL=OFF")
        .build();

    println!(
        r"cargo:rustc-link-search={}",
        build_dir.join("lib").display()
    );

    // ucd wordt WEL gebouwd (add_library(ucd STATIC ...) in src/ucd-tools/CMakeLists.txt)
    // maar NIET geinstalleerd: espeak-ng heeft alleen install(TARGETS espeak-ng LIBRARY).
    // libucd.a / ucd.lib blijft dus in de build-boom achter en staat nooit in lib/.
    //
    // Zonder deze extra zoekpaden: "could not find native static library `ucd`".
    //
    // De cmake-crate bouwt in {build_dir}/build/. Waar ucd precies landt hangt af van
    // de generator: multi-config (Visual Studio) zet hem in een Release/-submap,
    // single-config (Ninja, Make) direct in de doelmap. We geven ze allemaal op --
    // een niet-bestaand zoekpad is onschadelijk.
    let ucd = build_dir.join("build").join("src").join("ucd-tools");
    for pad in [
        ucd.clone(),                    // Ninja / Make
        ucd.join("Release"),            // Visual Studio (multi-config)
        ucd.join("RelWithDebInfo"),
        build_dir.join("build"),        // vangnet
    ] {
        println!(r"cargo:rustc-link-search=native={}", pad.display());
    }
}
