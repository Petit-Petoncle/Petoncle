"""
Researcher Agent - OSINT and web search specialist
"""
from mistralai import Mistral
import os


class Researcher:
    """
    OSINT agent for vulnerability research and web search
    """

    SYSTEM_PROMPT = """Tu es Researcher, expert en OSINT et recherche de vulnérabilités.

🔍 Ton rôle:
- Rechercher des CVEs et vulnérabilités
- Trouver des exploits et POCs
- Analyser les advisories de sécurité
- Résumer les informations trouvées

🎯 Sources que tu consultes:
- CVE databases (NVD, MITRE)
- Exploit-DB, Packet Storm
- GitHub (POCs, exploits)
- Security advisories (vendors)
- CTF writeups

📋 Format de réponse:
1. Résumé de la vulnérabilité
2. Sévérité (CVSS score si disponible)
3. Systèmes affectés
4. Exploits/POCs disponibles (avec liens si possible)
5. Recommandations de mitigation

💡 Style:
- Factuel et précis
- Citer les sources (CVE-XXXX-XXXX)
- Indiquer la criticité
- Pratique et actionnable
"""

    def __init__(self, api_key: str = None):
        """Initialize Researcher with Mistral client"""
        self.api_key = api_key or os.getenv("MISTRAL_API_KEY")
        if not self.api_key:
            raise ValueError("MISTRAL_API_KEY not provided")

        self.client = Mistral(api_key=self.api_key, timeout_ms=30000)

        # TODO: Add Tavily integration for web search
        # self.tavily_key = os.getenv("TAVILY_API_KEY")

    def process(self, message: str, context: list[str] = None) -> str:
        """
        Process a research request

        Args:
            message: User query about vulnerabilities/research
            context: Optional conversation context

        Returns:
            Research results and analysis
        """
        messages = [
            {"role": "system", "content": self.SYSTEM_PROMPT},
            {"role": "user", "content": message}
        ]

        response = self.client.chat.complete(
            model="mistral-small-latest",
            messages=messages,
            max_tokens=1024
        )

        return response.choices[0].message.content

    # TODO: Implement web search with Tavily
    # def search_web(self, query: str) -> list[dict]:
    #     """Search web for security information"""
    #     pass
