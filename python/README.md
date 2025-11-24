# Petoncle Python Agent Service

Backend Python pour Petoncle utilisant gRPC et Mistral API.

## Setup

1. **Installer les dépendances:**
```bash
cd python
pip install -r requirements.txt
```

2. **Configurer la clé API:**
```bash
cp .env.example .env
# Éditer .env et ajouter ta clé MISTRAL_API_KEY
```

3. **Générer les stubs gRPC:**
```bash
chmod +x generate_grpc.sh
./generate_grpc.sh
```

## Lancer le service

```bash
python agent_service.py
```

Le serveur gRPC démarre sur le port `50051` (configurable via `GRPC_PORT` dans `.env`).

## Architecture

```
agent_service.py          # Serveur gRPC
├── agents/
│   └── chat_agent.py     # Agent Mistral pour le chat
└── proto/
    ├── chat.proto        # Définition du service gRPC
    ├── chat_pb2.py       # Généré par protoc
    └── chat_pb2_grpc.py  # Généré par protoc
```

## Test

Tu peux tester le service avec un client gRPC (BloomRPC, grpcurl, ou le client Rust de Petoncle).
