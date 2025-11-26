"""
Orchestrator Agent - Routes requests to specialized agents
"""
from typing import Literal


class Orchestrator:
    """
    Orchestrator agent that analyzes user intent and routes to the appropriate specialist
    """

    # Intent keywords mapping
    INTENT_KEYWORDS = {
        "command_help": [
            "comment", "syntaxe", "utiliser", "commande", "command",
            "option", "flag", "paramètre", "exemple"
        ],
        "research": [
            "cve", "vulnérabilité", "vulnerability", "exploit", "recherche",
            "chercher", "trouver", "poc", "proof of concept", "security advisory"
        ],
        "report": [
            "rapport", "report", "résumé", "summary", "historique",
            "history", "logs", "documentation", "documenter"
        ],
    }

    def detect_intent(self, message: str) -> Literal["command_help", "research", "report", "general"]:
        """
        Detect user intent from message

        Args:
            message: User message to analyze

        Returns:
            Intent type: command_help, research, report, or general
        """
        message_lower = message.lower()

        # Count keyword matches for each intent
        scores = {}
        for intent, keywords in self.INTENT_KEYWORDS.items():
            score = sum(1 for keyword in keywords if keyword in message_lower)
            scores[intent] = score

        # Return intent with highest score (if > 0)
        if max(scores.values()) > 0:
            return max(scores, key=scores.get)

        # Default to general if no strong intent detected
        return "general"

    def route(self, message: str) -> dict:
        """
        Route message to appropriate agent

        Args:
            message: User message

        Returns:
            Routing decision with agent name and confidence
        """
        intent = self.detect_intent(message)

        routing = {
            "command_help": "toolsmith",
            "research": "researcher",
            "report": "scribe",
            "general": "general"
        }

        return {
            "agent": routing[intent],
            "intent": intent,
            "message": message
        }
