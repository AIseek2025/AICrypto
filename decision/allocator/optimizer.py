"""
Portfolio Optimizer — Multi-strategy allocation and risk-aware position sizing.

Implements:
- Kelly criterion position sizing
- Equal-weight and risk-parity allocation
- Drawdown-aware scaling
- Strategy auto-evaluation and ranking
"""

import math
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


class AllocationMethod(str, Enum):
    EQUAL_WEIGHT = "equal_weight"
    RISK_PARITY = "risk_parity"
    KELLY = "kelly"
    CUSTOM = "custom"


@dataclass
class StrategyMetrics:
    strategy_id: str
    total_trades: int
    win_rate: float
    avg_win: float
    avg_loss: float
    profit_factor: float
    sharpe_ratio: float
    max_drawdown: float
    status: str = "active"

    @property
    def expectancy(self) -> float:
        return self.win_rate * self.avg_win - (1 - self.win_rate) * abs(self.avg_loss)

    @property
    def kelly_fraction(self) -> float:
        if self.avg_loss == 0:
            return 0
        w = self.win_rate
        r = self.avg_win / abs(self.avg_loss)
        return (w * r - (1 - w)) / r if r > 0 else 0


@dataclass
class Allocation:
    strategy_id: str
    weight: float
    max_position_pct: float
    current_pnl_pct: float = 0.0


@dataclass
class PortfolioState:
    equity: float
    cash: float
    allocations: list[Allocation] = field(default_factory=list)
    total_exposure: float = 0.0
    daily_pnl_pct: float = 0.0
    max_drawdown_pct: float = 0.0


class PortfolioOptimizer:
    def __init__(self, equity: float, max_total_exposure: float = 0.8, max_single: float = 0.15):
        self.equity = equity
        self.max_total_exposure = max_total_exposure
        self.max_single = max_single
        self.strategy_metrics: dict[str, StrategyMetrics] = {}

    def update_metrics(self, metrics: StrategyMetrics) -> None:
        self.strategy_metrics[metrics.strategy_id] = metrics

    def auto_evaluate(self) -> dict[str, str]:
        """Evaluate all strategies and return status recommendations."""
        recommendations = {}
        for sid, m in self.strategy_metrics.items():
            if m.total_trades < 10:
                recommendations[sid] = "insufficient_data"
            elif m.expectancy <= 0:
                recommendations[sid] = "disable"
            elif m.max_drawdown < -0.15:
                recommendations[sid] = "reduce"
            elif m.sharpe_ratio > 1.5 and m.win_rate > 0.6:
                recommendations[sid] = "increase"
            elif m.sharpe_ratio < 0:
                recommendations[sid] = "review"
            else:
                recommendations[sid] = "maintain"
        return recommendations

    def allocate_equal_weight(self, active_strategies: list[str]) -> list[Allocation]:
        n = len(active_strategies)
        if n == 0:
            return []
        weight = min(1.0 / n, self.max_single)
        return [
            Allocation(strategy_id=sid, weight=weight, max_position_pct=weight * self.equity)
            for sid in active_strategies
        ]

    def allocate_risk_parity(self, active_strategies: list[str]) -> list[Allocation]:
        """Allocate inversely proportional to each strategy's risk (drawdown)."""
        risks = {}
        for sid in active_strategies:
            m = self.strategy_metrics.get(sid)
            risks[sid] = abs(m.max_drawdown) if m and m.max_drawdown != 0 else 0.1

        inv_risks = {sid: 1.0 / max(r, 0.01) for sid, r in risks.items()}
        total_inv = sum(inv_risks.values())

        allocations = []
        for sid in active_strategies:
            weight = min(inv_risks[sid] / total_inv, self.max_single)
            allocations.append(Allocation(
                strategy_id=sid, weight=weight,
                max_position_pct=weight * self.equity,
            ))
        return allocations

    def allocate_kelly(self, active_strategies: list[str]) -> list[Allocation]:
        """Allocate using fractional Kelly criterion."""
        allocations = []
        for sid in active_strategies:
            m = self.strategy_metrics.get(sid)
            if not m or m.kelly_fraction <= 0:
                allocations.append(Allocation(strategy_id=sid, weight=0, max_position_pct=0))
                continue

            half_kelly = m.kelly_fraction * 0.5
            weight = min(half_kelly, self.max_single)
            allocations.append(Allocation(
                strategy_id=sid, weight=weight,
                max_position_pct=weight * self.equity,
            ))

        total_weight = sum(a.weight for a in allocations)
        if total_weight > self.max_total_exposure:
            scale = self.max_total_exposure / total_weight
            for a in allocations:
                a.weight *= scale
                a.max_position_pct = a.weight * self.equity

        return allocations

    def allocate(self, method: AllocationMethod, active_strategies: list[str]) -> list[Allocation]:
        dispatchers = {
            AllocationMethod.EQUAL_WEIGHT: self.allocate_equal_weight,
            AllocationMethod.RISK_PARITY: self.allocate_risk_parity,
            AllocationMethod.KELLY: self.allocate_kelly,
        }
        dispatcher = dispatchers.get(method, self.allocate_equal_weight)
        return dispatcher(active_strategies)

    def portfolio_state(self, allocations: list[Allocation]) -> PortfolioState:
        total_exposure = sum(a.max_position_pct for a in allocations)
        return PortfolioState(
            equity=self.equity,
            cash=self.equity - total_exposure,
            allocations=allocations,
            total_exposure=total_exposure / self.equity,
        )

    def rank_strategies(self) -> list[tuple[str, float]]:
        """Rank strategies by composite score (expectancy + sharpe + low drawdown)."""
        scores = {}
        for sid, m in self.strategy_metrics.items():
            score = (
                m.expectancy * 0.4
                + m.sharpe_ratio * 0.1
                + (1 + m.max_drawdown) * 0.3
                + m.win_rate * 0.2
            )
            scores[sid] = round(score, 4)

        return sorted(scores.items(), key=lambda x: x[1], reverse=True)
