#!/usr/bin/env python3
"""
Hermes Intelligence Pipeline Alert System
Phase 8 Task 39: Comprehensive Alerting & Notification System

Advanced alert system for pipeline re-architecture with:
- Real-time threshold monitoring
- Multi-channel notification (Email, Slack, Telegram)
- Prof Jiang framework alert correlation
- Indonesian market intelligence alerts
- Security incident detection
- Escalation management
"""

import asyncio
import json
import logging
import smtplib
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Callable
from dataclasses import dataclass, asdict
from email.mime.text import MimeText
from email.mime.multipart import MimeMultipart
from pathlib import Path
import aiohttp
import requests
from enum import Enum

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class AlertSeverity(Enum):
    """Alert severity levels"""
    INFO = "info"
    WARNING = "warning"
    CRITICAL = "critical"
    EMERGENCY = "emergency"

class AlertChannel(Enum):
    """Alert notification channels"""
    EMAIL = "email"
    SLACK = "slack"
    TELEGRAM = "telegram"
    WEBHOOK = "webhook"
    SMS = "sms"

@dataclass
class Alert:
    """Alert data structure"""
    id: str
    title: str
    description: str
    severity: AlertSeverity
    source: str
    timestamp: datetime
    tags: List[str]
    metadata: Dict[str, Any]
    resolved: bool = False
    acknowledged: bool = False
    escalation_level: int = 0
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for serialization"""
        return {
            **asdict(self),
            'timestamp': self.timestamp.isoformat(),
            'severity': self.severity.value
        }

@dataclass
class AlertRule:
    """Alert rule configuration"""
    id: str
    name: str
    description: str
    condition: str  # Python expression
    severity: AlertSeverity
    channels: List[AlertChannel]
    cooldown_minutes: int
    escalation_minutes: int
    enabled: bool = True
    
class HermesAlertSystem:
    """Comprehensive alert system for Hermes Intelligence Pipeline"""
    
    def __init__(self, config_path: str = "alert_config.json"):
        self.config = self.load_config(config_path)
        self.active_alerts: Dict[str, Alert] = {}
        self.alert_rules: List[AlertRule] = []
        self.cooldown_cache: Dict[str, datetime] = {}
        self.notification_handlers: Dict[AlertChannel, Callable] = {
            AlertChannel.EMAIL: self.send_email_alert,
            AlertChannel.SLACK: self.send_slack_alert,
            AlertChannel.TELEGRAM: self.send_telegram_alert,
            AlertChannel.WEBHOOK: self.send_webhook_alert,
        }
        self.load_alert_rules()
        
    def load_config(self, config_path: str) -> Dict[str, Any]:
        """Load alert configuration"""
        default_config = {
            "email": {
                "smtp_server": "smtp.gmail.com",
                "smtp_port": 587,
                "username": "",
                "password": "",
                "from_address": "alerts@hermes-intelligence.com",
                "to_addresses": ["admin@hermes-intelligence.com"]
            },
            "slack": {
                "webhook_url": "",
                "channel": "#hermes-alerts",
                "username": "Hermes Alert Bot"
            },
            "telegram": {
                "bot_token": "",
                "chat_ids": []
            },
            "webhooks": {
                "primary": "",
                "backup": ""
            },
            "escalation": {
                "levels": [15, 60, 240],  # Minutes
                "emergency_contacts": []
            },
            "prof_jiang": {
                "geopolitical_threshold": 0.8,
                "confidence_threshold": 0.6,
                "enable_predictive_alerts": True
            },
            "indonesian_market": {
                "volatility_threshold": 5.0,  # Percent
                "correlation_break_threshold": 0.3,
                "enable_stock_alerts": True
            }
        }
        
        try:
            with open(config_path, 'r') as f:
                config = json.load(f)
                return {**default_config, **config}
        except FileNotFoundError:
            logger.warning(f"Alert config {config_path} not found, using defaults")
            return default_config
    
    def load_alert_rules(self):
        """Load predefined alert rules"""
        self.alert_rules = [
            # Service Health Rules
            AlertRule(
                id="service_down",
                name="Service Down Alert",
                description="Alert when any service becomes unhealthy",
                condition="service_status != 'healthy'",
                severity=AlertSeverity.CRITICAL,
                channels=[AlertChannel.EMAIL, AlertChannel.SLACK],
                cooldown_minutes=5,
                escalation_minutes=15
            ),
            
            AlertRule(
                id="high_response_time",
                name="High Response Time",
                description="Alert when service response time exceeds threshold",
                condition="response_time_ms > 1000",
                severity=AlertSeverity.WARNING,
                channels=[AlertChannel.SLACK],
                cooldown_minutes=10,
                escalation_minutes=30
            ),
            
            # Performance Rules
            AlertRule(
                id="high_cpu_usage",
                name="High CPU Usage",
                description="Alert when CPU usage exceeds 80%",
                condition="cpu_usage > 80",
                severity=AlertSeverity.WARNING,
                channels=[AlertChannel.EMAIL],
                cooldown_minutes=15,
                escalation_minutes=45
            ),
            
            AlertRule(
                id="high_memory_usage", 
                name="High Memory Usage",
                description="Alert when memory usage exceeds 85%",
                condition="memory_usage > 85",
                severity=AlertSeverity.CRITICAL,
                channels=[AlertChannel.EMAIL, AlertChannel.SLACK],
                cooldown_minutes=10,
                escalation_minutes=30
            ),
            
            AlertRule(
                id="queue_backup",
                name="Processing Queue Backup",
                description="Alert when processing queue depth is too high",
                condition="queue_depth > 1000",
                severity=AlertSeverity.WARNING,
                channels=[AlertChannel.SLACK],
                cooldown_minutes=20,
                escalation_minutes=60
            ),
            
            # Prof Jiang Framework Rules
            AlertRule(
                id="prof_jiang_low_confidence",
                name="Prof Jiang Low Confidence",
                description="Alert when Prof Jiang analysis confidence drops",
                condition="prof_jiang_confidence < 0.6",
                severity=AlertSeverity.INFO,
                channels=[AlertChannel.SLACK],
                cooldown_minutes=30,
                escalation_minutes=120
            ),
            
            AlertRule(
                id="geopolitical_tension_spike",
                name="Geopolitical Tension Spike",
                description="Alert when geopolitical tension levels spike",
                condition="geopolitical_tension > 0.8",
                severity=AlertSeverity.WARNING,
                channels=[AlertChannel.EMAIL, AlertChannel.TELEGRAM],
                cooldown_minutes=60,
                escalation_minutes=180
            ),
            
            # Indonesian Market Rules
            AlertRule(
                id="indonesian_stock_volatility",
                name="Indonesian Stock Volatility Alert",
                description="Alert when Indonesian stock volatility exceeds threshold",
                condition="stock_volatility > 5.0",
                severity=AlertSeverity.INFO,
                channels=[AlertChannel.TELEGRAM],
                cooldown_minutes=45,
                escalation_minutes=180
            ),
            
            AlertRule(
                id="correlation_breakdown",
                name="Stock Correlation Breakdown",
                description="Alert when expected stock correlations break down",
                condition="correlation_strength < 0.3",
                severity=AlertSeverity.WARNING,
                channels=[AlertChannel.EMAIL],
                cooldown_minutes=120,
                escalation_minutes=360
            ),
            
            # Security Rules
            AlertRule(
                id="security_breach_attempt",
                name="Security Breach Attempt",
                description="Alert on suspicious security activity",
                condition="failed_auth_attempts > 10",
                severity=AlertSeverity.EMERGENCY,
                channels=[AlertChannel.EMAIL, AlertChannel.SLACK, AlertChannel.TELEGRAM],
                cooldown_minutes=0,  # No cooldown for security
                escalation_minutes=5
            ),
            
            AlertRule(
                id="rate_limit_violations",
                name="Rate Limit Violations",
                description="Alert on excessive rate limit violations",
                condition="rate_limit_violations > 50",
                severity=AlertSeverity.WARNING,
                channels=[AlertChannel.SLACK],
                cooldown_minutes=30,
                escalation_minutes=90
            )
        ]
        
        logger.info(f"📋 Loaded {len(self.alert_rules)} alert rules")
    
    async def process_metrics(self, metrics: Dict[str, Any]):
        """Process incoming metrics and check alert rules"""
        logger.debug(f"🔍 Processing metrics: {len(metrics)} data points")
        
        for rule in self.alert_rules:
            if not rule.enabled:
                continue
                
            try:
                # Check if rule condition is met
                if self.evaluate_condition(rule.condition, metrics):
                    await self.trigger_alert(rule, metrics)
                    
            except Exception as e:
                logger.error(f"❌ Error evaluating rule {rule.id}: {e}")
    
    def evaluate_condition(self, condition: str, metrics: Dict[str, Any]) -> bool:
        """Safely evaluate alert condition"""
        try:
            # Create safe evaluation context
            safe_globals = {
                "__builtins__": {},
                "abs": abs,
                "max": max,
                "min": min,
                "len": len,
            }
            
            # Add metrics to local context
            local_context = metrics.copy()
            
            # Evaluate condition
            result = eval(condition, safe_globals, local_context)
            return bool(result)
            
        except Exception as e:
            logger.error(f"❌ Condition evaluation error: {e}")
            return False
    
    async def trigger_alert(self, rule: AlertRule, metrics: Dict[str, Any]):
        """Trigger an alert based on rule"""
        alert_key = f"{rule.id}_{rule.condition}"
        
        # Check cooldown
        if alert_key in self.cooldown_cache:
            last_alert = self.cooldown_cache[alert_key]
            if datetime.now() - last_alert < timedelta(minutes=rule.cooldown_minutes):
                return
        
        # Create alert
        alert = Alert(
            id=f"alert_{datetime.now().strftime('%Y%m%d_%H%M%S')}_{rule.id}",
            title=rule.name,
            description=self.format_alert_description(rule, metrics),
            severity=rule.severity,
            source=f"hermes-alerts/{rule.id}",
            timestamp=datetime.now(),
            tags=self.extract_alert_tags(rule, metrics),
            metadata=metrics.copy()
        )
        
        # Store alert
        self.active_alerts[alert.id] = alert
        self.cooldown_cache[alert_key] = datetime.now()
        
        logger.warning(f"🚨 Alert triggered: {alert.title} [{alert.severity.value.upper()}]")
        
        # Send notifications
        await self.send_notifications(alert, rule.channels)
        
        # Schedule escalation if configured
        if rule.escalation_minutes > 0:
            asyncio.create_task(self.schedule_escalation(alert, rule))
    
    def format_alert_description(self, rule: AlertRule, metrics: Dict[str, Any]) -> str:
        """Format alert description with current metrics"""
        description = f"{rule.description}\n\n"
        
        # Add relevant metrics
        relevant_metrics = self.extract_relevant_metrics(rule.condition, metrics)
        if relevant_metrics:
            description += "📊 Current Values:\n"
            for key, value in relevant_metrics.items():
                if isinstance(value, float):
                    description += f"• {key}: {value:.2f}\n"
                else:
                    description += f"• {key}: {value}\n"
        
        # Add timestamp
        description += f"\n🕐 Detected: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}"
        
        return description
    
    def extract_relevant_metrics(self, condition: str, metrics: Dict[str, Any]) -> Dict[str, Any]:
        """Extract metrics mentioned in the condition"""
        relevant = {}
        
        # Simple extraction based on common metric names
        metric_keys = [
            'cpu_usage', 'memory_usage', 'response_time_ms', 'queue_depth',
            'error_rate', 'prof_jiang_confidence', 'geopolitical_tension',
            'stock_volatility', 'correlation_strength', 'failed_auth_attempts',
            'rate_limit_violations'
        ]
        
        for key in metric_keys:
            if key in condition and key in metrics:
                relevant[key] = metrics[key]
        
        return relevant
    
    def extract_alert_tags(self, rule: AlertRule, metrics: Dict[str, Any]) -> List[str]:
        """Extract relevant tags for alert categorization"""
        tags = [rule.id, rule.severity.value]
        
        # Add service-specific tags
        if 'service_name' in metrics:
            tags.append(f"service:{metrics['service_name']}")
        
        # Add Prof Jiang tags
        if 'prof_jiang' in rule.id:
            tags.extend(['prof-jiang', 'geopolitical', 'intelligence'])
        
        # Add Indonesian market tags
        if 'indonesian' in rule.id or 'stock' in rule.id:
            tags.extend(['indonesian-market', 'stocks', 'portfolio'])
        
        # Add security tags
        if 'security' in rule.id or 'auth' in rule.id:
            tags.extend(['security', 'breach-attempt'])
            
        return tags
    
    async def send_notifications(self, alert: Alert, channels: List[AlertChannel]):
        """Send alert notifications to specified channels"""
        logger.info(f"📢 Sending notifications for alert {alert.id} to {len(channels)} channels")
        
        for channel in channels:
            try:
                handler = self.notification_handlers.get(channel)
                if handler:
                    await handler(alert)
                else:
                    logger.warning(f"⚠️  No handler for channel {channel}")
                    
            except Exception as e:
                logger.error(f"❌ Failed to send alert to {channel}: {e}")
    
    async def send_email_alert(self, alert: Alert):
        """Send email notification"""
        if not self.config["email"]["username"]:
            logger.warning("📧 Email configuration missing, skipping email alert")
            return
        
        try:
            msg = MimeMultipart()
            msg['From'] = self.config["email"]["from_address"]
            msg['To'] = ", ".join(self.config["email"]["to_addresses"])
            msg['Subject'] = f"🚨 Hermes Alert: {alert.title} [{alert.severity.value.upper()}]"
            
            # Create email body
            body = f"""
