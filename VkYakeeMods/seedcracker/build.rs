fn main() {
    println!("cargo:rerun-if-changed=c_src/cubiomes");
    println!("cargo:rerun-if-changed=assets/tables");

    cc::Build::new()
        .include("c_src/cubiomes")
        .include("c_src/cubiomes/tables")
        .file("c_src/cubiomes/biomenoise.c")
        .file("c_src/cubiomes/biomes.c")
        .file("c_src/cubiomes/finders.c")
        .file("c_src/cubiomes/generator.c")
        .file("c_src/cubiomes/layers.c")
        .file("c_src/cubiomes/noise.c")
        .file("c_src/cubiomes/quadbase.c")
        .file("c_src/cubiomes/util.c")
        .flag_if_supported("-fwrapv")
        .flag_if_supported("-O3")
        .compile("cubiomes");
}
