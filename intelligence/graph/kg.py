"""
Knowledge Graph — Multi-symbol relationship graph for correlation and sector analysis.

Builds and queries a graph of symbol relationships including:
- Sector membership
- Correlation edges
- Lead-lag relationships
- Market state transitions
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional
import json


class EdgeType(str, Enum):
    SECTOR_MEMBER = "sector_member"
    CORRELATED = "correlated"
    LEAD_LAG = "lead_lag"
    CAUSAL = "causal"
    DIVERGENT = "divergent"


class NodeType(str, Enum):
    SYMBOL = "symbol"
    SECTOR = "sector"
    EVENT = "event"
    REGIME = "regime"


@dataclass
class GraphNode:
    id: str
    node_type: NodeType
    properties: dict = field(default_factory=dict)

    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "type": self.node_type.value,
            "properties": self.properties,
        }


@dataclass
class GraphEdge:
    source: str
    target: str
    edge_type: EdgeType
    weight: float = 1.0
    properties: dict = field(default_factory=dict)

    def to_dict(self) -> dict:
        return {
            "source": self.source,
            "target": self.target,
            "type": self.edge_type.value,
            "weight": self.weight,
            "properties": self.properties,
        }


class KnowledgeGraph:
    def __init__(self):
        self.nodes: dict[str, GraphNode] = {}
        self.edges: list[GraphEdge] = []

    def add_node(self, node: GraphNode) -> None:
        self.nodes[node.id] = node

    def add_edge(self, edge: GraphEdge) -> None:
        self.edges.append(edge)

    def get_node(self, node_id: str) -> Optional[GraphNode]:
        return self.nodes.get(node_id)

    def get_neighbors(self, node_id: str, edge_type: Optional[EdgeType] = None) -> list[GraphNode]:
        neighbors = []
        for edge in self.edges:
            if edge.source == node_id:
                neighbor_id = edge.target
            elif edge.target == node_id:
                neighbor_id = edge.source
            else:
                continue

            if edge_type and edge.edge_type != edge_type:
                continue

            neighbor = self.nodes.get(neighbor_id)
            if neighbor:
                neighbors.append(neighbor)
        return neighbors

    def get_edges_for(self, node_id: str, edge_type: Optional[EdgeType] = None) -> list[GraphEdge]:
        result = []
        for edge in self.edges:
            if edge.source != node_id and edge.target != node_id:
                continue
            if edge_type and edge.edge_type != edge_type:
                continue
            result.append(edge)
        return result

    def sector_symbols(self, sector: str) -> list[str]:
        sector_node = f"sector:{sector}"
        return [
            edge.target.replace("symbol:", "")
            for edge in self.edges
            if edge.source == sector_node and edge.edge_type == EdgeType.SECTOR_MEMBER
        ]

    def strongly_correlated(self, symbol: str, threshold: float = 0.7) -> list[tuple[str, float]]:
        """Return symbols strongly correlated with the given symbol."""
        results = []
        for edge in self.edges:
            if edge.edge_type != EdgeType.CORRELATED:
                continue
            if edge.source == f"symbol:{symbol}":
                partner = edge.target.replace("symbol:", "")
            elif edge.target == f"symbol:{symbol}":
                partner = edge.source.replace("symbol:", "")
            else:
                continue
            if abs(edge.weight) >= threshold:
                results.append((partner, edge.weight))
        results.sort(key=lambda x: abs(x[1]), reverse=True)
        return results

    def export_json(self) -> str:
        return json.dumps({
            "nodes": [n.to_dict() for n in self.nodes.values()],
            "edges": [e.to_dict() for e in self.edges],
        }, indent=2, ensure_ascii=False)

    def stats(self) -> dict:
        return {
            "nodes": len(self.nodes),
            "edges": len(self.edges),
            "node_types": {t.value: sum(1 for n in self.nodes.values() if n.node_type == t) for t in NodeType},
            "edge_types": {t.value: sum(1 for e in self.edges if e.edge_type == t) for t in EdgeType},
        }


def build_graph_from_correlations(
    sectors: dict[str, list[str]],
    correlation_pairs: list[dict],
) -> KnowledgeGraph:
    """Build a knowledge graph from sector definitions and correlation data."""
    kg = KnowledgeGraph()

    for sector, symbols in sectors.items():
        kg.add_node(GraphNode(id=f"sector:{sector}", node_type=NodeType.SECTOR, properties={"name": sector}))
        for symbol in symbols:
            node_id = f"symbol:{symbol}"
            if node_id not in kg.nodes:
                kg.add_node(GraphNode(id=node_id, node_type=NodeType.SYMBOL, properties={"symbol": symbol}))
            kg.add_edge(GraphEdge(
                source=f"sector:{sector}", target=node_id,
                edge_type=EdgeType.SECTOR_MEMBER, weight=1.0,
            ))

    for pair in correlation_pairs:
        source_id = f"symbol:{pair['symbol_a']}"
        target_id = f"symbol:{pair['symbol_b']}"
        corr = pair.get("correlation", 0)
        edge_type = EdgeType.LEAD_LAG if pair.get("window", 0) != 0 else EdgeType.CORRELATED
        kg.add_edge(GraphEdge(
            source=source_id, target=target_id,
            edge_type=edge_type, weight=corr,
            properties={"window": pair.get("window", 0)},
        ))

    return kg
