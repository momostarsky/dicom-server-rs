
use crate::{
    create_cecho_response, create_cstore_response, dicom_file_handler, transfer::ABSTRACT_SYNTAXES,
    App,
};
use dicom_core::chrono::Local;
use dicom_dictionary_std::tags;
use dicom_object::InMemDicomObject;
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use dicom_ul::{pdu::PDataValueType, Pdu};
use snafu::{OptionExt, Report, ResultExt, Whatever};
use std::net::TcpStream;
use tracing::log::error;
use tracing::{debug, info, warn};
use common::kafka_producer_factory;
use common::kafka_producer_factory::KafkaProducer;

pub async fn run_store_sync(scu_stream: TcpStream, args: &App) -> Result<(), Whatever> {
    let App {
        verbose,
        calling_ae_title,
        strict,
        uncompressed_only,
        promiscuous,
        max_pdu_length,
        out_dir,
        json_store_path: _json_store_path,
        port: _port,
        non_blocking: _non_blocking,
    } = args;
    let verbose = *verbose;

    let mut instance_buffer: Vec<u8> = Vec::with_capacity(1024 * 1024);
    let mut msgid = 1;
    let mut sop_class_uid = "".to_string();
    let mut sop_instance_uid = "".to_string();
    let mut issue_patient_id = "".to_string();
    let mut options = dicom_ul::association::ServerAssociationOptions::new()
        .accept_any()
        .ae_title(calling_ae_title)
        .strict(*strict)
        .max_pdu_length(*max_pdu_length)
        .promiscuous(*promiscuous);

    if *uncompressed_only {
        options = options
            .with_transfer_syntax("1.2.840.10008.1.2")
            .with_transfer_syntax("1.2.840.10008.1.2.1");
    } else {
        for ts in TransferSyntaxRegistry.iter() {
            if !ts.is_unsupported() {
                options = options.with_transfer_syntax(ts.uid());
            }
        }
    };

    for uid in ABSTRACT_SYNTAXES {
        options = options.with_abstract_syntax(*uid);
    }

    let mut association = options
        .establish(scu_stream)
        .whatever_context("could not establish association")?;

    info!("New association from {}", association.client_ae_title());
    debug!(
        "> Presentation contexts: {:?}",
        association.presentation_contexts()
    );
    let base_dir = out_dir.to_str().unwrap();
    let kafka_producer = kafka_producer_factory::create_main_kafka_producer();
    let mut dicom_message_lists = vec![];
    loop {
        match association.receive() {
            Ok(mut pdu) => {
                // if verbose {
                //     debug!("scu ----> scp: {}", pdu.short_description());
                // }
                match pdu {
                    Pdu::PData { ref mut data } => {
                        if data.is_empty() {
                            debug!("Ignoring empty PData PDU");
                            continue;
                        }

                        for data_value in data {
                            if data_value.value_type == PDataValueType::Data && !data_value.is_last
                            {
                                instance_buffer.append(&mut data_value.data);
                            } else if data_value.value_type == PDataValueType::Command
                                && data_value.is_last
                            {
                                // commands are always in implicit VR LE
                                let ts =
                                    dicom_transfer_syntax_registry::entries::IMPLICIT_VR_LITTLE_ENDIAN
                                        .erased();
                                let data_value = &data_value;
                                let v = &data_value.data;

                                let obj = InMemDicomObject::read_dataset_with_ts(v.as_slice(), &ts)
                                    .whatever_context("failed to read incoming DICOM command")?;
                                let command_field = obj
                                    .element(tags::COMMAND_FIELD)
                                    .whatever_context("Missing Command Field")?
                                    .uint16()
                                    .whatever_context("Command Field is not an integer")?;

                                if command_field == 0x0030 {
                                    // Handle C-ECHO-RQ
                                    let echo_response = create_cecho_response(msgid);
                                    let mut echo_data = Vec::new();

                                    echo_response
                                        .write_dataset_with_ts(&mut echo_data, &ts)
                                        .whatever_context(
                                            "could not write C-ECHO response object",
                                        )?;

                                    let pdu_response = Pdu::PData {
                                        data: vec![dicom_ul::pdu::PDataValue {
                                            presentation_context_id: data_value
                                                .presentation_context_id,
                                            value_type: PDataValueType::Command,
                                            is_last: true,
                                            data: echo_data,
                                        }],
                                    };
                                    association.send(&pdu_response).whatever_context(
                                        "failed to send C-ECHO response object to SCU",
                                    )?;
                                } else {
                                    msgid = obj
                                        .element(tags::MESSAGE_ID)
                                        .whatever_context("Missing Message ID")?
                                        .to_int()
                                        .whatever_context("Message ID is not an integer")?;
                                    sop_class_uid = obj
                                        .element(tags::AFFECTED_SOP_CLASS_UID)
                                        .whatever_context("missing Affected SOP Class UID")?
                                        .to_str()
                                        .whatever_context(
                                            "could not retrieve Affected SOP Class UID",
                                        )?
                                        .to_string();
                                    sop_instance_uid = obj
                                        .element(tags::AFFECTED_SOP_INSTANCE_UID)
                                        .whatever_context("missing Affected SOP Instance UID")?
                                        .to_str()
                                        .whatever_context(
                                            "could not retrieve Affected SOP Instance UID",
                                        )?
                                        .to_string();

                                    issue_patient_id = obj
                                        .element(tags::ISSUER_OF_PATIENT_ID)
                                        .whatever_context("missing ISSUER_OF_PATIENT_ID")?
                                        .to_str()
                                        .whatever_context(
                                            "could not retrieve ISSUER_OF_PATIENT_ID",
                                        )?
                                        .to_string();
                                }
                                instance_buffer.clear();
                            } else if data_value.value_type == PDataValueType::Data
                                && data_value.is_last
                            {
                                instance_buffer.append(&mut data_value.data);

                                let presentation_context = association
                                    .presentation_contexts()
                                    .iter()
                                    .find(|pc| pc.id == data_value.presentation_context_id)
                                    .whatever_context("missing presentation context")?;
                                let ts = &presentation_context.transfer_syntax;

                                // let obj = InMemDicomObject::read_dataset_with_ts(
                                //     instance_buffer.as_slice(),
                                //     TransferSyntaxRegistry.get(ts).unwrap(),
                                // )
                                // .whatever_context("failed to read DICOM data object")?;

                                match dicom_file_handler::process_dicom_file(
                                    &instance_buffer,
                                    base_dir,
                                    &issue_patient_id,
                                    ts,
                                    &sop_instance_uid,
                                    &mut dicom_message_lists,
                                )
                                .await
                                {
                                    Ok(_) => {
                                        info!(
                                            "Successfully processed DICOM file for SOP instance {}",
                                            sop_instance_uid
                                        );
                                        // 继续执行后续操作（发送C-STORE响应等）
                                    }
                                    Err(e) => {
                                        warn!(
                                            "Failed to process DICOM file for SOP instance {}: {}",
                                            sop_instance_uid, e
                                        );
                                        // 可以选择是否继续执行后续操作，或者返回错误
                                        // 根据业务需求决定如何处理
                                    }
                                }
                                // match kafka_producer
                                //     .send_message("dicom", &dicom_message_lists[0])
                                //     .await
                                // {
                                //     Ok(_) => {
                                //         info!("Successfully sent messages to Kafka");
                                //     }
                                //     Err(e) => {
                                //         error!("Failed to send messages to Kafka: {}", e);
                                //     }
                                // }
                                if dicom_message_lists.len() >= 10 {
                                    match kafka_producer
                                        .send_batch_messages(  &dicom_message_lists)
                                        .await
                                    {
                                        Ok(_) => {
                                            info!("Successfully sent messages to Kafka");
                                        }
                                        Err(e) => {
                                            error!("Failed to send messages to Kafka: {}", e);
                                        }
                                    }

                                    dicom_message_lists.clear();
                                }

                                // send C-STORE-RSP object
                                // commands are always in implicit VR LE
                                let ts =
                                dicom_transfer_syntax_registry::entries::IMPLICIT_VR_LITTLE_ENDIAN
                                    .erased();

                                let obj = create_cstore_response(
                                    msgid,
                                    &sop_class_uid,
                                    &sop_instance_uid,
                                );

                                let mut obj_data = Vec::new();

                                obj.write_dataset_with_ts(&mut obj_data, &ts)
                                    .whatever_context("could not write response object")?;

                                let pdu_response = Pdu::PData {
                                    data: vec![dicom_ul::pdu::PDataValue {
                                        presentation_context_id: data_value.presentation_context_id,
                                        value_type: PDataValueType::Command,
                                        is_last: true,
                                        data: obj_data,
                                    }],
                                };
                                association
                                    .send(&pdu_response)
                                    .whatever_context("failed to send response object to SCU")?;
                            }
                        }
                    }
                    Pdu::ReleaseRQ => {
                        association.send(&Pdu::ReleaseRP).unwrap_or_else(|e| {
                            warn!(
                                "Failed to send association release message to SCU: {}",
                                snafu::Report::from_error(e)
                            );
                        });
                        info!(
                            "Released association with {}",
                            association.client_ae_title()
                        );
                        break;
                    }
                    Pdu::AbortRQ { source } => {
                        warn!("Aborted connection from: {:?}", source);
                        break;
                    }
                    _ => {}
                }
            }
            Err(err @ dicom_ul::association::server::Error::Receive { .. }) => {
                if verbose {
                    info!("{}", Report::from_error(err));
                } else {
                    info!("{}", err);
                }
                break;
            }
            Err(err) => {
                warn!("Unexpected error: {}", Report::from_error(err));
                break;
            }
        }
    }


    if let Ok(peer_addr) = association.inner_stream().peer_addr() {
        info!(
            "Dropping connection with {} ({})",
            association.client_ae_title(),
            peer_addr
        );
    } else {
        info!("Dropping connection with {}", association.client_ae_title());
    }

    dicom_file_handler::sendmessage_to_kafka(&kafka_producer, &dicom_message_lists).await;
    Ok(())
}
