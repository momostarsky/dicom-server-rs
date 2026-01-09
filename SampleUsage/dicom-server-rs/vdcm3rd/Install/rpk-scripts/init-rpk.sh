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

# 定义要创建的主题列表
TOPICS=("dicom_image_queue" "dicom_state_queue" "log_queue" "storage_queue" "webapi_access_queue")

# 检查并创建主题
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

# 等待 Redpanda 进程
wait $REDPANDA_PID