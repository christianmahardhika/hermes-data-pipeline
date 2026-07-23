# Bugfix Requirements Document

## Introduction

The Hermes news pipeline's RSS collector lacks resilience mechanisms, causing degraded intelligence coverage when feeds become unreachable. Without a circuit breaker, the collector wastes 60 seconds per dead feed per cycle attempting requests that will never succeed. Additionally, there is no automated freshness monitoring to detect "silent deaths" (feeds returning HTTP 200 but no new content), no feed health CI for proactive detection, and no category-based observability. This results in wasted compute, delayed collection cycles, and undetected coverage gaps.

## Bug Analysis

### Current Behavior (Defect)

1.1 WHEN a feed has consecutive_failures >= 5 in the feed_health table THEN the system still attempts to fetch that feed on every collection cycle, wasting up to 60 seconds per dead feed per cycle on timeout

1.2 WHEN a feed returns HTTP 200 but has not produced new articles for 24+ hours THEN the system reports no warning and treats the feed as healthy (silent death)

1.3 WHEN a feed URL becomes permanently unreachable (404, 403, or timeout) THEN the system logs the failure but has no automated CI workflow to detect and alert on accumulated degradation

1.4 WHEN collection stats are reported THEN the system shows only aggregate success/error counts with no per-category breakdown (Indonesian, International Business, Asia Pacific, etc.)

1.5 WHEN a feed transitions between healthy and unhealthy states THEN the system has no circuit state tracking (OPEN/HALF-OPEN/CLOSED) and no backoff escalation beyond the per-request exponential backoff

### Expected Behavior (Correct)

2.1 WHEN a feed has consecutive_failures >= 5 THEN the system SHALL skip that feed (circuit OPEN) for 60 minutes, then attempt ONE probe request (HALF-OPEN), and if successful reset to CLOSED; if the probe fails, extend the skip period with exponential backoff up to a 6-hour cap

2.2 WHEN a feed returns HTTP 200 but has not produced new articles for 24+ hours (and its historical average is > 0 articles/day) THEN the system SHALL flag that feed as "stale" in freshness tracking and report it in health checks

2.3 WHEN feed health validation runs (daily CI or on-demand) THEN the system SHALL curl every configured feed URL, record HTTP status and response time, and alert if overall feed success rate drops below 80%

2.4 WHEN collection stats are reported THEN the system SHALL include per-category breakdown showing success/failure/skipped counts for each feed category (Indonesian, InternationalBusiness, InternationalGeneral, AsiaPacific, Market, Tech)

2.5 WHEN a feed's circuit state transitions (CLOSED→OPEN, OPEN→HALF-OPEN, HALF-OPEN→CLOSED or HALF-OPEN→OPEN) THEN the system SHALL log the transition with feed name, failure count, and next retry time, and persist circuit state in SQLite so it survives daemon restarts

### Unchanged Behavior (Regression Prevention)

3.1 WHEN a feed URL is reachable and returns valid RSS/Atom XML THEN the system SHALL CONTINUE TO fetch, parse, and store the feed content with the existing retry-with-backoff mechanism

3.2 WHEN a feed's primary URL fails but fallback_urls are configured and reachable THEN the system SHALL CONTINUE TO attempt fallback URLs in order and succeed if any fallback responds

3.3 WHEN a feed fetch succeeds THEN the system SHALL CONTINUE TO reset consecutive_failures to 0 and update last_success in the feed_health table

3.4 WHEN a feed fetch fails THEN the system SHALL CONTINUE TO increment consecutive_failures and record last_error in the feed_health table

3.5 WHEN the collector completes a cycle THEN the system SHALL CONTINUE TO report aggregate CollectStats (total success and error counts) in addition to any new per-category stats

3.6 WHEN the self-healing monitor detects unhealthy feeds (consecutive_failures >= threshold) THEN the system SHALL CONTINUE TO send Telegram alerts for critical feed degradation
