"""
Chat agent using Mistral API for terminal assistance
"""
import os
from mistralai import Mistral


class ChatAgent:
    """Agent that handles chat interactions using Mistral API"""

    def __init__(self, api_key: str = None):
        """
        Initialize the chat agent

        Args:
            api_key: Mistral API key (defaults to MISTRAL_API_KEY env var)
        """
        self.api_key = api_key or os.getenv("MISTRAL_API_KEY")
        if not self.api_key:
            raise ValueError("MISTRAL_API_KEY not provided")

        self.client = Mistral(api_key=self.api_key)
        self.conversation_history = []

    def chat(self, message: str, context: list[str] = None) -> str:
        """
        Send a message and get AI response

        Args:
            message: User message
            context: Optional command history for context

        Returns:
            AI response message
        """
        # Build system prompt with terminal context
        system_prompt = """You are Petoncle, an AI assistant for pentesters and security researchers.
You help users with terminal commands, security tools, and cybersecurity tasks.

Key capabilities:
- Suggest shell commands for security testing
- Explain security tools (nmap, sqlmap, metasploit, etc.)
- Help with penetration testing workflows
- Provide command examples

Always be helpful, concise, and security-focused.
"""

        # Add command history context if provided
        if context:
            context_str = "\n".join(f"$ {cmd}" for cmd in context[-5:])  # Last 5 commands
            system_prompt += f"\n\nRecent terminal commands:\n{context_str}"

        # Add user message to history
        self.conversation_history.append({
            "role": "user",
            "content": message
        })

        # Prepare messages with system prompt
        messages = [
            {"role": "system", "content": system_prompt}
        ] + self.conversation_history

        # Call Mistral API
        response = self.client.chat.complete(
            model="mistral-small-latest",
            messages=messages,
            max_tokens=1024
        )

        # Extract response text
        assistant_message = response.choices[0].message.content

        # Add to history
        self.conversation_history.append({
            "role": "assistant",
            "content": assistant_message
        })

        return assistant_message

    def reset(self):
        """Clear conversation history"""
        self.conversation_history = []
