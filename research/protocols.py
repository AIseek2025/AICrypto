"""AICrypto protocol definitions for Python research layer."""

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Optional


class SourceType(str, Enum):
    EXCHANGE = "exchange"
    SOCIAL = "social"
    NEWS = "news"
    ONCHAIN = "onchain"
    SYSTEM = "system"


@dataclass
class CanonicalEvent:
    schema_name: str = "canonical_event"
    schema_version: str = "v1"
    event_id: str = ""
    trace_id: str = ""
    source_type: SourceType = SourceType.EXCHANGE
    source_name: str = ""
    event_type: str = ""
    symbol: Optional[str] = None
    ts_event: int = 0
    ts_ingested: int = 0
    payload: dict[str, Any] = field(default_factory=dict)
    quality_flags: list[str] = field(default_factory=list)
    tags: list[str] = field(default_factory=list)


@dataclass
class MarketSnapshot:
    schema_name: str = "market_snapshot"
    schema_version: str = "v1"
    symbol: str = ""
    exchange: str = "binance"
    market_type: str = "usds_m_futures"
    last_price: str = "0"
    mark_price: Optional[str] = None
    index_price: Optional[str] = None
    funding_rate: Optional[str] = None
    open_interest: Optional[str] = None
    volume_24h: Optional[str] = None
    ts_snapshot: int = 0


@dataclass
class FeatureVector:
    schema_name: str = "feature_vector"
    schema_version: str = "v1"
    feature_set: str = ""
    feature_version: str = "v1"
    symbol: str = ""
    window: str = ""
    ts_feature: int = 0
    features: dict[str, Any] = field(default_factory=dict)
    source_refs: list[str] = field(default_factory=list)


class Direction(str, Enum):
    LONG = "LONG"
    SHORT = "SHORT"
    NEUTRAL = "NEUTRAL"


class SignalType(str, Enum):
    ENTRY = "entry"
    EXIT = "exit"
    ADD = "add"
    REDUCE = "reduce"
    RISK_ALERT = "risk_alert"


@dataclass
class SignalEvent:
    signal_id: str = ""
    signal_type: SignalType = SignalType.ENTRY
    symbol: str = ""
    direction: Direction = Direction.LONG
    confidence: float = 0.0
    horizon: str = "swing"
    reason_codes: list[str] = field(default_factory=list)
    evidence_refs: list[str] = field(default_factory=list)
    ts_signal: int = 0


class SkillFamily(str, Enum):
    TREND = "trend"
    SHORT = "short"
    CORRELATION = "correlation"
    RISK = "risk"


@dataclass
class SkillSpec:
    skill_id: str = ""
    skill_name: str = ""
    skill_family: SkillFamily = SkillFamily.TREND
    entry_conditions: dict[str, Any] = field(default_factory=dict)
    position_rules: dict[str, Any] = field(default_factory=dict)
    add_rules: Optional[dict[str, Any]] = None
    reduce_rules: Optional[dict[str, Any]] = None
    exit_rules: dict[str, Any] = field(default_factory=dict)
    risk_rules: dict[str, Any] = field(default_factory=dict)
    applicable_market_states: list[str] = field(default_factory=list)
    input_contract: dict[str, Any] = field(default_factory=dict)
    output_contract: dict[str, Any] = field(default_factory=dict)
    status: str = "draft"
    version: str = "v1"
