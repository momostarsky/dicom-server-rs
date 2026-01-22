use crate::change_ts::ExtractError::{MissingTag, TagValueUnExcepted};
use common::dicom_utils;
use dicom_core::header::Tag;
use dicom_core::value::{PixelFragmentSequence, PrimitiveValue};
use dicom_core::{DataElement, DicomValue, VR};
use dicom_dictionary_std::tags;
use dicom_object::open_file;
use snafu::{OptionExt, ResultExt, Snafu};
use std::borrow::Cow;
use std::path::PathBuf;

pub struct ExtractMultiFrameToMultiFile {
    pub input_file_path: String,
    pub output_file_dir: String,
    pub change_transfer_syntax: bool,
}
#[derive(Debug, Snafu)]
pub enum ExtractError {
    #[snafu(display("could not read DICOM file {}", path.display()))]
    ReadFile {
        #[snafu(source(from(dicom_object::ReadError, Box::new)))]
        source: Box<dicom_object::ReadError>,
        path: PathBuf,
    },

    /// missing offset table entry for frame #{frame_number}
    MissingOffsetEntry { frame_number: u32 },
    /// missing tag {tag}
    MissingTag { tag: Tag },
    /// tag value is unexpected
    TagValueUnExcepted { tag: Tag, value: String },
    /// missing key property {name}
    MissingProperty { name: &'static str },
    /// property {name} contains an invalid value
    InvalidPropertyValue {
        name: &'static str,
        #[snafu(source(from(dicom_core::value::ConvertValueError, Box::new)))]
        source: Box<dicom_core::value::ConvertValueError>,
    },
    /// pixel data of frame #{frame_number} is out of bounds
    FrameOutOfBounds { frame_number: u32 },
    /// failed to save image to file
    SaveImage {
        #[snafu(source(from(dicom_pixeldata::image::ImageError, Box::new)))]
        source: Box<dicom_pixeldata::image::ImageError>,
    },
    /// failed to save pixel data to file
    SaveData { source: std::io::Error },
    /// Unexpected DICOM pixel data as data set sequence
    UnexpectedPixelData,
    /// No files given
    NoFiles,
    /// Read dir error
    ReadDir { source: std::io::Error },
}

impl ExtractMultiFrameToMultiFile {
    pub fn new(
        input_file_path: String,
        output_file_dir: String,
        change_transfer_syntax: bool,
    ) -> Self {
        ExtractMultiFrameToMultiFile {
            input_file_path,
            output_file_dir,
            change_transfer_syntax,
        }
    }