Hermes Intelligence Pipeline Alert

Alert ID: {alert.id}
Severity: {alert.severity.value.upper()}
Source: {alert.source}
Timestamp: {alert.timestamp.strftime('%Y-%m-%d %H:%M:%S')}

Description:
{alert.description}

Tags: {', '.join(alert.tags)}

---
This is an automated alert from Hermes Intelligence Pipeline Monitoring System.
Pipeline Re-Architecture Phase 8: Observability & Monitoring
            """.strip()
            
            msg.attach(MimeText(body, 'plain'))
            
            # Send email
            server = smtplib.SMTP(self.config["email"]["smtp_server"], self.config["email"]["smtp_port"])
            server.starttls()
            server.login(self.config["email"]["username"], self.config["email"]["password"])
            server.send_message(msg)
            server.quit()
            
            logger.info(f"✅ Email alert sent for {alert.id}")
            
        except Exception as e:
            logger.error(f"❌ Email send failed: {e}")
    
    async def send_slack_alert(self, alert: Alert):
        """Send Slack notification"""
        webhook_url = self.config["slack"]["webhook_url"]
        if not webhook_url:
            logger.warning("🔗 Slack webhook not configured, skipping Slack alert")
            return
        
        try:
            # Color coding by severity
            color_map = {
                AlertSeverity.INFO: "#36a64f",      # Green
                AlertSeverity.WARNING: "#ff9500",   # Orange  
                AlertSeverity.CRITICAL: "#ff0000",  # Red
                AlertSeverity.EMERGENCY: "#8b0000"  # Dark red
            }
            
            payload = {
                "channel": self.config["slack"]["channel"],
                "username": self.config["slack"]["username"],
                "attachments": [{
                    "color": color_map.get(alert.severity, "#cccccc"),
                    "title": f"🚨 {alert.title}",
                    "text": alert.description,
                    "fields": [
                        {
                            "title": "Severity",
                            "value": alert.severity.value.upper(),
                            "short": True
                        },
                        {
                            "title": "Source",
                            "value": alert.source,
                            "short": True
                        },
                        {
                            "title": "Alert ID",
                            "value": alert.id,
                            "short": True
                        },
                        {
                            "title": "Tags",
                            "value": ", ".join(alert.tags),
                            "short": True
                        }
                    ],
                    "footer": "Hermes Intelligence Pipeline",
                    "ts": int(alert.timestamp.timestamp())
                }]
            }
            
            async with aiohttp.ClientSession() as session:
                async with session.post(webhook_url, json=payload) as response:
                    if response.status == 200:
                        logger.info(f"✅ Slack alert sent for {alert.id}")
                    else:
                        logger.error(f"❌ Slack send failed with status {response.status}")
                        
        except Exception as e:
            logger.error(f"❌ Slack send failed: {e}")
    
    async def send_telegram_alert(self, alert: Alert):
        """Send Telegram notification"""
        bot_token = self.config["telegram"]["bot_token"]
        chat_ids = self.config["telegram"]["chat_ids"]
        
        if not bot_token or not chat_ids:
            logger.warning("📱 Telegram configuration missing, skipping Telegram alert")
            return
        
        try:
            # Format message
            emoji_map = {
                AlertSeverity.INFO: "ℹ️",
                AlertSeverity.WARNING: "⚠️", 
                AlertSeverity.CRITICAL: "🚨",
                AlertSeverity.EMERGENCY: "🔥"
            }
            
            emoji = emoji_map.get(alert.severity, "🔔")
            
            message = f"""
{emoji} *Hermes Alert*

