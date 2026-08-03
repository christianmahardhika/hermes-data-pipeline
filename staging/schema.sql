-- Hermes Data Pipeline - SQLite Staging Schema
-- Temporary storage for raw data before processing
-- Purge policy: 7 days TTL for processed records

-- Raw finance data (stocks, prices, fundamentals)
CREATE TABLE IF NOT EXISTS raw_finance_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL,              -- 'yahoo', 'idx', 'fincept', 'bps'
    symbol TEXT,                        -- 'BBRI', 'INCO', etc.
    data_type TEXT NOT NULL,            -- 'price', 'volume', 'fundamental', 'correlation'
    raw_json TEXT NOT NULL,             -- Original JSON from API
    fetched_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    processed_at TIMESTAMP,             -- NULL if not processed
    status TEXT DEFAULT 'pending',      -- 'pending', 'processed', 'error'
    error_message TEXT                  -- Error details if status='error'
);

-- Raw news data (RSS, social media, web scraping)
CREATE TABLE IF NOT EXISTS raw_news_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL,               -- 'rss', 'hackernews', 'reddit', 'youtube'
    title TEXT,
    content TEXT,
    url TEXT UNIQUE,                    -- Prevent duplicates
    author TEXT,
    published_at TIMESTAMP,
    fetched_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    processed_at TIMESTAMP,
    status TEXT DEFAULT 'pending',
    error_message TEXT
);

-- Raw economic indicators (BPS, BOJ, FDIC, etc)
CREATE TABLE IF NOT EXISTS raw_economic_indicators (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL,               -- 'BPS', 'BOJ', 'FDIC', 'AU_GOV'
    indicator_id TEXT,                  -- BPS var_id, BOJ series code, etc
    indicator_name TEXT,
    value REAL,
    unit TEXT,
    country TEXT,                       -- 'ID', 'JP', 'US', 'AU'
    date TIMESTAMP NOT NULL,            -- Date of the data point
    raw_json TEXT,                      -- Original JSON from API
    fetched_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    processed_at TIMESTAMP,
    status TEXT DEFAULT 'pending',
    error_message TEXT
);

-- Raw commodity prices (coal, nickel, gold, etc)
CREATE TABLE IF NOT EXISTS raw_commodity_prices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL,               -- 'AU_GOV', 'LME', 'COMEX'
    commodity TEXT NOT NULL,            -- 'coal', 'nickel', 'gold', 'palm_oil'
    price_usd REAL,
    price_idr REAL,
    unit TEXT,
    date TIMESTAMP NOT NULL,
    raw_json TEXT,                      -- Original JSON from API
    fetched_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    processed_at TIMESTAMP,
    status TEXT DEFAULT 'pending',
    error_message TEXT
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_finance_status ON raw_finance_data(status);
CREATE INDEX IF NOT EXISTS idx_finance_symbol ON raw_finance_data(symbol);
CREATE INDEX IF NOT EXISTS idx_finance_fetched ON raw_finance_data(fetched_at);

CREATE INDEX IF NOT EXISTS idx_news_status ON raw_news_data(status);
CREATE INDEX IF NOT EXISTS idx_news_fetched ON raw_news_data(fetched_at);
CREATE INDEX IF NOT EXISTS idx_news_source ON raw_news_data(source);

CREATE INDEX IF NOT EXISTS idx_economic_status ON raw_economic_indicators(status);
CREATE INDEX IF NOT EXISTS idx_economic_source ON raw_economic_indicators(source);
CREATE INDEX IF NOT EXISTS idx_economic_date ON raw_economic_indicators(date);

CREATE INDEX IF NOT EXISTS idx_commodity_status ON raw_commodity_prices(status);
CREATE INDEX IF NOT EXISTS idx_commodity_name ON raw_commodity_prices(commodity);
CREATE INDEX IF NOT EXISTS idx_commodity_date ON raw_commodity_prices(date);

-- View for monitoring pending records
CREATE VIEW IF NOT EXISTS staging_status AS
SELECT 
    'finance' as table_name,
    COUNT(*) as total_records,
    SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) as pending,
    SUM(CASE WHEN status = 'processed' THEN 1 ELSE 0 END) as processed,
    SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END) as errors
FROM raw_finance_data
UNION ALL
SELECT 
    'news' as table_name,
    COUNT(*) as total_records,
    SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) as pending,
    SUM(CASE WHEN status = 'processed' THEN 1 ELSE 0 END) as processed,
    SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END) as errors
FROM raw_news_data
UNION ALL
SELECT 
    'economic' as table_name,
    COUNT(*) as total_records,
    SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) as pending,
    SUM(CASE WHEN status = 'processed' THEN 1 ELSE 0 END) as processed,
    SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END) as errors
FROM raw_economic_indicators
UNION ALL
SELECT 
    'commodity' as table_name,
    COUNT(*) as total_records,
    SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) as pending,
    SUM(CASE WHEN status = 'processed' THEN 1 ELSE 0 END) as processed,
    SUM(CASE WHEN status = 'error' THEN 1 ELSE 0 END) as errors
FROM raw_commodity_prices;
