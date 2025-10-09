use crate::{
    create_cecho_response, create_cstore_response, dicom_file_handler, transfer::ABSTRACT_SYNTAXES,
    App,
};

use common::message_sender_kafka::KafkaMessagePublisher;
use common::server_config;
use common::utils::get_logger;
use dicom_dictionary_std::tags;
use dicom_encoding::snafu::{OptionExt, Report, ResultExt, Whatever};
use dicom_object::InMemDicomObject;
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use dicom_ul::{pdu::PDataValueType, Pdu};
use slog::o;

use crate::dicom_file_handler::classify_and_publish_dicom_messages;
use slog::{debug, info, warn};

pub async fn run_store_async(
    scu_stream: tokio::net::TcpStream,
    args: &App,
) -> Result<(), Whatever> {
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

    let peer = scu_stream.peer_addr().unwrap();

    let rlogger = get_logger();
    let logger = rlogger.new(o!("storescp"=>"run_store_async"));
    info!(
        logger,
        "New association from remote ip: {} and remote port: {}",
        peer.ip(),
        peer.port()
    );

    let mut instance_buffer: Vec<u8> = Vec::with_capacity(1024 * 1024);
    let mut message_id = 1;
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
        .establish_async(scu_stream)
        .await
        .whatever_context("could not establish association")?;

    info!(
        logger,
        "New association from {}",
        association.client_ae_title()
    );

    debug!(
        logger,
        "> Presentation contexts: {:?}",
        association.presentation_contexts()
    );
    let base_dir = out_dir.to_str().unwrap();

    let app_config = server_config::load_config().whatever_context("failed to load config")?;

    let queue_config = app_config.message_queue;

    let queue_topic_main = &queue_config.topic_main.as_str();
    let queue_topic_change = &queue_config.topic_change_transfer_syntax.as_str();

    let storage_producer = KafkaMessagePublisher::new(queue_topic_main.parse().unwrap());
    let change_producer = KafkaMessagePublisher::new(queue_topic_change.parse().unwrap());

    let mut dicom_message_lists: Vec<common::database_entities::DicomObjectMeta> = vec![];
    loop {
        match association.receive().await {
            Ok(mut pdu) => {
                // if verbose {
                //     debug!("scu ----> scp: {}", pdu.short_description());
                // }
                match pdu {
                    Pdu::PData { ref mut data } => {
                        if data.is_empty() {
                            debug!(logger, "Ignoring empty PData PDU");
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
                                    let cecho_response = create_cecho_response(message_id);
                                    let mut cecho_data = Vec::new();

                                    cecho_response
                                        .write_dataset_with_ts(&mut cecho_data, &ts)
                                        .whatever_context(
                                            "could not write C-ECHO response object",
                                        )?;

                                    let pdu_response = Pdu::PData {
                                        data: vec![dicom_ul::pdu::PDataValue {
                                            presentation_context_id: data_value
                                                .presentation_context_id,
                                            value_type: PDataValueType::Command,
                                            is_last: true,
                                            data: cecho_data,
                                        }],
                                    };
                                    association.send(&pdu_response).await.whatever_context(
                                        "failed to send C-ECHO response object to SCU",
                                    )?;
                                } else {
                                    message_id = obj
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
                                    issue_patient_id = "1234567890".to_string();
                                    // obj
                                    //     .element(tags::ISSUER_OF_PATIENT_ID)
                                    //     .whatever_context("missing ISSUER_OF_PATIENT_ID")?
                                    //     .to_str()
                                    //     .whatever_context(
                                    //         "could not retrieve ISSUER_OF_PATIENT_ID",
                                    //     )?
                                    //     .trim_end_matches("\0")
                                    //     .to_string();
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
                                            logger,
                                            "Successfully processed DICOM file for SOP instance {}",
                                            sop_instance_uid
                                        );
                                        // 继续执行后续操作（发送C-STORE响应等）
                                    }
                                    Err(e) => {
                                        warn!(
                                            logger,
                                            "Failed to process DICOM file for SOP instance {}: {}",
                                            sop_instance_uid,
                                            e
                                        );
                                        // 可以选择是否继续执行后续操作，或者返回错误
                                        // 根据业务需求决定如何处理
                                    }
                                }
                                if dicom_message_lists.len() >= 10 {
                                    match classify_and_publish_dicom_messages(
                                        &dicom_message_lists,
                                        &storage_producer,
                                        &change_producer,
                                       
                                        queue_topic_main,
                                        queue_topic_change,
                                    )
                                    .await
                                    {
                                        Ok(_) => {
                                            info!(&logger, "Successfully published DICOM messages");
                                        }
                                        Err(e) => {
                                            warn!(
                                                &logger,
                                                "Failed to publish DICOM messages: {}", e
                                            );
                                        }
                                    };
                                    dicom_message_lists.clear();
                                }

                                // send C-STORE-RSP object
                                // commands are always in implicit VR LE
                                let ts =
                                    dicom_transfer_syntax_registry::entries::IMPLICIT_VR_LITTLE_ENDIAN
                                        .erased();

                                let obj = create_cstore_response(
                                    message_id,
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
                                    .await
                                    .whatever_context("failed to send response object to SCU")?;
                            }
                        }
                    }
                    Pdu::ReleaseRQ => {
                        association.send(&Pdu::ReleaseRP).await.unwrap_or_else(|e| {
                            warn!(
                                logger,
                                "Failed to send association release message to SCU: {}",
                                Report::from_error(e)
                            );
                        });
                        info!(
                            logger,
                            "Released association with {}",
                            association.client_ae_title()
                        );
                        break;
                    }
                    Pdu::AbortRQ { source } => {
                        warn!(logger, "Aborted connection from: {:?}", source);
                        break;
                    }
                    _ => {}
                }
            }
            Err(err @ dicom_ul::association::Error::ReceivePdu { .. }) => {
                if verbose {
                    info!(logger, "{}", Report::from_error(err));
                } else {
                    info!(logger, "{}", err);
                }
                break;
            }
            Err(err) => {
                warn!(logger, "Unexpected error: {}", Report::from_error(err));
                break;
            }
        }
    }

    if let Ok(peer_addr) = association.inner_stream().peer_addr() {
        info!(
            logger,
            "Dropping connection with {} ({})",
            association.client_ae_title(),
            peer_addr
        );
    } else {
        info!(
            logger,
            "Dropping connection with {}",
            association.client_ae_title()
        );
    }
    if !dicom_message_lists.is_empty() {
        info!(
            &logger,
            "Finished processing association with {}",
            association.client_ae_title()
        );
        match classify_and_publish_dicom_messages(
            &dicom_message_lists,
            &storage_producer,
            &change_producer,
            queue_topic_main,
            queue_topic_change,
        )
        .await
        {
            Ok(_) => {
                info!(&logger, "Successfully published DICOM messages");
            }
            Err(e) => {
                warn!(&logger, "Failed to publish DICOM messages: {}", e);
            }
        };
    }

    Ok(())
}
