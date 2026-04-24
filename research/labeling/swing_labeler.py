"""主升浪/主跌浪样本标注工具。

用于从历史K线数据中识别和标注主升浪/主跌浪样本，
为后续策略研究提供标注数据集。
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


class TrendType(str, Enum):
    BULL = "bull"
    BEAR = "bear"
    NEUTRAL = "neutral"


class TrendPhase(str, Enum):
    ACCELERATION = "acceleration"
    MAIN_WAVE = "main_wave"
    EXHAUSTION = "exhaustion"
    PULLBACK = "pullback"


@dataclass
class TrendSample:
    sample_id: str = ""
    symbol: str = ""
    trend_type: TrendType = TrendType.BULL
    phase: TrendPhase = TrendPhase.MAIN_WAVE
    start_time: int = 0
    end_time: int = 0
    start_price: float = 0.0
    end_price: float = 0.0
    peak_price: float = 0.0
    trough_price: float = 0.0
    total_return: float = 0.0
    max_drawdown: float = 0.0
    duration_bars: int = 0
    volume_ratio: float = 1.0
    consecutive_up: int = 0
    consecutive_down: int = 0
    tags: list[str] = field(default_factory=list)


@dataclass
class KlineBar:
    time: int = 0
    open: float = 0.0
    high: float = 0.0
    low: float = 0.0
    close: float = 0.0
    volume: float = 0.0
    quote_volume: float = 0.0
    trades: int = 0


def label_main_swing(
    bars: list[KlineBar],
    min_return_pct: float = 0.15,
    min_duration: int = 10,
    max_drawdown_pct: float = 0.10,
) -> list[TrendSample]:
    """标注主升浪/主跌浪样本。

    Args:
        bars: K线数据列表
        min_return_pct: 最小涨跌幅阈值（默认15%）
        min_duration: 最小持续K线数（默认10）
        max_drawdown_pct: 最大回撤阈值（默认10%）

    Returns:
        标注的主升浪/主跌浪样本列表
    """
    if len(bars) < min_duration:
        return []

    samples = []
    window_size = min(50, len(bars))

    for start_idx in range(len(bars) - min_duration):
        best_sample = None

        for end_idx in range(start_idx + min_duration, min(start_idx + window_size, len(bars))):
            window = bars[start_idx:end_idx + 1]
            sample = _analyze_window(start_idx, end_idx, window, min_return_pct, max_drawdown_pct)
            if sample is not None:
                if best_sample is None or sample.total_return > best_sample.total_return:
                    best_sample = sample

        if best_sample is not None:
            samples.append(best_sample)

    return _deduplicate_samples(samples)


def _analyze_window(
    start_idx: int,
    end_idx: int,
    window: list[KlineBar],
    min_return: float,
    max_dd: float,
) -> Optional[TrendSample]:
    """分析一个窗口是否构成主升浪/主跌浪。"""
    start_price = window[0].close
    end_price = window[-1].close
    total_return = (end_price - start_price) / start_price if start_price > 0 else 0.0

    if abs(total_return) < min_return:
        return None

    is_bull = total_return > 0
    peak = max(b.high for b in window)
    trough = min(b.low for b in window)

    if is_bull:
        max_dd = (peak - trough) / peak if peak > 0 else 0.0
        if max_dd > 0.30:
            return None
    else:
        max_dd = (peak - trough) / trough if trough > 0 else 0.0
        if max_dd > 0.30:
            return None

    consecutive_up = 0
    consecutive_down = 0
    max_consec_up = 0
    max_consec_down = 0
    for i in range(1, len(window)):
        if window[i].close > window[i-1].close:
            consecutive_up += 1
            consecutive_down = 0
            max_consec_up = max(max_consec_up, consecutive_up)
        else:
            consecutive_down += 1
            consecutive_up = 0
            max_consec_down = max(max_consec_down, consecutive_down)

    avg_volume = sum(b.volume for b in window) / len(window) if window else 1.0
    first_half_vol = sum(b.volume for b in window[:len(window)//2]) / (len(window)//2) if len(window) >= 4 else 1.0
    vol_ratio = avg_volume / first_half_vol if first_half_vol > 0 else 1.0

    return TrendSample(
        sample_id=f"swing_{start_idx}_{end_idx}",
        symbol="",
        trend_type=TrendType.BULL if is_bull else TrendType.BEAR,
        phase=TrendPhase.MAIN_WAVE,
        start_time=window[0].time,
        end_time=window[-1].time,
        start_price=start_price,
        end_price=end_price,
        peak_price=peak,
        trough_price=trough,
        total_return=total_return,
        max_drawdown=max_dd,
        duration_bars=len(window),
        volume_ratio=vol_ratio,
        consecutive_up=max_consec_up,
        consecutive_down=max_consec_down,
        tags=["auto_labeled", "main_swing"],
    )


def _deduplicate_samples(samples: list[TrendSample]) -> list[TrendSample]:
    """去除重叠的样本，保留收益最大的。"""
    if not samples:
        return []

    samples.sort(key=lambda s: abs(s.total_return), reverse=True)
    result = []
    used_ranges: list[tuple[int, int]] = []

    for sample in samples:
        s_start = sample.start_time
        s_end = sample.end_time
        overlap = False
        for r_start, r_end in used_ranges:
            if s_start <= r_end and s_end >= r_start:
                overlap_pct = (min(s_end, r_end) - max(s_start, r_start)) / (s_end - s_start)
                if overlap_pct > 0.5:
                    overlap = True
                    break
        if not overlap:
            result.append(sample)
            used_ranges.append((s_start, s_end))

    return result


def print_sample_report(samples: list[TrendSample]) -> None:
    """打印样本报告。"""
    if not samples:
        print("No samples found.")
        return

    bull = [s for s in samples if s.trend_type == TrendType.BULL]
    bear = [s for s in samples if s.trend_type == TrendType.BEAR]

    print(f"=== Swing Labeling Report ===")
    print(f"Total samples: {len(samples)}")
    print(f"  Bull swings: {len(bull)}")
    print(f"  Bear swings: {len(bear)}")
    print()

    for s in samples[:20]:
        direction = "↑" if s.trend_type == TrendType.BULL else "↓"
        print(
            f"  {direction} {s.sample_id} | "
            f"ret={s.total_return:+.1%} | "
            f"dd={s.max_drawdown:.1%} | "
            f"bars={s.duration_bars} | "
            f"consec_up={s.consecutive_up} down={s.consecutive_down}"
        )
