use dicom_dictionary_std::tags;
use dicom_object::collector::CharacterSetOverride;

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


    println!("ISO_192");
    let iso192_bytes = vec![
	0xE8, 0x84, 0x8A, 0xE6, 0x9F, 0xB1, 0xE4, 0xBE, 0xA7, 0xE5, 0xBC, 0xAF, 0x2D, 0xE8, 0xA7, 0x86,
	0xE5, 0x9B, 0xBE, 0x20  ];

    println!("{:0x?}", iso192_bytes);
    println!("{:?}", iso192_bytes.len());

    let ds = encoding_rs::UTF_8.decode(&iso192_bytes);
    println!("{}", ds.0);
    println!("{:?}", ds.1);
    println!("ISO_192");
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
    // let file = "/home/dhz/jpdata/CDSS/DicomTest/zhDicom/ISO-2022-IR58.dcm";
    let file = "/home/dhz/jpdata/CDSS/DicomTest/zhDicom2/1.dcm";
    match dicom_object::OpenFileOptions::new()
        .charset_override(CharacterSetOverride::AnyVr)
        .read_until(tags::PIXEL_DATA)
        .open_file(file)

    {
        Ok(dcm_obj) => {
            let character_set = dcm_obj.element(tags::SPECIFIC_CHARACTER_SET)?;
            println!("File's specific character set: {:?}", character_set.value().to_str() );
            let body_part_examined_elm = dcm_obj.element(tags::BODY_PART_EXAMINED)?;
            let body_part_examined = body_part_examined_elm.value().to_str();
            println!("Body part examined: {:?}", body_part_examined); 
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }

    // let meta = dcm_obj.meta();
    // println!("{:?}", meta);
    Ok(())
}
