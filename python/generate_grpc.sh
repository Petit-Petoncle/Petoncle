#!/bin/bash
# Generate Python gRPC stubs from proto file

cd "$(dirname "$0")"

echo "Generating gRPC code from proto/chat.proto..."

python -m grpc_tools.protoc \
    -I./proto \
    --python_out=./proto \
    --grpc_python_out=./proto \
    ./proto/chat.proto

echo "✓ Generated chat_pb2.py and chat_pb2_grpc.py"
