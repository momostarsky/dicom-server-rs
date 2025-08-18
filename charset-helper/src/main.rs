use dicom_dictionary_std::tags;
use dicom_encoding::text::{DefaultCharacterSetCodec, TextCodec};
use dicom_object::DefaultDicomObject;

// #[derive(Debug, Snafu)]
// pub enum Error {
//     #[snafu(display("Error opening file: {}", source))]
//     OpenFile {
//         path: String
//
//     },
// }
fn main() -> Result<(), Box<dyn std::error::Error>> {


    let file = "/home/dhz/jpdata/CDSS/Large/1.dcm";
    match dicom_object::OpenFileOptions::new()
        .read_until(tags::PIXEL_DATA)
        .open_file(file)
    {
        Ok(dcm_obj) => {

            use dicom_encoding::text::{SpecificCharacterSet, TextCodec};

            let character_set = SpecificCharacterSet::from_code("ISO_IR 192").unwrap();
            assert_eq!(character_set, SpecificCharacterSet::ISO_IR_192);
            println!("File's specific character set: {:?}", character_set );
            let body_part_element = dcm_obj.element(tags::BODY_PART_EXAMINED)?;
            let body_part_bytes = body_part_element.value().to_bytes()?;
            // 2. Get the corresponding text codec from the character set.
            //    `dicom-rs` handles the mapping from "ISO_IR 192" to a UTF-8 decoder.
            let body_part_string = character_set.decode(&*body_part_bytes).unwrap();

            println!("---");
            println!("Raw bytes of Body Part Examined: {:02X?}", body_part_bytes);

            let a2 = DefaultCharacterSetCodec.decode(&*body_part_bytes).unwrap();

            println!("Decoded string: {}", a2);
            println!("Correctly decoded string: {}", body_part_string);
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }

    // let meta = dcm_obj.meta();
    // println!("{:?}", meta);
    Ok(())
}