    pub fn run(&self) -> Result<(), ExtractError> {
        let dicom_obj = open_file(&self.input_file_path).context(ReadFileSnafu {
            path: PathBuf::from(&self.input_file_path),
        })?;

        let tags = vec![
            tags::NUMBER_OF_FRAMES,
            tags::SERIES_INSTANCE_UID,
            tags::PIXEL_DATA,
            tags::ROWS,
            tags::COLUMNS,
            tags::SAMPLES_PER_PIXEL,
            tags::BITS_ALLOCATED,
        ];
        for ctag in tags {
            if !dicom_utils::tag_exists(&dicom_obj, ctag) {
                return Err(MissingTag { tag: ctag });
            }
        }

        let num_frames =
            dicom_utils::get_int_value(&dicom_obj, tags::NUMBER_OF_FRAMES).unwrap_or(0);
        if num_frames <= 1 {
            return Err(TagValueUnExcepted {
                tag: tags::NUMBER_OF_FRAMES,
                value: num_frames.to_string(),
            });
        }

        let get_int_property = |tag, name| {
            dicom_obj
                .get(tag)
                .context(MissingPropertySnafu { name })?
                .to_int::<usize>()
                .context(InvalidPropertyValueSnafu { name })
        };
        let series_uid =
            dicom_utils::get_text_value(&dicom_obj, tags::SERIES_INSTANCE_UID).unwrap();
        let pixel_element = dicom_obj.get(tags::PIXEL_DATA).unwrap();
        let pixel_vr = pixel_element.vr(); // 获取实际的VR类型（VR::OB 或 VR::OW）

        let pixeldata = pixel_element.value();
        let rows = get_int_property(tags::ROWS, "Rows")?;
        let columns = get_int_property(tags::COLUMNS, "Columns")?;
        let samples_per_pixel = get_int_property(tags::SAMPLES_PER_PIXEL, "Samples Per Pixel")?;
        let bits_allocated = get_int_property(tags::BITS_ALLOCATED, "Bits Allocated")?;
        let frame_size = rows * columns * samples_per_pixel * ((bits_allocated + 7) / 8);

        println!("Pixel VR: {:?}", pixel_vr);
        for frame_number in 0..num_frames {
            let out_data = match pixeldata {
                DicomValue::PixelSequence(seq) => {
                    let number_of_frames = num_frames;

                    if number_of_frames as usize == seq.fragments().len() {
                        // frame-to-fragment mapping is 1:1

                        // get fragment containing our frame
                        let fragment = seq.fragments().get(frame_number as usize).unwrap();

                        Cow::Borrowed(&fragment[..])
                    } else {
                        let offset_table = seq.offset_table();
                        let base_offset = offset_table.get(frame_number as usize).copied();
                        let base_offset = if frame_number == 0 {
                            base_offset.unwrap_or(0) as usize
                        } else {
                            base_offset.context(MissingOffsetEntrySnafu {
                                frame_number: frame_number as u32,
                            })? as usize
                        };
                        let next_offset = offset_table.get(frame_number as usize + 1);

                        let mut offset = 0;
                        let mut frame_data = Vec::new();
                        for fragment in seq.fragments() {
                            // include it
                            if offset >= base_offset {
                                frame_data.extend_from_slice(fragment);
                            }
                            offset += fragment.len() + 8;
                            if let Some(&next_offset) = next_offset {
                                if offset >= next_offset as usize {
                                    // next fragment is for the next frame
                                    break;
                                }
                            }
                        }

                        Cow::Owned(frame_data)
                    }
                }
                DicomValue::Primitive(v) => {
                    let frame = frame_number as usize;
                    let mut data = v.to_bytes();
                    match &mut data {
                        Cow::Borrowed(data) => {
                            *data = data
                                .get((frame_size * frame)..(frame_size * (frame + 1)))
                                .unwrap();
                        }
                        Cow::Owned(data) => {
                            *data = data
                                .get((frame_size * frame)..(frame_size * (frame + 1)))
                                .unwrap()
                                .to_vec();
                        }
                    }
                    data
                }
                _ => {
                    return UnexpectedPixelDataSnafu.fail();
                }
            };

            let all_data = match out_data {
                Cow::Borrowed(b) => b.to_vec(),
                Cow::Owned(b) => b,
            };
            let mut out_obj = dicom_obj.clone();
            out_obj.remove_element(tags::SOP_INSTANCE_UID);
            out_obj.remove_element(tags::NUMBER_OF_FRAMES);
            out_obj.remove_element(tags::INSTANCE_NUMBER);
            out_obj.remove_element(tags::PIXEL_DATA);
            out_obj.put(DataElement::new(
                tags::SOP_INSTANCE_UID,
                VR::UI,
                PrimitiveValue::from(format!("{}.000{}", series_uid, frame_number + 1)),
            ));
            out_obj.put(DataElement::new(
                tags::INSTANCE_NUMBER,
                VR::IS,
                PrimitiveValue::from(frame_number + 1),
            ));
            out_obj.put(DataElement::new(
                tags::NUMBER_OF_FRAMES,
                VR::IS,
                PrimitiveValue::from(1),
            ));

            out_obj.put(DataElement::new(
                tags::PIXEL_DATA,
                pixel_vr,
                DicomValue::PixelSequence(PixelFragmentSequence::new_fragments(vec![all_data])),
            ));
            let out_file_path = format!(
                "{}/{}_{}.dcm",
                self.output_file_dir,
                series_uid,
                frame_number + 1
            );
            // 这里不再需要显式指定字典类型，因为write_file方法会自动处理
            //TODO: out_obj 写入磁盘
            // write the DICOM file to disk
            // 修正部分：直接调用 out_obj 实例的 write_to_file 方法
            match out_obj.write_to_file(&out_file_path) {
                Ok(_) => println!("Wrote {}", out_file_path),
                Err(e) => eprintln!("failed to write {}: {}", out_file_path, e),
            }
        }

        Ok(())
    }
}
