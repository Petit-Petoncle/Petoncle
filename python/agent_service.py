"""
gRPC server for Petoncle AI agent service
"""
import os
import sys
import logging
from concurrent import futures
import grpc
from dotenv import load_dotenv

# Add proto directory to path for generated modules
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'proto'))

# Import generated gRPC modules (will be generated from proto file)
import chat_pb2
import chat_pb2_grpc

from agents import ChatAgent
from agents.graph import MultiAgentSystem

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='[%(asctime)s] %(levelname)s - %(message)s',
    datefmt='%H:%M:%S'
)
logger = logging.getLogger(__name__)


class ChatServiceServicer(chat_pb2_grpc.ChatServiceServicer):
    """Implementation of ChatService gRPC service"""

    def __init__(self):
        """Initialize the service with multi-agent system"""
        load_dotenv()
        self.agent_system = MultiAgentSystem()
        logger.info("Multi-agent system initialized (Orchestrator + 4 specialists)")

    def SendMessage(self, request, context):
        """
        Handle chat message requests

        Args:
            request: ChatRequest with message and optional context
            context: gRPC context

        Returns:
            ChatResponse with AI response
        """
        try:
            # Validate input
            if not request.message:
                logger.warning("Received empty message")
                context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
                context.set_details("Message cannot be empty")
                return chat_pb2.ChatResponse(message="⚠️ Message cannot be empty", agent="error")

            if len(request.message) > 10000:  # 10K char limit
                logger.warning(f"Message too long: {len(request.message)} chars")
                context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
                context.set_details("Message too long (max 10000 characters)")
                return chat_pb2.ChatResponse(message="⚠️ Message too long (max 10000 characters)", agent="error")

            # Sanitize message (remove null bytes, excessive whitespace)
            sanitized_message = request.message.replace('\x00', '').strip()
            if not sanitized_message:
                logger.warning("Message became empty after sanitization")
                context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
                context.set_details("Message contains invalid characters")
                return chat_pb2.ChatResponse(message="⚠️ Message contains invalid characters", agent="error")

            logger.info(f"Received message: {sanitized_message[:50]}...")

            # Process through multi-agent system
            result = self.agent_system.process(
                message=sanitized_message,
                context=list(request.context) if request.context else None
            )

            response_text = result["response"]
            agent_used = result["agent"]

            logger.info(f"Agent used: {agent_used}")
            logger.info(f"Sending response: {response_text[:50]}...")

            return chat_pb2.ChatResponse(
                message=response_text,
                commands=[],  # Command extraction feature removed
                agent=agent_used  # Which agent handled the request
            )

        except ValueError as e:
            logger.error(f"Validation error: {e}")
            context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
            context.set_details(str(e))
            return chat_pb2.ChatResponse(message=f"⚠️ Validation error: {e}", agent="error")
        except Exception as e:
            logger.error(f"Error processing message: {e}")
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
            return chat_pb2.ChatResponse(message=f"❌ Error: {e}", agent="error")


def serve(port: int = 50051):
    """
    Start the gRPC server

    Args:
        port: Port to listen on (default: 50051)
    """
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    chat_pb2_grpc.add_ChatServiceServicer_to_server(
        ChatServiceServicer(), server
    )

    server.add_insecure_port(f'[::]:{port}')
    server.start()

    logger.info(f"🚀 Petoncle Agent Service started on port {port}")
    logger.info("Ready to receive chat requests...")

    try:
        server.wait_for_termination()
    except KeyboardInterrupt:
        logger.info("Shutting down server...")
        server.stop(0)


if __name__ == '__main__':
    # Get port from env or use default
    port = int(os.getenv('GRPC_PORT', 50051))
    serve(port)
