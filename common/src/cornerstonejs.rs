use std::collections::HashSet;
use std::sync::LazyLock;

// cornerstonejs 支持的传输语法
//参考: https://github.com/cornerstonejs/cornerstoneWADOImageLoader/blob/main/src/imageLoader/wadouri/getTransferSyntax.js
pub static SUPPORTED_TRANSFER_SYNTAXES: LazyLock<HashSet<&'static str>> =
    LazyLock::new(supported_transfer_syntaies);
// cornerstonejs 支持的传输语法
// https://github.com/cornerstonejs/cornerstone3D/blob/main/packages/dicomImageLoader/src/imageLoader/decodeImageFrame.ts
fn supported_transfer_syntaies() -> HashSet<&'static str> {
    vec![
        "1.2.840.10008.1.2",
        "1.2.840.10008.1.2.1",
        "1.2.840.10008.1.2.2",
        "1.2.840.10008.1.2.1.99",
        "1.2.840.10008.1.2.5",
        "1.2.840.10008.1.2.4.50",
        "1.2.840.10008.1.2.4.51",
        "1.2.840.10008.1.2.4.57",
        "1.2.840.10008.1.2.4.70",
        "1.2.840.10008.1.2.4.80",
        "1.2.840.10008.1.2.4.81",
        "1.2.840.10008.1.2.4.90",
        "1.2.840.10008.1.2.4.91",
        "1.2.840.10008.1.2.4.96",
        "1.2.840.10008.1.2.4.201",
        "1.2.840.10008.1.2.4.202",
        "1.2.840.10008.1.2.4.203",
    ]
    .into_iter()
    .collect()
}
