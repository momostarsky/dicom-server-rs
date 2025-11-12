# 单机版本测试用途
rpk topic create log_queue         --partitions 1 --replicas 1
rpk topic create storage_queue     --partitions 1 --replicas 1
rpk topic create dicom_state_queue --partitions 1 --replicas 1
rpk topic create dicom_image_queue --partitions 1 --replicas 1
