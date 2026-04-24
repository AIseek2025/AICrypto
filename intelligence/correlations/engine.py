"""
Correlation Engine — Cross-symbol and cross-sector correlation analysis.

Computes rolling correlations, sector rotation signals, and
lead-lag relationships between trading pairs.
"""

import numpy as np
import pandas as pd
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


class CorrelationType(str, Enum):
    POSITIVE = "positive"
    NEGATIVE = "negative"
    NONE = "none"


@dataclass
class CorrelationPair:
    symbol_a: str
    symbol_b: str
    correlation: float
    correlation_type: CorrelationType
    window: int
    sector: Optional[str] = None


@dataclass
class SectorSnapshot:
    sector: str
    symbols: list[str]
    avg_change: float
    lead_symbol: Optional[str]
    rotation_signal: Optional[str]


SECTOR_MAP: dict[str, list[str]] = {
    "layer1": ["ETHUSDT", "SOLUSDT", "ADAUSDT", "AVAXUSDT", "DOTUSDT"],
    "defi": ["UNIUSDT", "AAVEUSDT", "MKRUSDT", "COMPUSDT", "SNXUSDT"],
    "meme": ["DOGEUSDT", "SHIBUSDT", "PEPEUSDT", "FLOKIUSDT", "BONKUSDT"],
    "exchange": ["BNBUSDT", "OKBUSDT", "CROUSDT"],
    "btc_eco": ["BTCUSDT", "BCHUSDT", "BSVUSDT"],
    "ai": ["FETUSDT", "AGIXUSDT", "OCEANUSDT", "RNDRUSDT"],
}


def compute_rolling_correlation(
    series_a: pd.Series,
    series_b: pd.Series,
    window: int = 20,
) -> pd.Series:
    """Compute rolling Pearson correlation between two price series."""
    returns_a = series_a.pct_change()
    returns_b = series_b.pct_change()
    return returns_a.rolling(window).corr(returns_b)


def classify_correlation(value: float, threshold: float = 0.5) -> CorrelationType:
    if value >= threshold:
        return CorrelationType.POSITIVE
    if value <= -threshold:
        return CorrelationType.NEGATIVE
    return CorrelationType.NONE


def compute_correlation_matrix(
    price_df: pd.DataFrame,
    window: int = 20,
) -> pd.DataFrame:
    """Compute rolling correlation matrix for all symbol pairs."""
    returns = price_df.pct_change()
    corr = returns.rolling(window).corr()
    return corr


def detect_sector_rotation(
    price_df: pd.DataFrame,
    lookback: int = 24,
) -> list[SectorSnapshot]:
    """Detect sector rotation by comparing average sector performance."""
    returns = price_df.pct_change(lookback).iloc[-1]
    snapshots: list[SectorSnapshot] = []

    for sector, symbols in SECTOR_MAP.items():
        valid = [s for s in symbols if s in returns.index]
        if not valid:
            continue

        sector_returns = returns[valid]
        avg_change = float(sector_returns.mean())
        lead = valid[int(sector_returns.abs().argmax())] if len(valid) > 0 else None

        rotation = None
        if avg_change > 0.05:
            rotation = "sector_momentum_up"
        elif avg_change < -0.05:
            rotation = "sector_momentum_down"

        snapshots.append(SectorSnapshot(
            sector=sector,
            symbols=valid,
            avg_change=avg_change,
            lead_symbol=lead,
            rotation_signal=rotation,
        ))

    snapshots.sort(key=lambda s: s.avg_change, reverse=True)
    return snapshots


def find_lead_lag_pairs(
    price_df: pd.DataFrame,
    max_lag: int = 5,
    top_n: int = 10,
) -> list[CorrelationPair]:
    """Find lead-lag relationships between symbol pairs using cross-correlation."""
    returns = price_df.pct_change().dropna()
    symbols = returns.columns.tolist()
    pairs: list[CorrelationPair] = []

    for i in range(len(symbols)):
        for j in range(i + 1, len(symbols)):
            a, b = symbols[i], symbols[j]
            cross_corr = []
            for lag in range(-max_lag, max_lag + 1):
                shifted = returns[a].shift(lag)
                corr = shifted.corr(returns[b])
                cross_corr.append((lag, corr if not np.isnan(corr) else 0))

            best_lag, best_corr = max(cross_corr, key=lambda x: abs(x[1]))

            for sector, members in SECTOR_MAP.items():
                if a in members and b in members:
                    same_sector = sector
                    break
            else:
                same_sector = None

            pairs.append(CorrelationPair(
                symbol_a=a,
                symbol_b=b,
                correlation=best_corr,
                correlation_type=classify_correlation(best_corr),
                window=best_lag,
                sector=same_sector,
            ))

    pairs.sort(key=lambda p: abs(p.correlation), reverse=True)
    return pairs[:top_n]
