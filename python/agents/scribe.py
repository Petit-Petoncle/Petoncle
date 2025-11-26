"""
Scribe Agent - Report generation specialist
"""
from mistralai import Mistral
import os


class Scribe:
    """
    Report writing agent that documents pentesting activities
    """

    SYSTEM_PROMPT = """Tu es Scribe, expert en documentation et rédaction de rapports de sécurité.

📝 Ton rôle:
- Documenter les activités de pentesting
- Générer des rapports structurés
- Résumer les findings
- Créer de la documentation claire

📋 Types de rapports:
- Résumé d'activités (timeline)
- Findings de sécurité (vulnérabilités trouvées)
- Documentation technique (procédures)
- Executive summary (pour management)

🎯 Format de rapport:
1. **Contexte** - Objectif et scope
2. **Méthodologie** - Approche utilisée
3. **Findings** - Découvertes (avec sévérité)
4. **Preuves** - Commandes et outputs
5. **Recommandations** - Actions à prendre

💡 Style:
- Structuré et professionnel
- Markdown formaté
- Clair et concis
- Actionnable
"""

    def __init__(self, api_key: str = None):
        """Initialize Scribe with Mistral client"""
        self.api_key = api_key or os.getenv("MISTRAL_API_KEY")
        if not self.api_key:
            raise ValueError("MISTRAL_API_KEY not provided")

        self.client = Mistral(api_key=self.api_key, timeout_ms=30000)

    def process(self, message: str, context: list[str] = None) -> str:
        """
        Process a report generation request

        Args:
            message: User query about report generation
            context: Optional command history for context

        Returns:
            Generated report or documentation
        """
        # Add command history context if available
        system_prompt = self.SYSTEM_PROMPT
        if context:
            history_str = "\n".join(f"- {cmd}" for cmd in context[-10:])  # Last 10 commands
            system_prompt += f"\n\n**Historique des commandes récentes:**\n{history_str}"

        messages = [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": message}
        ]

        response = self.client.chat.complete(
            model="mistral-small-latest",
            messages=messages,
            max_tokens=2048  # More tokens for reports
        )

        return response.choices[0].message.content
