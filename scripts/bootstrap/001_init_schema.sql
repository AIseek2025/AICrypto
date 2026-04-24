-- Symbol master table
CREATE TABLE IF NOT EXISTS symbols (
    symbol VARCHAR(32) PRIMARY KEY,
    contract_type VARCHAR(32) NOT NULL DEFAULT 'perpetual',
    underlying VARCHAR(32),
    quote_asset VARCHAR(16) NOT NULL DEFAULT 'USDT',
    margin_asset VARCHAR(16) NOT NULL DEFAULT 'USDT',
    status VARCHAR(16) NOT NULL DEFAULT 'trading',
    onboard_date TIMESTAMPTZ,
    price_precision INT NOT NULL DEFAULT 8,
    quantity_precision INT NOT NULL DEFAULT 8,
    tick_size DECIMAL(20, 10) NOT NULL DEFAULT 0.01,
    step_size DECIMAL(20, 10) NOT NULL DEFAULT 0.001,
    min_qty DECIMAL(20, 10),
    min_notional DECIMAL(20, 4),
    max_leverage INT DEFAULT 20,
    maint_margin_ratio DECIMAL(10, 6),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Kline table (TimescaleDB hypertable)
CREATE TABLE IF NOT EXISTS klines (
    time TIMESTAMPTZ NOT NULL,
    symbol VARCHAR(32) NOT NULL,
    interval VARCHAR(16) NOT NULL,
    open DECIMAL(20, 8) NOT NULL,
    high DECIMAL(20, 8) NOT NULL,
    low DECIMAL(20, 8) NOT NULL,
    close DECIMAL(20, 8) NOT NULL,
    volume DECIMAL(20, 8) NOT NULL,
    close_time TIMESTAMPTZ,
    quote_volume DECIMAL(20, 8),
    trades INT,
    taker_buy_volume DECIMAL(20, 8),
    PRIMARY KEY (symbol, interval, time)
);

SELECT create_hypertable('klines', 'time', if_not_exists => TRUE);

-- Mark Price table
CREATE TABLE IF NOT EXISTS mark_prices (
    time TIMESTAMPTZ NOT NULL,
    symbol VARCHAR(32) NOT NULL,
    mark_price DECIMAL(20, 8) NOT NULL,
    index_price DECIMAL(20, 8) NOT NULL,
    estimated_settle_price DECIMAL(20, 8),
    funding_rate DECIMAL(20, 10),
    next_funding_time TIMESTAMPTZ,
    PRIMARY KEY (symbol, time)
);

SELECT create_hypertable('mark_prices', 'time', if_not_exists => TRUE);

-- Funding Rate table
CREATE TABLE IF NOT EXISTS funding_rates (
    time TIMESTAMPTZ NOT NULL,
    symbol VARCHAR(32) NOT NULL,
    funding_rate DECIMAL(20, 10) NOT NULL,
    funding_time TIMESTAMPTZ NOT NULL,
    mark_price DECIMAL(20, 8),
    PRIMARY KEY (symbol, time)
);

SELECT create_hypertable('funding_rates', 'time', if_not_exists => TRUE);

-- Open Interest table
CREATE TABLE IF NOT EXISTS open_interests (
    time TIMESTAMPTZ NOT NULL,
    symbol VARCHAR(32) NOT NULL,
    open_interest DECIMAL(20, 8) NOT NULL,
    notional_value DECIMAL(20, 4),
    PRIMARY KEY (symbol, time)
);

SELECT create_hypertable('open_interests', 'time', if_not_exists => TRUE);

-- Orders table
CREATE TABLE IF NOT EXISTS orders (
    order_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    intent_id VARCHAR(128) NOT NULL,
    symbol VARCHAR(32) NOT NULL,
    side VARCHAR(8) NOT NULL,
    position_side VARCHAR(8) NOT NULL,
    order_type VARCHAR(32) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    price_limit DECIMAL(20, 8),
    status VARCHAR(32) NOT NULL DEFAULT 'created',
    filled_qty DECIMAL(20, 8) DEFAULT 0,
    avg_fill_price DECIMAL(20, 8),
    exchange_order_id VARCHAR(128),
    reduce_only BOOLEAN DEFAULT FALSE,
    origin_ref VARCHAR(256),
    trace_id VARCHAR(128),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_orders_symbol ON orders(symbol);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created_at ON orders(created_at);

-- Positions table
CREATE TABLE IF NOT EXISTS positions (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(32) NOT NULL,
    side VARCHAR(8) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    entry_price DECIMAL(20, 8) NOT NULL,
    mark_price DECIMAL(20, 8),
    unrealized_pnl DECIMAL(20, 8),
    leverage INT,
    margin DECIMAL(20, 8),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(symbol, side)
);

-- Skill Registry table
CREATE TABLE IF NOT EXISTS skills (
    skill_id VARCHAR(128) PRIMARY KEY,
    skill_name VARCHAR(256) NOT NULL,
    skill_family VARCHAR(32) NOT NULL,
    version VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'draft',
    spec JSONB NOT NULL,
    backtest_report_ref VARCHAR(256),
    paper_trade_report_ref VARCHAR(256),
    owner VARCHAR(128),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Signal Events table
CREATE TABLE IF NOT EXISTS signal_events (
    signal_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    signal_type VARCHAR(32) NOT NULL,
    symbol VARCHAR(32) NOT NULL,
    direction VARCHAR(8) NOT NULL,
    confidence DECIMAL(5, 4) NOT NULL,
    horizon VARCHAR(32),
    reason_codes JSONB,
    evidence_refs JSONB,
    skill_id VARCHAR(128),
    ts_signal TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_signals_symbol ON signal_events(symbol);
CREATE INDEX idx_signals_ts ON signal_events(ts_signal);

-- Risk Decisions table
CREATE TABLE IF NOT EXISTS risk_decisions (
    decision_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    target_ref VARCHAR(128) NOT NULL,
    decision VARCHAR(16) NOT NULL,
    severity VARCHAR(16) NOT NULL,
    rule_hits JSONB,
    exposure_snapshot JSONB,
    required_actions JSONB,
    review_required BOOLEAN DEFAULT FALSE,
    ts_decision TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Audit Log table
CREATE TABLE IF NOT EXISTS audit_log (
    id BIGSERIAL PRIMARY KEY,
    action VARCHAR(64) NOT NULL,
    actor VARCHAR(128),
    target_type VARCHAR(64),
    target_id VARCHAR(256),
    details JSONB,
    trace_id VARCHAR(128),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_created_at ON audit_log(created_at);
CREATE INDEX idx_audit_action ON audit_log(action);
