#!/bin/bash

set -e

echo "Starting Redpanda initialization..."
IPADDR=$(hostname -I | awk '{print $1}')
echo "Host IP: $IPADDR"
ADVERTISE_IP=${HOST_IP:-${IPADDR:-localhost}}
echo "Detected Container IP: $IPADDR"
echo "Final Advertised IP: $ADVERTISE_IP"
# start redpanda service
# shellcheck disable=SC1073
rpk redpanda start \
      --kafka-addr internal://0.0.0.0:9092,external://0.0.0.0:19092 \
      --advertise-kafka-addr internal://redpanda:9092,external://"${ADVERTISE_IP}":19092 \
      --mode dev-container \
      --smp 1 &
REDPANDA_PID=$!

# wait for redpanda to started...
echo "Waiting for Redpanda..."
until rpk cluster info --brokers localhost:9092 &> /dev/null; do
  echo "waiting Redpanda Kafka API..."
  sleep 5
done

echo "Creating topics..."

# topices to be create.
TOPICS=("dicom_image_queue" "dicom_state_queue" "log_queue" "storage_queue" "webapi_access_queue")

# check and create topics
for topic in "${TOPICS[@]}"; do
  if rpk topic list --brokers localhost:9092 | grep -q "^$topic "; then
    echo "Topic '$topic' already exists, skipping..."
  else
    echo "Creating topic '$topic'..."
    rpk topic create "$topic" --partitions 1 --replicas 1 --brokers localhost:9092
    echo "Topic '$topic' created"
  fi
done

echo "Topics created completed"

# waiting to redpanda process exit
wait $REDPANDA_PID