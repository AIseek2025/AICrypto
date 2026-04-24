"""
Agent Framework — Multi-agent orchestration for research, signal, execution, risk, and review.

Defines 5 agent types and a simple orchestration layer for coordinated workflows.
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Optional
import json
import uuid
import time


class AgentType(str, Enum):
    RESEARCH = "research"
    SIGNAL = "signal"
    EXECUTION = "execution"
    RISK = "risk"
    REVIEW = "review"


class AgentStatus(str, Enum):
    IDLE = "idle"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"


@dataclass
class AgentMessage:
    message_id: str = field(default_factory=lambda: str(uuid.uuid4()))
    from_agent: str = ""
    to_agent: str = ""
    message_type: str = ""
    payload: dict = field(default_factory=dict)
    timestamp: float = field(default_factory=time.time)


@dataclass
class AgentResult:
    agent_id: str
    agent_type: AgentType
    status: AgentStatus
    output: dict = field(default_factory=dict)
    messages: list[AgentMessage] = field(default_factory=list)
    error: Optional[str] = None


class BaseAgent(ABC):
    def __init__(self, agent_id: str, agent_type: AgentType):
        self.agent_id = agent_id
        self.agent_type = agent_type
        self.status = AgentStatus.IDLE
        self.inbox: list[AgentMessage] = []

    @abstractmethod
    def execute(self, context: dict) -> AgentResult:
        ...

    def receive(self, message: AgentMessage) -> None:
        self.inbox.append(message)

    def send(self, to_agent: str, message_type: str, payload: dict) -> AgentMessage:
        msg = AgentMessage(
            from_agent=self.agent_id,
            to_agent=to_agent,
            message_type=message_type,
            payload=payload,
        )
        return msg


class ResearchAgent(BaseAgent):
    """Investigates market conditions, evaluates data quality, produces research reports."""

    def __init__(self, agent_id: str = "research-001"):
        super().__init__(agent_id, AgentType.RESEARCH)

    def execute(self, context: dict) -> AgentResult:
        self.status = AgentStatus.RUNNING
        symbols = context.get("symbols", [])
        market_data = context.get("market_data", {})

        research_findings = []
        for symbol in symbols:
            data = market_data.get(symbol, {})
            price_change = data.get("change_24h", 0)
            volume = data.get("volume_24h", 0)

            condition = "neutral"
            if price_change > 5 and volume > 1e9:
                condition = "strong_bullish"
            elif price_change > 2:
                condition = "bullish"
            elif price_change < -5 and volume > 1e9:
                condition = "strong_bearish"
            elif price_change < -2:
                condition = "bearish"

            research_findings.append({
                "symbol": symbol,
                "condition": condition,
                "price_change": price_change,
                "volume": volume,
                "data_quality": "good" if volume > 0 else "no_data",
            })

        self.status = AgentStatus.COMPLETED
        return AgentResult(
            agent_id=self.agent_id,
            agent_type=self.agent_type,
            status=self.status,
            output={"findings": research_findings},
            messages=[
                self.send("signal-001", "research_complete", {"findings_count": len(research_findings)})
            ],
        )


class SignalAgent(BaseAgent):
    """Evaluates research findings and generates trading signals."""

    def __init__(self, agent_id: str = "signal-001"):
        super().__init__(agent_id, AgentType.SIGNAL)

    def execute(self, context: dict) -> AgentResult:
        self.status = AgentStatus.RUNNING
        findings = context.get("research_findings", [])
        skills = context.get("skills", [])

        signals = []
        for finding in findings:
            condition = finding.get("condition", "")
            symbol = finding.get("symbol", "")
            confidence = min(abs(finding.get("price_change", 0)) / 10, 1.0)

            if condition in ("strong_bullish", "bullish"):
                signals.append({
                    "symbol": symbol,
                    "direction": "LONG",
                    "signal_type": "entry",
                    "confidence": round(confidence, 3),
                    "reason_codes": [condition, "research_driven"],
                })
            elif condition in ("strong_bearish", "bearish"):
                signals.append({
                    "symbol": symbol,
                    "direction": "SHORT",
                    "signal_type": "entry",
                    "confidence": round(confidence, 3),
                    "reason_codes": [condition, "research_driven"],
                })

        self.status = AgentStatus.COMPLETED
        return AgentResult(
            agent_id=self.agent_id,
            agent_type=self.agent_type,
            status=self.status,
            output={"signals": signals},
            messages=[
                self.send("risk-001", "signals_generated", {"signal_count": len(signals)})
            ],
        )


class RiskAgent(BaseAgent):
    """Evaluates signals against risk rules and approves/denies/reduces."""

    def __init__(self, agent_id: str = "risk-001"):
        super().__init__(agent_id, AgentType.RISK)

    def execute(self, context: dict) -> AgentResult:
        self.status = AgentStatus.RUNNING
        signals = context.get("signals", [])
        portfolio_state = context.get("portfolio", {})
        risk_rules = context.get("risk_rules", [])

        decisions = []
        for signal in signals:
            confidence = signal.get("confidence", 0)
            decision = "allow"
            reason = []

            if confidence < 0.3:
                decision = "deny"
                reason.append("low_confidence")
            if portfolio_state.get("open_positions", 0) >= 5:
                decision = "deny"
                reason.append("max_positions")
            if portfolio_state.get("daily_loss_pct", 0) < -3:
                decision = "deny"
                reason.append("daily_loss_limit")

            decisions.append({
                "signal": signal,
                "decision": decision,
                "reason": reason,
            })

        self.status = AgentStatus.COMPLETED
        return AgentResult(
            agent_id=self.agent_id,
            agent_type=self.agent_type,
            status=self.status,
            output={"risk_decisions": decisions},
            messages=[
                self.send("execution-001", "risk_decisions", {"approved": sum(1 for d in decisions if d["decision"] == "allow")})
            ],
        )


class ExecutionAgent(BaseAgent):
    """Executes approved trades."""

    def __init__(self, agent_id: str = "execution-001"):
        super().__init__(agent_id, AgentType.EXECUTION)

    def execute(self, context: dict) -> AgentResult:
        self.status = AgentStatus.RUNNING
        decisions = context.get("risk_decisions", [])
        dry_run = context.get("dry_run", True)

        executions = []
        for decision in decisions:
            if decision["decision"] != "allow":
                continue
            signal = decision["signal"]
            executions.append({
                "symbol": signal["symbol"],
                "direction": signal["direction"],
                "status": "simulated" if dry_run else "submitted",
                "confidence": signal["confidence"],
            })

        self.status = AgentStatus.COMPLETED
        return AgentResult(
            agent_id=self.agent_id,
            agent_type=self.agent_type,
            status=self.status,
            output={"executions": executions, "dry_run": dry_run},
            messages=[
                self.send("review-001", "execution_report", {"executed": len(executions)})
            ],
        )


class ReviewAgent(BaseAgent):
    """Reviews execution results and archives outcomes."""

    def __init__(self, agent_id: str = "review-001"):
        super().__init__(agent_id, AgentType.REVIEW)

    def execute(self, context: dict) -> AgentResult:
        self.status = AgentStatus.RUNNING
        executions = context.get("executions", [])

        review = {
            "total_executed": len(executions),
            "symbols_traded": list(set(e["symbol"] for e in executions)),
            "long_count": sum(1 for e in executions if e["direction"] == "LONG"),
            "short_count": sum(1 for e in executions if e["direction"] == "SHORT"),
            "avg_confidence": sum(e["confidence"] for e in executions) / max(len(executions), 1),
            "recommendation": "proceed" if len(executions) > 0 else "no_action",
        }

        self.status = AgentStatus.COMPLETED
        return AgentResult(
            agent_id=self.agent_id,
            agent_type=self.agent_type,
            status=self.status,
            output={"review": review},
        )


class AgentOrchestrator:
    """Coordinates multi-agent workflow execution."""

    def __init__(self):
        self.research = ResearchAgent()
        self.signal = SignalAgent()
        self.risk = RiskAgent()
        self.execution = ExecutionAgent()
        self.review = ReviewAgent()
        self.results: list[AgentResult] = []

    def run_pipeline(self, context: dict) -> dict:
        ctx = dict(context)

        research_result = self.research.execute(ctx)
        self.results.append(research_result)
        ctx["research_findings"] = research_result.output.get("findings", [])

        signal_result = self.signal.execute(ctx)
        self.results.append(signal_result)
        ctx["signals"] = signal_result.output.get("signals", [])

        risk_result = self.risk.execute(ctx)
        self.results.append(risk_result)
        ctx["risk_decisions"] = risk_result.output.get("risk_decisions", [])

        execution_result = self.execution.execute(ctx)
        self.results.append(execution_result)
        ctx["executions"] = execution_result.output.get("executions", [])

        review_result = self.review.execute(ctx)
        self.results.append(review_result)

        return {
            "pipeline_status": "completed",
            "agents_run": len(self.results),
            "research_findings": len(research_result.output.get("findings", [])),
            "signals_generated": len(signal_result.output.get("signals", [])),
            "risk_approved": sum(1 for d in risk_result.output.get("risk_decisions", []) if d["decision"] == "allow"),
            "executions": len(execution_result.output.get("executions", [])),
            "review": review_result.output.get("review", {}),
        }
