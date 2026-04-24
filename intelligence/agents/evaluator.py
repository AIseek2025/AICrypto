"""
Strategy Auto-Evaluator — Automated strategy lifecycle management.

Handles:
- Performance scoring
- Auto promotion/demotion across validation stages
- Cool-down and disable rules
- Historical tracking
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional
import time


class ValidationStage(str, Enum):
    DRAFT = "draft"
    BACKTEST_PASSED = "backtest_passed"
    PAPER_APPROVED = "paper_approved"
    LIVE = "live"
    DISABLED = "disabled"


@dataclass
class StrategyPerformance:
    strategy_id: str
    stage: ValidationStage = ValidationStage.DRAFT
    backtest_trades: int = 0
    backtest_win_rate: float = 0.0
    backtest_sharpe: float = 0.0
    backtest_max_dd: float = 0.0
    paper_trades: int = 0
    paper_win_rate: float = 0.0
    paper_sharpe: float = 0.0
    live_trades: int = 0
    live_win_rate: float = 0.0
    live_sharpe: float = 0.0
    consecutive_losses: int = 0
    total_pnl: float = 0.0
    last_updated: float = field(default_factory=time.time)


PROMOTION_CRITERIA = {
    ValidationStage.DRAFT: {
        "min_trades": 20,
        "min_win_rate": 0.45,
        "min_sharpe": 0.5,
        "max_drawdown": -0.15,
    },
    ValidationStage.BACKTEST_PASSED: {
        "min_trades": 15,
        "min_win_rate": 0.40,
        "min_sharpe": 0.3,
        "max_drawdown": -0.20,
    },
    ValidationStage.PAPER_APPROVED: {
        "min_trades": 10,
        "min_win_rate": 0.35,
        "min_sharpe": 0.0,
        "max_drawdown": -0.25,
    },
}

DEMOTION_TRIGGERS = {
    "consecutive_losses": 5,
    "daily_loss_pct": -0.05,
    "sharpe_drop_below": -0.5,
}


class StrategyEvaluator:
    def __init__(self):
        self.strategies: dict[str, StrategyPerformance] = {}

    def register(self, strategy_id: str) -> StrategyPerformance:
        perf = StrategyPerformance(strategy_id=strategy_id)
        self.strategies[strategy_id] = perf
        return perf

    def evaluate(self, strategy_id: str) -> dict:
        """Evaluate a strategy and return promotion/demotion recommendation."""
        perf = self.strategies.get(strategy_id)
        if not perf:
            return {"strategy_id": strategy_id, "action": "unknown", "reason": "not registered"}

        if perf.stage == ValidationStage.DISABLED:
            return {"strategy_id": strategy_id, "action": "none", "reason": "disabled"}

        if perf.consecutive_losses >= DEMOTION_TRIGGERS["consecutive_losses"]:
            return {
                "strategy_id": strategy_id,
                "action": "demote",
                "from_stage": perf.stage.value,
                "to_stage": _prev_stage(perf.stage).value,
                "reason": f"consecutive_losses={perf.consecutive_losses}",
            }

        if perf.stage == ValidationStage.LIVE and perf.live_sharpe < DEMOTION_TRIGGERS["sharpe_drop_below"]:
            return {
                "strategy_id": strategy_id,
                "action": "demote",
                "from_stage": perf.stage.value,
                "to_stage": ValidationStage.PAPER_APPROVED.value,
                "reason": f"live_sharpe={perf.live_sharpe:.2f} below threshold",
            }

        criteria = PROMOTION_CRITERIA.get(perf.stage)
        if not criteria:
            return {"strategy_id": strategy_id, "action": "maintain", "reason": "no promotion path"}

        stage_data = _get_stage_metrics(perf, perf.stage)
        if _meets_criteria(stage_data, criteria):
            next_stage = _next_stage(perf.stage)
            return {
                "strategy_id": strategy_id,
                "action": "promote",
                "from_stage": perf.stage.value,
                "to_stage": next_stage.value,
                "reason": "meets all criteria",
                "metrics": stage_data,
            }

        return {"strategy_id": strategy_id, "action": "maintain", "reason": "does not meet criteria"}

    def evaluate_all(self) -> list[dict]:
        return [self.evaluate(sid) for sid in self.strategies]

    def apply_action(self, strategy_id: str, action: dict) -> None:
        """Apply a promotion/demotion action."""
        perf = self.strategies.get(strategy_id)
        if not perf:
            return
        if action["action"] == "promote":
            perf.stage = ValidationStage(action["to_stage"])
        elif action["action"] == "demote":
            perf.stage = ValidationStage(action["to_stage"])
            perf.consecutive_losses = 0
        elif action["action"] == "disable":
            perf.stage = ValidationStage.DISABLED
        perf.last_updated = time.time()


def _next_stage(stage: ValidationStage) -> ValidationStage:
    order = list(ValidationStage)
    idx = order.index(stage)
    return order[idx + 1] if idx < len(order) - 2 else stage


def _prev_stage(stage: ValidationStage) -> ValidationStage:
    order = list(ValidationStage)
    idx = order.index(stage)
    return order[idx - 1] if idx > 0 else stage


def _get_stage_metrics(perf: StrategyPerformance, stage: ValidationStage) -> dict:
    prefix = {
        ValidationStage.DRAFT: "backtest",
        ValidationStage.BACKTEST_PASSED: "paper",
        ValidationStage.PAPER_APPROVED: "live",
    }.get(stage, "backtest")
    return {
        "trades": getattr(perf, f"{prefix}_trades", 0),
        "win_rate": getattr(perf, f"{prefix}_win_rate", 0),
        "sharpe": getattr(perf, f"{prefix}_sharpe", 0),
        "max_drawdown": getattr(perf, "backtest_max_dd", 0),
    }


def _meets_criteria(metrics: dict, criteria: dict) -> bool:
    return (
        metrics["trades"] >= criteria["min_trades"]
        and metrics["win_rate"] >= criteria["min_win_rate"]
        and metrics["sharpe"] >= criteria["min_sharpe"]
        and metrics["max_drawdown"] >= criteria["max_drawdown"]
    )
