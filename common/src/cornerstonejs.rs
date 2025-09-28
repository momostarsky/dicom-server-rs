use std::collections::HashSet;
use std::sync::LazyLock;

// cornerstonejs 支持的传输语法
//参考: https://github.com/cornerstonejs/cornerstoneWADOImageLoader/blob/main/src/imageLoader/wadouri/getTransferSyntax.js
pub static SUPPORTED_TRANSFER_SYNTAXES: LazyLock<HashSet<&'static str>> =
    LazyLock::new(supported_transfer_syntaies);
// cornerstonejs 支持的传输语法
// https://github.com/cornerstonejs/cornerstone3D/blob/main/packages/dicomImageLoader/src/imageLoader/decodeImageFrame.ts

/// cornerstonejs 支持的传输语法
/// 参考: https://github.com/cornerstonejs/cornerstoneWADOImageLoader/blob/master/src/imageLoader/decodeImageFrame.js
/// 由于部分手机端不支持JPEG2000，所以这里也不支持JPEG2000
fn supported_transfer_syntaies() -> HashSet<&'static str> {
    vec![
        "1.2.840.10008.1.2",  // Implicit VR Little Endian
        "1.2.840.10008.1.2.1", // Explicit VR Little Endian
        "1.2.840.10008.1.2.2", // Explicit VR Big Endian (retired)
        "1.2.840.10008.1.2.1.99", // Deflate transfer syntax (deflated by dicomParser)
        "1.2.840.10008.1.2.5",    // RLE Lossless
        // "1.2.840.10008.1.2.4.50", // JPEG Baseline lossy process 1 (8 bit)  .bitsAllocated === 8 && samplesPerPixel === 3 || 4
        // "1.2.840.10008.1.2.4.51", // JPEG Baseline lossy process 2 & 4 (12 bit)
        // "1.2.840.10008.1.2.4.57", // JPEG Lossless, Nonhierarchical (Processes 14)
        // "1.2.840.10008.1.2.4.70", // JPEG Lossless, Nonhierarchical (Processes 14 [Selection 1])
        // "1.2.840.10008.1.2.4.80", // JPEG-LS Lossless Image Compression
        // "1.2.840.10008.1.2.4.81", // JPEG-LS Lossy (Near-Lossless) Image Compression
        // "1.2.840.10008.1.2.4.90", // JPEG 2000 Lossless
        // "1.2.840.10008.1.2.4.91", // JPEG 2000 Lossy
        // "1.2.840.10008.1.2.4.96", //  HTJ2K
        // "1.2.840.10008.1.2.4.201",
        // "1.2.840.10008.1.2.4.202",
        // "1.2.840.10008.1.2.4.203",
    ]
    .into_iter()
    .collect()
}