*{alert.title}*
Severity: `{alert.severity.value.upper()}`
Source: `{alert.source}`
Time: `{alert.timestamp.strftime('%Y-%m-%d %H:%M:%S')}`

{alert.description}

Tags: `{', '.join(alert.tags)}`

_Hermes Intelligence Pipeline Monitoring_
            """.strip()
            
            # Send to all configured chat IDs
            for chat_id in chat_ids:
                url = f"https://api.telegram.org/bot{bot_token}/sendMessage"
                payload = {
                    "chat_id": chat_id,
                    "text": message,
                    "parse_mode": "Markdown"
                }
                
                async with aiohttp.ClientSession() as session:
                    async with session.post(url, json=payload) as response:
                        if response.status == 200:
                            logger.info(f"✅ Telegram alert sent to {chat_id} for {alert.id}")
                        else:
                            logger.error(f"❌ Telegram send failed to {chat_id}: {response.status}")
                            
        except Exception as e:
            logger.error(f"❌ Telegram send failed: {e}")
    
    async def send_webhook_alert(self, alert: Alert):
        """Send webhook notification"""
        primary_webhook = self.config["webhooks"]["primary"]
        if not primary_webhook:
            logger.warning("🔗 Webhook URL not configured, skipping webhook alert")
            return
        
        try:
            payload = alert.to_dict()
            
            async with aiohttp.ClientSession() as session:
                async with session.post(primary_webhook, json=payload) as response:
                    if response.status == 200:
                        logger.info(f"✅ Webhook alert sent for {alert.id}")
                    else:
                        logger.error(f"❌ Webhook send failed: {response.status}")
                        
        except Exception as e:
            logger.error(f"❌ Webhook send failed: {e}")
    
    async def schedule_escalation(self, alert: Alert, rule: AlertRule):
        """Schedule alert escalation"""
        await asyncio.sleep(rule.escalation_minutes * 60)
        
        # Check if alert is still active and not acknowledged
        if alert.id in self.active_alerts and not alert.acknowledged and not alert.resolved:
            logger.warning(f"⏰ Escalating unacknowledged alert: {alert.id}")
            
            # Increase escalation level
            alert.escalation_level += 1
            
            # Send escalated notification
            escalated_alert = Alert(
                id=f"{alert.id}_escalation_{alert.escalation_level}",
                title=f"ESCALATED: {alert.title}",
                description=f"Alert has been escalated (Level {alert.escalation_level})\n\n{alert.description}",
                severity=AlertSeverity.EMERGENCY if alert.escalation_level > 2 else AlertSeverity.CRITICAL,
                source=alert.source,
                timestamp=datetime.now(),
                tags=alert.tags + ["escalated", f"level-{alert.escalation_level}"],
                metadata=alert.metadata,
                escalation_level=alert.escalation_level
            )
            
            # Send to emergency contacts if available
            emergency_channels = [AlertChannel.EMAIL, AlertChannel.TELEGRAM]
            await self.send_notifications(escalated_alert, emergency_channels)
    
    def acknowledge_alert(self, alert_id: str, user: str = "system") -> bool:
        """Acknowledge an active alert"""
        if alert_id in self.active_alerts:
            self.active_alerts[alert_id].acknowledged = True
            logger.info(f"✅ Alert {alert_id} acknowledged by {user}")
            return True
        return False
    
    def resolve_alert(self, alert_id: str, user: str = "system") -> bool:
        """Mark an alert as resolved"""
        if alert_id in self.active_alerts:
            self.active_alerts[alert_id].resolved = True
            logger.info(f"✅ Alert {alert_id} resolved by {user}")
            return True
        return False
    
    def get_active_alerts(self) -> List[Alert]:
        """Get all active (unresolved) alerts"""
        return [alert for alert in self.active_alerts.values() if not alert.resolved]
    
    def get_alert_statistics(self) -> Dict[str, Any]:
        """Get alert system statistics"""
        active_alerts = self.get_active_alerts()
        
        return {
            "total_alerts": len(self.active_alerts),
            "active_alerts": len(active_alerts),
            "resolved_alerts": len([a for a in self.active_alerts.values() if a.resolved]),
            "acknowledged_alerts": len([a for a in self.active_alerts.values() if a.acknowledged]),
            "severity_breakdown": {
                severity.value: len([a for a in active_alerts if a.severity == severity])
                for severity in AlertSeverity
            },
            "rules_enabled": len([r for r in self.alert_rules if r.enabled]),
            "rules_total": len(self.alert_rules)
        }

def create_alert_config():
    """Create default alert configuration"""
    config = {
        "email": {
            "smtp_server": "smtp.gmail.com",
            "smtp_port": 587,
            "username": "your-email@gmail.com",
            "password": "your-app-password",
            "from_address": "alerts@hermes-intelligence.com",
            "to_addresses": ["admin@hermes-intelligence.com"]
        },
        "slack": {
            "webhook_url": "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK",
            "channel": "#hermes-alerts",
            "username": "Hermes Alert Bot"
        },
        "telegram": {
            "bot_token": "YOUR_BOT_TOKEN",
            "chat_ids": ["YOUR_CHAT_ID"]
        },
        "webhooks": {
            "primary": "https://your-webhook-endpoint.com/alerts",
            "backup": ""
        },
        "escalation": {
            "levels": [15, 60, 240],
            "emergency_contacts": ["emergency@hermes-intelligence.com"]
        },
        "prof_jiang": {
            "geopolitical_threshold": 0.8,
            "confidence_threshold": 0.6,
            "enable_predictive_alerts": True
        },
        "indonesian_market": {
            "volatility_threshold": 5.0,
            "correlation_break_threshold": 0.3,
            "enable_stock_alerts": True
        }
    }
    
    with open("alert_config.json", "w") as f:
        json.dump(config, f, indent=2)
    
    logger.info("📝 Created alert configuration file")

async def main():
    """Main alert system test"""
    # Create configuration if needed
    if not Path("alert_config.json").exists():
        create_alert_config()
    
    # Initialize alert system
    alert_system = HermesAlertSystem()
    
    # Test with sample metrics
    test_metrics = {
        "service_name": "hermes-analyst",
        "service_status": "healthy",
        "cpu_usage": 75.5,
        "memory_usage": 82.3,
        "response_time_ms": 450,
        "queue_depth": 150,
        "prof_jiang_confidence": 0.85,
        "geopolitical_tension": 0.65,
        "stock_volatility": 3.2,
        "correlation_strength": 0.75,
        "failed_auth_attempts": 2,
        "rate_limit_violations": 15
    }
    
    logger.info("🧪 Testing alert system with sample metrics")
    await alert_system.process_metrics(test_metrics)
    
    # Display statistics
    stats = alert_system.get_alert_statistics()
    logger.info(f"📊 Alert Statistics: {json.dumps(stats, indent=2)}")

if __name__ == "__main__":
    asyncio.run(main())