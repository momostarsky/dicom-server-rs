use std::fmt;

#[derive(Debug)]
pub enum ExtractionError {
    MissingPatientId,
    EmptyPatientId,
    MissingStudyUid,
    EmptyStudyUid,
    MissingSeriesUid,
    EmptySeriesUid,
    MissingSopUid,
    EmptySopUid,
    MissiingStudyDate,
    EmptyStudyDate,
    MissingModality,
    EmptyModality,
}

impl fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractionError::MissingPatientId => write!(f, "Missing patient ID in DICOM object"),
            ExtractionError::EmptyPatientId => write!(f, "Patient ID is empty in DICOM object"),
            ExtractionError::MissingStudyUid => write!(f, "Missing study UID in DICOM object"),
            ExtractionError::EmptyStudyUid => write!(f, "Study UID is empty in DICOM object"),
            ExtractionError::MissingSeriesUid => write!(f, "Missing series UID in DICOM object"),
            ExtractionError::EmptySeriesUid => write!(f, "Series UID is empty in DICOM object"),
            ExtractionError::MissingSopUid => write!(f, "Missing SOP UID in DICOM object"),
            ExtractionError::EmptySopUid => write!(f, "SOP UID is empty in DICOM object"),
            ExtractionError::MissiingStudyDate => {write!(f, "Missing StudyDate in DICOM object")}
            ExtractionError::EmptyStudyDate => write!(f, "StudyDate is empty in DICOM object"),
            ExtractionError::MissingModality => write!(f, "Missing Modality in DICOM object"),
            ExtractionError::EmptyModality => write!(f, "Modality is empty in DICOM object"),
        }
    }
}

impl std::error::Error for ExtractionError {}
