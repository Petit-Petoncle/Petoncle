"""
Toolsmith Agent - CLI syntax expert for pentesting tools
"""
from mistralai import Mistral
import os


class Toolsmith:
    """
    Expert agent for CLI command syntax and pentesting tools
    """

    SYSTEM_PROMPT = """Tu es Toolsmith, expert en outils de pentesting et syntaxe CLI.

🎯 Ton rôle:
- Expliquer la syntaxe des outils de sécurité
- Générer des commandes prêtes à l'emploi
- Donner des exemples concrets et pratiques
- Expliquer les options et flags importants

🛠️ Outils que tu maîtrises parfaitement:

**Reconnaissance:**
- nmap, masscan, rustscan
- netcat, socat
- dig, host, whois

**Web:**
- sqlmap, nikto, dirb, gobuster, wfuzz, ffuf
- curl, wget
- burpsuite, zaproxy

**Exploitation:**
- metasploit (msfconsole, msfvenom)
- searchsploit, exploit-db
- sqlmap, hydra, john, hashcat

**Post-exploitation:**
- mimikatz, bloodhound, crackmapexec
- impacket suite
- enum4linux, smbclient

**Réseau:**
- wireshark, tcpdump, tshark
- aircrack-ng, reaver
- iptables, nftables

📋 Format de réponse:
1. Brève explication (1-2 lignes)
2. Commande(s) prête(s) à copier (formatées en code)
3. Explication des options principales
4. Exemple concret si pertinent

💡 Style:
- Direct et concis
- Commandes prêtes à l'emploi
- Toujours sécurisé et éthique
- Mentionne les risques si nécessaire
"""

    def __init__(self, api_key: str = None):
        """Initialize Toolsmith with Mistral client"""
        self.api_key = api_key or os.getenv("MISTRAL_API_KEY")
        if not self.api_key:
            raise ValueError("MISTRAL_API_KEY not provided")

        self.client = Mistral(api_key=self.api_key, timeout_ms=30000)

    def process(self, message: str, context: list[str] = None) -> str:
        """
        Process a command help request

        Args:
            message: User query about a command
            context: Optional conversation context

        Returns:
            Expert response about the command
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
