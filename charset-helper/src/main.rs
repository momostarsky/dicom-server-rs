use dicom_dictionary_std::tags;
use dicom_encoding::text::{DefaultCharacterSetCodec};

// #[derive(Debug, Snafu)]
// pub enum Error {
//     #[snafu(display("Error opening file: {}", source))]
//     OpenFile {
//         path: String
//
//     },
// }
fn main() -> Result<(), Box<dyn std::error::Error>> {

    let rw ="使其与原始仓库保持一致";

    println!("{}", rw);

    let ver = encoding_rs::GBK.encode(rw);
    let formatted_bytes: Vec<String> = ver.0.iter().map(|b| format!("0x{:02X}", b)).collect();
    println!("{}", formatted_bytes.join(", "));
    println!("{:?}", ver.1);
    // 0x0008,0x0070 Manufacturer
    let iso2022_ir58_bytes = vec![

	0xB0, 0xB2, 0xBB, 0xD5, 0xD0, 0xC7, 0xC1, 0xE9, 0xD0, 0xC5, 0xCF, 0xA2, 0xBF, 0xC6, 0xBC, 0xBC,
	0xD3, 0xD0, 0xCF, 0xDE, 0xB9, 0xAB, 0xCB, 0xBE

    ];
    println!("{:0x?}", iso2022_ir58_bytes);
    println!("{:?}", iso2022_ir58_bytes.len());

    let ds = encoding_rs::GBK.decode(&iso2022_ir58_bytes);
    println!("{}", ds.0);
    println!("{:?}", ds.1);

    let es = encoding_rs::GB18030.decode(&iso2022_ir58_bytes);
    println!("{}", es.0);
    println!("{:?}", es.1);

    // let file = "/home/dhz/jpdata/CDSS/Large/1.dcm";
    let file = "//home/dhz/jpdata/CDSS/DicomTest/zhDicom/ISO-2022-IR58.dcm";
    match dicom_object::OpenFileOptions::new()
        .read_until(tags::PIXEL_DATA)
        .open_file(file)
    {
        Ok(dcm_obj) => {
            let character_set = dcm_obj.element(tags::SPECIFIC_CHARACTER_SET)?;
            println!("File's specific character set: {:?}", character_set.value().to_str() );
            let manufacturer_elm = dcm_obj.element(tags::MANUFACTURER)?;
            let vec = manufacturer_elm.value().to_bytes().unwrap().to_vec();
            println!("{:0x?}", vec);
            println!("{:?}", vec.len());
            let manufacturer_elm_string =   encoding_rs::GBK.decode( &vec);
            println!("{}", manufacturer_elm_string.0);
            println!("{:?}", manufacturer_elm_string.1);

        }
        Err(e) => {
            println!("{:?}", e);
        }
    }

    // let meta = dcm_obj.meta();
    // println!("{:?}", meta);
    Ok(())
}
