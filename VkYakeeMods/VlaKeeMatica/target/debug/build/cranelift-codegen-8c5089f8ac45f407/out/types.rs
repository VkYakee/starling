/// An integer type with 8 bits.
/// WARNING: arithmetic on 8bit integers is incomplete
pub const I8: Type = Type(0x74);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// An integer type with 16 bits.
/// WARNING: arithmetic on 16bit integers is incomplete
pub const I16: Type = Type(0x75);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// An integer type with 32 bits.
pub const I32: Type = Type(0x76);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// An integer type with 64 bits.
pub const I64: Type = Type(0x77);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// An integer type with 128 bits.
pub const I128: Type = Type(0x78);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A 16-bit floating point type represented in the IEEE 754-2008
/// *binary16* interchange format. This corresponds to the :c:type:`_Float16`
/// type in most C implementations.
/// WARNING: f16 support is a work-in-progress and is incomplete
pub const F16: Type = Type(0x79);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A 32-bit floating point type represented in the IEEE 754-2008
/// *binary32* interchange format. This corresponds to the :c:type:`float`
/// type in most C implementations.
pub const F32: Type = Type(0x7a);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A 64-bit floating point type represented in the IEEE 754-2008
/// *binary64* interchange format. This corresponds to the :c:type:`double`
/// type in most C implementations.
pub const F64: Type = Type(0x7b);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A 128-bit floating point type represented in the IEEE 754-2008
/// *binary128* interchange format. This corresponds to the :c:type:`_Float128`
/// type in most C implementations.
/// WARNING: f128 support is a work-in-progress and is incomplete
pub const F128: Type = Type(0x7c);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 2 lanes containing a `i8` each.
pub const I8X2: Type = Type(0x84);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 2 lanes containing `i8` bits each.
pub const I8X2XN: Type = Type(0x104);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 4 lanes containing a `i8` each.
pub const I8X4: Type = Type(0x94);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 2 lanes containing a `i16` each.
pub const I16X2: Type = Type(0x85);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 2 lanes containing a `f16` each.
pub const F16X2: Type = Type(0x89);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 4 lanes containing `i8` bits each.
pub const I8X4XN: Type = Type(0x114);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 2 lanes containing `i16` bits each.
pub const I16X2XN: Type = Type(0x105);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 2 lanes containing `f16` bits each.
pub const F16X2XN: Type = Type(0x109);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 8 lanes containing a `i8` each.
pub const I8X8: Type = Type(0xa4);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 4 lanes containing a `i16` each.
pub const I16X4: Type = Type(0x95);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 2 lanes containing a `i32` each.
pub const I32X2: Type = Type(0x86);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 4 lanes containing a `f16` each.
pub const F16X4: Type = Type(0x99);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 2 lanes containing a `f32` each.
pub const F32X2: Type = Type(0x8a);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 8 lanes containing `i8` bits each.
pub const I8X8XN: Type = Type(0x124);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 4 lanes containing `i16` bits each.
pub const I16X4XN: Type = Type(0x115);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 2 lanes containing `i32` bits each.
pub const I32X2XN: Type = Type(0x106);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 4 lanes containing `f16` bits each.
pub const F16X4XN: Type = Type(0x119);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 2 lanes containing `f32` bits each.
pub const F32X2XN: Type = Type(0x10a);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 16 lanes containing a `i8` each.
pub const I8X16: Type = Type(0xb4);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 8 lanes containing a `i16` each.
pub const I16X8: Type = Type(0xa5);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 4 lanes containing a `i32` each.
pub const I32X4: Type = Type(0x96);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 2 lanes containing a `i64` each.
pub const I64X2: Type = Type(0x87);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 8 lanes containing a `f16` each.
pub const F16X8: Type = Type(0xa9);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 4 lanes containing a `f32` each.
pub const F32X4: Type = Type(0x9a);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 2 lanes containing a `f64` each.
pub const F64X2: Type = Type(0x8b);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 16 lanes containing `i8` bits each.
pub const I8X16XN: Type = Type(0x134);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 8 lanes containing `i16` bits each.
pub const I16X8XN: Type = Type(0x125);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 4 lanes containing `i32` bits each.
pub const I32X4XN: Type = Type(0x116);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 2 lanes containing `i64` bits each.
pub const I64X2XN: Type = Type(0x107);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 8 lanes containing `f16` bits each.
pub const F16X8XN: Type = Type(0x129);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 4 lanes containing `f32` bits each.
pub const F32X4XN: Type = Type(0x11a);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 2 lanes containing `f64` bits each.
pub const F64X2XN: Type = Type(0x10b);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 32 lanes containing a `i8` each.
pub const I8X32: Type = Type(0xc4);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 16 lanes containing a `i16` each.
pub const I16X16: Type = Type(0xb5);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 8 lanes containing a `i32` each.
pub const I32X8: Type = Type(0xa6);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 4 lanes containing a `i64` each.
pub const I64X4: Type = Type(0x97);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 2 lanes containing a `i128` each.
pub const I128X2: Type = Type(0x88);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 16 lanes containing a `f16` each.
pub const F16X16: Type = Type(0xb9);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 8 lanes containing a `f32` each.
pub const F32X8: Type = Type(0xaa);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 4 lanes containing a `f64` each.
pub const F64X4: Type = Type(0x9b);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 2 lanes containing a `f128` each.
pub const F128X2: Type = Type(0x8c);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 32 lanes containing `i8` bits each.
pub const I8X32XN: Type = Type(0x144);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 16 lanes containing `i16` bits each.
pub const I16X16XN: Type = Type(0x135);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 8 lanes containing `i32` bits each.
pub const I32X8XN: Type = Type(0x126);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 4 lanes containing `i64` bits each.
pub const I64X4XN: Type = Type(0x117);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 2 lanes containing `i128` bits each.
pub const I128X2XN: Type = Type(0x108);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 16 lanes containing `f16` bits each.
pub const F16X16XN: Type = Type(0x139);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 8 lanes containing `f32` bits each.
pub const F32X8XN: Type = Type(0x12a);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 4 lanes containing `f64` bits each.
pub const F64X4XN: Type = Type(0x11b);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 2 lanes containing `f128` bits each.
pub const F128X2XN: Type = Type(0x10c);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 64 lanes containing a `i8` each.
pub const I8X64: Type = Type(0xd4);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 32 lanes containing a `i16` each.
pub const I16X32: Type = Type(0xc5);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 16 lanes containing a `i32` each.
pub const I32X16: Type = Type(0xb6);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 8 lanes containing a `i64` each.
pub const I64X8: Type = Type(0xa7);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 4 lanes containing a `i128` each.
pub const I128X4: Type = Type(0x98);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 32 lanes containing a `f16` each.
pub const F16X32: Type = Type(0xc9);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 16 lanes containing a `f32` each.
pub const F32X16: Type = Type(0xba);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 8 lanes containing a `f64` each.
pub const F64X8: Type = Type(0xab);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A SIMD vector with 4 lanes containing a `f128` each.
pub const F128X4: Type = Type(0x9c);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 64 lanes containing `i8` bits each.
pub const I8X64XN: Type = Type(0x154);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 32 lanes containing `i16` bits each.
pub const I16X32XN: Type = Type(0x145);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 16 lanes containing `i32` bits each.
pub const I32X16XN: Type = Type(0x136);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 8 lanes containing `i64` bits each.
pub const I64X8XN: Type = Type(0x127);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 4 lanes containing `i128` bits each.
pub const I128X4XN: Type = Type(0x118);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 32 lanes containing `f16` bits each.
pub const F16X32XN: Type = Type(0x149);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 16 lanes containing `f32` bits each.
pub const F32X16XN: Type = Type(0x13a);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 8 lanes containing `f64` bits each.
pub const F64X8XN: Type = Type(0x12b);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
/// A dynamically-scaled SIMD vector with a minimum of 4 lanes containing `f128` bits each.
pub const F128X4XN: Type = Type(0x11c);
 // /data/data/com.termux/files/home/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cranelift-codegen-meta-0.135.8/src/gen_types.rs:19
