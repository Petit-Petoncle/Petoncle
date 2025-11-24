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
        """Initialize the service with a chat agent"""
        load_dotenv()
        self.agent = ChatAgent()
        logger.info("Chat agent initialized")

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
            logger.info(f"Received message: {request.message[:50]}...")

            # Get AI response
            response_text = self.agent.chat(
                message=request.message,
                context=list(request.context) if request.context else None
            )

            logger.info(f"Sending response: {response_text[:50]}...")

            # Extract commands from response (simple heuristic for now)
            commands = self._extract_commands(response_text)

            return chat_pb2.ChatResponse(
                message=response_text,
                commands=commands
            )

        except Exception as e:
            logger.error(f"Error processing message: {e}")
            context.set_code(grpc.StatusCode.INTERNAL)
            context.set_details(str(e))
            return chat_pb2.ChatResponse(message=f"Error: {e}")

    def _extract_commands(self, text: str) -> list[str]:
        """
        Extract shell commands from AI response

        Simple heuristic: lines that start with common command names
        """
        commands = []
        command_prefixes = ['nmap', 'sqlmap', 'nc', 'netcat', 'curl', 'wget',
                          'grep', 'find', 'awk', 'sed', 'python', 'ruby']

        for line in text.split('\n'):
            trimmed = line.strip()
            for prefix in command_prefixes:
                if trimmed.startswith(prefix + ' '):
                    commands.append(trimmed)
                    break

        return commands[:9]  # Limit to 9 for UI (1-9 keys)


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
