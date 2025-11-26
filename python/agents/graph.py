"""
LangGraph workflow for multi-agent system
"""
from typing import TypedDict, Literal, Annotated
from langgraph.graph import StateGraph, END
import operator

from .orchestrator import Orchestrator
from .toolsmith import Toolsmith
from .researcher import Researcher
from .scribe import Scribe
from .chat_agent import ChatAgent


class AgentState(TypedDict):
    """
    Shared state between all agents

    This state is passed through the entire workflow and each agent
    can read from it and write to it.
    """
    # Input
    message: str  # User's original message
    context: list[str]  # Command history context

    # Routing
    intent: str  # Detected intent (command_help, research, report, general)
    agent: str  # Which agent should handle this (toolsmith, researcher, scribe, general)

    # Output
    response: str  # Final response to user
    agent_used: str  # Which agent actually processed the request

    # Metadata
    steps: Annotated[list[str], operator.add]  # Track workflow steps (for debugging)


class MultiAgentSystem:
    """
    Multi-agent system using LangGraph

    Workflow:
    1. User input → Orchestrator (routing)
    2. Orchestrator → Specialist agent (processing)
    3. Specialist → Response
    """

    def __init__(self, api_key: str = None):
        """Initialize all agents"""
        self.orchestrator = Orchestrator()
        self.toolsmith = Toolsmith(api_key)
        self.researcher = Researcher(api_key)
        self.scribe = Scribe(api_key)
        self.general_agent = ChatAgent(api_key)

        # Build the graph
        self.graph = self._build_graph()

    def _build_graph(self) -> StateGraph:
        """
        Build the LangGraph workflow

        Flow:
        START → orchestrate → route_to_agent → specialist_agent → END
        """
        workflow = StateGraph(AgentState)

        # Add nodes
        workflow.add_node("orchestrate", self._orchestrate_node)
        workflow.add_node("toolsmith", self._toolsmith_node)
        workflow.add_node("researcher", self._researcher_node)
        workflow.add_node("scribe", self._scribe_node)
        workflow.add_node("general", self._general_node)

        # Set entry point
        workflow.set_entry_point("orchestrate")

        # Add conditional routing after orchestrator
        workflow.add_conditional_edges(
            "orchestrate",
            self._route_to_agent,
            {
                "toolsmith": "toolsmith",
                "researcher": "researcher",
                "scribe": "scribe",
                "general": "general",
            }
        )

        # All specialist agents end the workflow
        workflow.add_edge("toolsmith", END)
        workflow.add_edge("researcher", END)
        workflow.add_edge("scribe", END)
        workflow.add_edge("general", END)

        return workflow.compile()

    def _orchestrate_node(self, state: AgentState) -> AgentState:
        """
        Orchestrator node: Detect intent and route
        """
        routing = self.orchestrator.route(state["message"])

        return {
            **state,
            "intent": routing["intent"],
            "agent": routing["agent"],
            "steps": ["orchestrate"]
        }

    def _route_to_agent(self, state: AgentState) -> Literal["toolsmith", "researcher", "scribe", "general"]:
        """
        Conditional edge: Route to the appropriate agent
        """
        return state["agent"]

    def _toolsmith_node(self, state: AgentState) -> AgentState:
        """Toolsmith node: Handle command syntax queries"""
        response = self.toolsmith.process(state["message"], state.get("context"))

        return {
            **state,
            "response": response,
            "agent_used": "toolsmith",
            "steps": ["toolsmith"]
        }

    def _researcher_node(self, state: AgentState) -> AgentState:
        """Researcher node: Handle OSINT and vulnerability research"""
        response = self.researcher.process(state["message"], state.get("context"))

        return {
            **state,
            "response": response,
            "agent_used": "researcher",
            "steps": ["researcher"]
        }

    def _scribe_node(self, state: AgentState) -> AgentState:
        """Scribe node: Handle report generation"""
        response = self.scribe.process(state["message"], state.get("context"))

        return {
            **state,
            "response": response,
            "agent_used": "scribe",
            "steps": ["scribe"]
        }

    def _general_node(self, state: AgentState) -> AgentState:
        """General node: Handle generic queries (fallback)"""
        response = self.general_agent.chat(state["message"], state.get("context"))

        return {
            **state,
            "response": response,
            "agent_used": "general",
            "steps": ["general"]
        }

    def process(self, message: str, context: list[str] = None) -> dict:
        """
        Process a message through the multi-agent system

        Args:
            message: User message
            context: Optional command history

        Returns:
            dict with 'response' and 'agent_used'
        """
        initial_state: AgentState = {
            "message": message,
            "context": context or [],
            "intent": "",
            "agent": "",
            "response": "",
            "agent_used": "",
            "steps": []
        }

        # Run the graph
        final_state = self.graph.invoke(initial_state)

        return {
            "response": final_state["response"],
            "agent": final_state["agent_used"],
            "intent": final_state["intent"],
            "steps": final_state["steps"]  # For debugging
        }
