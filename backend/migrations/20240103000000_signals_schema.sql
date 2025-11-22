-- ============================================================================
-- SIGNALS SCHEMA: Real Signal Ingestion (No AI, No Mocks)
-- ============================================================================

-- Signal types enum
DO $$ BEGIN
    CREATE TYPE signal_type AS ENUM (
        'hiring',           -- Job postings detected
        'funding',          -- Funding announcement
        'github_activity',  -- Repository activity spike
        'tech_adoption',    -- New technology adoption
        'expansion',        -- Office/team expansion
        'product_launch'    -- New product/feature launch
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Signal source enum
DO $$ BEGIN
    CREATE TYPE signal_source AS ENUM (
        'wellfound',        -- Wellfound (AngelList) job postings
        'github',           -- GitHub API
        'rss_feed',         -- RSS/Blog parsing
        'twitter',          -- Twitter/X activity
        'linkedin',         -- LinkedIn (future)
        'crunchbase',       -- Crunchbase (future)
        'manual'            -- Manually added
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- ============================================================================
-- Companies Table: Real Web3 companies we track
-- ============================================================================
CREATE TABLE IF NOT EXISTS companies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    domain VARCHAR(255) UNIQUE NOT NULL,
    logo_url TEXT,
    description TEXT,
    industry VARCHAR(100) DEFAULT 'web3',
    employee_count_range VARCHAR(50),  -- '1-10', '11-50', '51-200', etc.
    founded_year INTEGER,
    headquarters VARCHAR(255),
    
    -- Social/tracking links
    website_url TEXT,
    github_org VARCHAR(255),           -- GitHub organization name
    twitter_handle VARCHAR(255),
    linkedin_url TEXT,
    wellfound_slug VARCHAR(255),       -- For Wellfound scraping
    
    -- Tracking metadata
    is_active BOOLEAN DEFAULT TRUE,
    last_scraped_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_companies_domain ON companies(domain);
CREATE INDEX IF NOT EXISTS idx_companies_github ON companies(github_org);
CREATE INDEX IF NOT EXISTS idx_companies_wellfound ON companies(wellfound_slug);
CREATE INDEX IF NOT EXISTS idx_companies_active ON companies(is_active);

-- ============================================================================
-- Signals Table: Real detected signals
-- ============================================================================
CREATE TABLE IF NOT EXISTS signals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    
    -- Signal classification
    signal_type VARCHAR(50) NOT NULL,   -- hiring, funding, github_activity, etc.
    source VARCHAR(50) NOT NULL,        -- wellfound, github, rss_feed, etc.
    
    -- Signal data
    title VARCHAR(500) NOT NULL,        -- "Hiring Senior Solidity Developer"
    description TEXT,                   -- Full details
    source_url TEXT,                    -- Link to original source
    raw_data JSONB DEFAULT '{}',        -- Full scraped data for debugging
    
    -- Confidence scoring (rule-based, NOT AI)
    confidence_score DECIMAL(3,2) DEFAULT 0.50,  -- 0.00 to 1.00
    confidence_factors JSONB DEFAULT '{}',       -- Explainable factors
    
    -- Timing
    detected_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    signal_date DATE,                   -- When the signal actually occurred
    expires_at TIMESTAMP WITH TIME ZONE, -- When signal becomes stale
    
    -- Status
    is_verified BOOLEAN DEFAULT FALSE,  -- Manual verification flag
    is_published BOOLEAN DEFAULT TRUE,  -- Show on public feed
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_signals_company ON signals(company_id);
CREATE INDEX IF NOT EXISTS idx_signals_type ON signals(signal_type);
CREATE INDEX IF NOT EXISTS idx_signals_source ON signals(source);
CREATE INDEX IF NOT EXISTS idx_signals_detected ON signals(detected_at DESC);
CREATE INDEX IF NOT EXISTS idx_signals_published ON signals(is_published, detected_at DESC);
CREATE INDEX IF NOT EXISTS idx_signals_confidence ON signals(confidence_score DESC);

-- ============================================================================
-- Hiring Signals Detail Table (for job postings)
-- ============================================================================
CREATE TABLE IF NOT EXISTS hiring_signals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    signal_id UUID NOT NULL REFERENCES signals(id) ON DELETE CASCADE,
    
    job_title VARCHAR(255) NOT NULL,
    department VARCHAR(100),
    location VARCHAR(255),
    job_type VARCHAR(50),               -- full-time, contract, etc.
    experience_level VARCHAR(50),       -- junior, mid, senior, lead
    salary_range VARCHAR(100),
    
    -- Keywords for scoring
    keywords TEXT[],                    -- ['solidity', 'rust', 'defi']
    is_web3_role BOOLEAN DEFAULT FALSE,
    
    posted_date DATE,
    source_job_id VARCHAR(255),         -- ID from source platform
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_hiring_signals_signal ON hiring_signals(signal_id);
CREATE INDEX IF NOT EXISTS idx_hiring_signals_web3 ON hiring_signals(is_web3_role);

-- ============================================================================
-- GitHub Activity Signals Detail Table
-- ============================================================================
CREATE TABLE IF NOT EXISTS github_signals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    signal_id UUID NOT NULL REFERENCES signals(id) ON DELETE CASCADE,
    
    repo_name VARCHAR(255) NOT NULL,
    repo_url TEXT,
    
    -- Activity metrics
    commits_last_7d INTEGER DEFAULT 0,
    commits_last_30d INTEGER DEFAULT 0,
    stars_count INTEGER DEFAULT 0,
    stars_gained_7d INTEGER DEFAULT 0,
    forks_count INTEGER DEFAULT 0,
    contributors_count INTEGER DEFAULT 0,
    open_issues INTEGER DEFAULT 0,
    
    -- Latest activity
    last_commit_at TIMESTAMP WITH TIME ZONE,
    last_release_at TIMESTAMP WITH TIME ZONE,
    last_release_tag VARCHAR(100),
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_github_signals_signal ON github_signals(signal_id);
CREATE INDEX IF NOT EXISTS idx_github_signals_commits ON github_signals(commits_last_7d DESC);

-- ============================================================================
-- Funding Signals Detail Table
-- ============================================================================
CREATE TABLE IF NOT EXISTS funding_signals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    signal_id UUID NOT NULL REFERENCES signals(id) ON DELETE CASCADE,
    
    round_type VARCHAR(50),             -- pre-seed, seed, series-a, etc.
    amount_usd BIGINT,                  -- Amount in USD cents
    amount_display VARCHAR(50),         -- "$10M"
    
    investors TEXT[],                   -- Array of investor names
    lead_investor VARCHAR(255),
    
    announced_date DATE,
    source_article_url TEXT,
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_funding_signals_signal ON funding_signals(signal_id);
CREATE INDEX IF NOT EXISTS idx_funding_signals_round ON funding_signals(round_type);

-- ============================================================================
-- Scraper State Table (track what we've already scraped)
-- ============================================================================
CREATE TABLE IF NOT EXISTS scraper_state (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source VARCHAR(50) NOT NULL,        -- wellfound, github, etc.
    company_id UUID REFERENCES companies(id) ON DELETE CASCADE,
    
    last_scraped_at TIMESTAMP WITH TIME ZONE,
    last_successful_at TIMESTAMP WITH TIME ZONE,
    last_error TEXT,
    error_count INTEGER DEFAULT 0,
    
    -- Pagination/cursor state
    cursor_state JSONB DEFAULT '{}',
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    UNIQUE(source, company_id)
);

CREATE INDEX IF NOT EXISTS idx_scraper_state_source ON scraper_state(source);
CREATE INDEX IF NOT EXISTS idx_scraper_state_company ON scraper_state(company_id);

-- ============================================================================
-- Seed: Initial Web3 Companies (REAL companies, not mocks)
-- ============================================================================
INSERT INTO companies (name, domain, github_org, twitter_handle, wellfound_slug, industry, description) VALUES
    -- DeFi Protocols
    ('Aave', 'aave.com', 'aave', 'aaborrowingpower', 'aave', 'defi', 'Open source liquidity protocol'),
    ('Uniswap', 'uniswap.org', 'Uniswap', 'Uniswap', 'uniswap-labs', 'defi', 'Decentralized trading protocol'),
    ('Compound', 'compound.finance', 'compound-finance', 'compoundfinance', 'compound-labs', 'defi', 'Algorithmic money markets'),
    ('MakerDAO', 'makerdao.com', 'makerdao', 'MakerDAO', 'makerdao', 'defi', 'Decentralized stablecoin'),
    ('Lido', 'lido.fi', 'lidofinance', 'LidoFinance', 'lido', 'defi', 'Liquid staking solution'),
    ('Curve', 'curve.fi', 'curvefi', 'CurveFinance', 'curve-finance', 'defi', 'Stablecoin exchange'),
    ('Yearn', 'yearn.fi', 'yearn', 'yeaborrowingpower', 'yearn-finance', 'defi', 'Yield aggregator'),
    ('Balancer', 'balancer.fi', 'balancer-labs', 'Balancer', 'balancer-labs', 'defi', 'Automated portfolio manager'),
    
    -- L1/L2 Chains
    ('Arbitrum', 'arbitrum.io', 'OffchainLabs', 'arbitrum', 'offchain-labs', 'infrastructure', 'Ethereum L2 scaling'),
    ('Optimism', 'optimism.io', 'ethereum-optimism', 'optimaborrowingpower', 'optimism-pbc', 'infrastructure', 'Ethereum L2 scaling'),
    ('Polygon', 'polygon.technology', 'maticnetwork', '0xPolygon', 'polygon-technology', 'infrastructure', 'Ethereum scaling platform'),
    ('StarkWare', 'starkware.co', 'starkware-libs', 'StarkWareLtd', 'starkware', 'infrastructure', 'ZK rollup technology'),
    ('zkSync', 'zksync.io', 'matter-labs', 'zaborrowingpower', 'matter-labs', 'infrastructure', 'ZK rollup L2'),
    ('Scroll', 'scroll.io', 'scroll-tech', 'Scroll_ZKP', 'scroll', 'infrastructure', 'zkEVM L2'),
    
    -- NFT/Gaming
    ('OpenSea', 'opensea.io', 'ProjectOpenSea', 'opensea', 'opensea', 'nft', 'NFT marketplace'),
    ('Blur', 'blur.io', 'blur-io', 'blur_io', 'blur', 'nft', 'NFT marketplace'),
    ('Immutable', 'immutable.com', 'immutable', 'Immutable', 'immutable', 'gaming', 'Web3 gaming platform'),
    ('Sky Mavis', 'skymavis.com', 'axieinfinity', 'SkyMavisHQ', 'sky-mavis', 'gaming', 'Axie Infinity creators'),
    
    -- Infrastructure
    ('Alchemy', 'alchemy.com', 'alchemyplatform', 'AlchemyPlatform', 'alchemy', 'infrastructure', 'Web3 development platform'),
    ('Infura', 'infura.io', 'INFURA', 'infaborrowingpower_io', 'infura', 'infrastructure', 'Ethereum API provider'),
    ('The Graph', 'thegraph.com', 'graphprotocol', 'graphprotocol', 'the-graph', 'infrastructure', 'Indexing protocol'),
    ('Chainlink', 'chain.link', 'smartcontractkit', 'chainlink', 'chainlink-labs', 'infrastructure', 'Oracle network'),
    ('Moralis', 'moralis.io', 'MoralisWeb3', 'MoralisWeb3', 'moralis', 'infrastructure', 'Web3 development platform'),
    
    -- Wallets
    ('MetaMask', 'metamask.io', 'MetaMask', 'MetaMask', 'metamask', 'wallet', 'Ethereum wallet'),
    ('Rainbow', 'rainbow.me', 'rainbow-me', 'rainbowdotme', 'rainbow', 'wallet', 'Ethereum wallet'),
    ('Phantom', 'phantom.app', 'phantom', 'phantom', 'phantom', 'wallet', 'Multi-chain wallet'),
    
    -- Security
    ('OpenZeppelin', 'openzeppelin.com', 'OpenZeppelin', 'OpenZeppelin', 'openzeppelin', 'security', 'Smart contract security'),
    ('Trail of Bits', 'trailofbits.com', 'trailofbits', 'traborrowingpowerlofbits', 'trail-of-bits', 'security', 'Security research'),
    ('Certik', 'certik.com', 'CertiKProject', 'CertiK', 'certik', 'security', 'Blockchain security'),
    
    -- Data/Analytics
    ('Dune', 'dune.com', 'duneanalytics', 'DuneAnalytics', 'dune', 'analytics', 'Blockchain analytics'),
    ('Nansen', 'nansen.ai', 'nansen-ai', 'naborrowingpower', 'nansen', 'analytics', 'On-chain analytics'),
    ('Messari', 'messari.io', 'messari', 'MessariCrypto', 'messari', 'analytics', 'Crypto research')
ON CONFLICT (domain) DO NOTHING;

-- ============================================================================
-- View: Public Signal Feed (for the landing page)
-- ============================================================================
CREATE OR REPLACE VIEW public_signal_feed AS
SELECT 
    s.id,
    s.signal_type,
    s.source,
    s.title,
    s.description,
    s.source_url,
    s.confidence_score,
    s.detected_at,
    s.signal_date,
    c.id as company_id,
    c.name as company_name,
    c.domain as company_domain,
    c.logo_url as company_logo,
    c.industry
FROM signals s
JOIN companies c ON s.company_id = c.id
WHERE s.is_published = TRUE
  AND c.is_active = TRUE
  AND (s.expires_at IS NULL OR s.expires_at > NOW())
ORDER BY s.detected_at DESC;
