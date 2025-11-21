## 🏗 Architecture Technique

Ce projet repose sur une architecture hybride **Rust + Python** conçue pour concilier performance extrême (latence < 50ms pour la frappe) et puissance d'analyse (écosystème IA riche).

L'architecture suit le pattern **"Two Brains"** (Deux Cerveaux) :
1.  **Fast Brain (Rust/ONNX) :** Gestion temps réel, UI et autocomplétion.
2.  **Slow Brain (Python/Agents) :** Raisonnement complexe, RAG et outils externes.

### 📊 Vue d'Ensemble

```mermaid
graph TD
    subgraph Terminal Host [Poste du Pentester]
        User((Utilisateur)) -->|Frappe| Core[Rust Core Wrapper]
        
        subgraph "⚡ Fast Brain (Latence < 50ms)"
            Core -->|Keystroke| AutoComp[Moteur Autocomplétion]
            AutoComp -->|Ghost Text| Core
        end
        
        subgraph "🧠 Slow Brain (Python Service)"
            Core -->|gRPC / UDS| PyServer[Python Agent Service]
            
            PyServer --> Orch{Orchestrator Agent}
            
            Orch -->|Syntaxe| Toolsmith[Agent: Toolsmith]
            Orch -->|Web Search| Researcher[Agent: Researcher]
            Orch -->|Reporting| Scribe[Agent: Scribe]
            
            subgraph "📚 Long Term Memory (RAG)"
                CmdLog[Command Log] -->|Ingestion| VectorDB[(ChromaDB)]
                VectorDB -->|Retrieval| Orch
                Scribe -->|Read Logs| VectorDB
            end
        end
    end
    
    Researcher <-->|API| Internet((Web / CVEs))
