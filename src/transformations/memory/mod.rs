pub mod memory_transformer;
mod visitors;
mod memory_encryption;

#[derive(Eq, PartialEq, Hash, Debug)]
enum MemEncFuncType {
    Unsigned8,  // 1 byte
    Unsigned16, // 2 bytes
    // Unsigned32, // 4 bytes
    // Unsigned64, // don't think it exists
    Signed8,  // 1 byte
    Signed16, // 2 bytes
    Signed32, // 4 bytes
    Signed64, // 8 bytes

    Float32,
    Float64,
}