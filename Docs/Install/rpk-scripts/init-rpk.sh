#!/bin/bash

set -e

echo "Starting Redpanda initialization..."

# 启动 Redpanda 服务
rpk redpanda start \
      --kafka-addr internal://0.0.0.0:9092,external://0.0.0.0:19092 \
      --advertise-kafka-addr internal://redpanda:9092,external://localhost:19092 \
      --mode dev-container \
      --smp 1 &
REDPANDA_PID=$!

# 等待服务启动
echo "Waiting for Redpanda..."
until rpk cluster info --brokers localhost:9092 &> /dev/null; do
  echo "waiting Redpanda Kafka API..."
  sleep 5
done

echo "Creating topics..."
rpk topic create dicom_image_queue --partitions 1 --replicas 1 --brokers localhost:9092
rpk topic create dicom_state_queue --partitions 1 --replicas 1 --brokers localhost:9092
rpk topic create log_queue --partitions 1 --replicas 1 --brokers localhost:9092
rpk topic create storage_queue --partitions 1 --replicas 1 --brokers localhost:9092
rpk topic create webapi_access_queue --partitions 1 --replicas 1 --brokers localhost:9092

echo "Topics created completed"

# 等待 Redpanda 进程
wait $REDPANDA_PID