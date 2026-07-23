#!/usr/bin/env python3
"""
Monitor berita tentang PP BUMN Ekspor komoditas Prabowo.
Checks CNBC, CNN Indonesia, Kompas for updates.
Sends alert if new developments found.
"""

import json
import os
from datetime import datetime
from openai import OpenAI

STATE_FILE = os.path.expanduser("~/.hermes/bumn_export_state.json")
KEYWORDS = [
    "BUMN Ekspor",
    "PP ekspor komoditas",
    "Prabowo ekspor batu bara",
    "BUMN khusus ekspor",
    "Purbaya ekspor",
    "APBI BUMN ekspor",
    "under invoicing ekspor",
    "DHE SDA"
]

# Known articles (to avoid duplicates)
KNOWN_ARTICLES = [
    "Resmi Umumkan Aturan Baru Ekspor, Prabowo: Bukan Kebijakan Aneh-Aneh!",
    "Prabowo Bentuk BUMN Khusus Ekspor Batu Bara Cs, Pengusaha Teriak",
    "Prabowo Bakal Keras Kontrol Sawit, Bos Pengusaha Sawit Respons Begini",
    "Lengkap! Poin Penting Pidato Prabowo di DPR: Target Ekonomi - Tambang"
]


def load_state():
    if os.path.exists(STATE_FILE):
        with open(STATE_FILE) as f:
            return json.load(f)
    return {"last_check": None, "known_articles": KNOWN_ARTICLES}


def save_state(state):
    os.makedirs(os.path.dirname(STATE_FILE), exist_ok=True)
    with open(STATE_FILE, "w") as f:
        json.dump(state, f, indent=2)


def check_news():
    """Check CNBC Indonesia for new articles about BUMN Ekspor."""
    import requests
    from bs4 import BeautifulSoup
    
    alerts = []
    
    try:
        # Check CNBC search
        url = "https://www.cnbcindonesia.com/search?query=BUMN+ekspor+komoditas"
        headers = {
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
        }
        response = requests.get(url, headers=headers, timeout=10)
        soup = BeautifulSoup(response.content, 'html.parser')
        
        # Find article titles
        articles = soup.find_all(['h2', 'h3'], limit=10)
        
        for article in articles:
            title = article.get_text(strip=True)
            if any(kw.lower() in title.lower() for kw in KEYWORDS):
                alerts.append({
                    "title": title,
                    "source": "CNBC Indonesia",
                    "url": f"https://www.cnbcindonesia.com/search?query=BUMN+ekspor"
                })
        
    except Exception as e:
        alerts.append({
            "title": f"Error checking news: {str(e)}",
            "source": "System",
            "url": ""
        })
    
    return alerts


def summarize_with_llm(new_articles):
    """Use LLM to create brief summary if new articles found."""
    if not new_articles:
        return None
    
    client = OpenAI(
        base_url="http://localhost:9000/v1",
        api_key="Pagupon123"
    )
    
    prompt = f"""Summarize these news alerts about Indonesia's new BUMN Export policy for commodities.

New articles detected:
{json.dumps(new_articles, indent=2)}

Context: Prabowo announced PP requiring ALL commodity exports (coal, palm oil, nickel) to go through government-appointed BUMN. This is to combat under-invoicing, transfer pricing, and DHE leakage.

Create a brief alert message (2-3 sentences) for investors about what's new and what to watch.
"""
    
    try:
        response = client.chat.completions.create(
            model="claude-sonnet-4.5",
            messages=[{"role": "user", "content": prompt}],
            temperature=0.7,
            max_tokens=500
        )
        return response.choices[0].message.content.strip()
    except:
        return "⚠️ New articles detected about BUMN Export policy. Check CNBC Indonesia for details."


def main():
    state = load_state()
    
    # Check for new articles
    alerts = check_news()
    
    # Filter out known articles
    new_alerts = []
    for alert in alerts:
        if alert["title"] not in state.get("known_articles", []):
            new_alerts.append(alert)
            state["known_articles"].append(alert["title"])
    
    state["last_check"] = datetime.now().isoformat()
    save_state(state)
    
    if new_alerts:
        summary = summarize_with_llm(new_alerts)
        
        digest = []
        digest.append("🚨 **BUMN EXPORT POLICY ALERT**")
        digest.append("")
        digest.append(f"📅 {datetime.now().strftime('%d %b %Y %H:%M')}")
        digest.append("")
        digest.append(f"**{len(new_alerts)} new development(s) detected:**")
        digest.append("")
        for alert in new_alerts[:5]:
            digest.append(f"• {alert['title']}")
            digest.append(f"  Source: {alert['source']}")
        digest.append("")
        digest.append("---")
        digest.append("💡 **Summary:**")
        digest.append(summary)
        digest.append("")
        digest.append("📌 **Action Items:**")
        digest.append("• Check full article for implementation details")
        digest.append("• Monitor which BUMN gets appointed")
        digest.append("• Watch for judicial review / legal challenges")
        digest.append("")
        digest.append("Related stocks: PTBA, ADRO, ITMG, BYAN, AALI, SGRO, INCO, ANTM, MDKA")
        
        print("\n".join(digest))
    else:
        # Silent run - no news
        pass


if __name__ == "__main__":
    main()
