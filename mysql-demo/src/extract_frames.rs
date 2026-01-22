use crate::extract_frames::ExtractError::{FrameOutOfBounds, MissingTag, TagValueUnExcepted};
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
    #[allow(dead_code)]
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
    #[snafu(display("could not write DICOM file {}", path.display()))]
    WriteFile {
        #[snafu(source(from(dicom_object::WriteError, Box::new)))]
        source: Box<dicom_object::WriteError>,
        path: PathBuf,
    },
    /// missing offset table entry for frame #{frame_number}
    MissingOffsetEntry {
        frame_number: u32,
    },
    /// missing tag {tag}
    MissingTag {
        tag: Tag,
    },
    /// tag value is unexpected
    TagValueUnExcepted {
        tag: Tag,
        value: String,
    },
    TagValueLengthOutofBound {
        tag: Tag,
        value: String,
    },
    /// missing key property {name}
    MissingProperty {
        name: &'static str,
    },
    /// property {name} contains an invalid value
    InvalidPropertyValue {
        name: &'static str,
        #[snafu(source(from(dicom_core::value::ConvertValueError, Box::new)))]
        source: Box<dicom_core::value::ConvertValueError>,
    },
    /// pixel data of frame #{frame_number} is out of bounds
    #[snafu(display("pixel data of frame #{} is out of bounds", frame_number))]
    FrameOutOfBounds {
        frame_number: u32,
    },
    /// Unexpected DICOM pixel data as data set sequence
    UnexpectedPixelData,
    /// No files given
    NoFiles,
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
            tags::SOP_INSTANCE_UID,
            tags::PHOTOMETRIC_INTERPRETATION,
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

        let pixel_element = dicom_obj
            .get(tags::PIXEL_DATA)
            .context(MissingPropertySnafu { name: "PIXEL_DATA" })?;
        let pixel_vr = pixel_element.vr(); // 获取实际的VR类型（VR::OB 或 VR::OW）

        let pixeldata = pixel_element.value();
        let rows = get_int_property(tags::ROWS, "Rows")?;
        let columns = get_int_property(tags::COLUMNS, "Columns")?;
        let samples_per_pixel = get_int_property(tags::SAMPLES_PER_PIXEL, "Samples Per Pixel")?;
        let bits_allocated = get_int_property(tags::BITS_ALLOCATED, "Bits Allocated")?;
        let frame_size = rows * columns * samples_per_pixel * ((bits_allocated + 7) / 8);
        println!("Pixel VR: {:?}", pixel_vr);
        let mut sop_uid = dicom_gen_uid::gen_uid();
        if sop_uid.len() > 60 {
            sop_uid = sop_uid[0..60].to_string();
        }
        for frame_number in 0..num_frames {
            let out_data = match pixeldata {
                DicomValue::PixelSequence(seq) => {
                    let number_of_frames = num_frames;

                    if number_of_frames as usize == seq.fragments().len() {
                        // frame-to-fragment mapping is 1:1

                        // get fragment containing our frame
                        // let fragment = seq.fragments().get(frame_number as usize).context( FrameOutOfBounds { frame_number: frame_number as u32})?;

                        let fragment =
                            seq.fragments()
                                .get(frame_number as usize)
                                .ok_or(FrameOutOfBounds {
                                    frame_number: frame_number as u32,
                                })?;

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
                PrimitiveValue::from(format!("{}.{:04}", sop_uid, frame_number + 1)),
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
                "{}/{}.{:04}.dcm",
                self.output_file_dir,
                sop_uid,
                frame_number + 1
            );
            // 这里不再需要显式指定字典类型，因为write_file方法会自动处理
            // TODO: out_obj 写入磁盘
            // write the DICOM file to disk
            // 修正部分：直接调用 out_obj 实例的 write_to_file 方法
            out_obj
                .write_to_file(&out_file_path)
                .context(WriteFileSnafu {
                    path: out_file_path,
                })?;
        }
        Ok(())
    }
}
