#!/usr/bin/env python3
"""
Daily Digest: The Intelligent Investor — 10 menit per sub-chapter.
Sends weekdays at 17:00 WIB. Tracks progress via state file.
Extracts text from PDF, summarizes with LLM to readable bullet points.
"""

import json, os, re, sys

STATE_FILE = os.path.expanduser("~/.hermes/digest_state.json")
PDF_PATH = os.path.expanduser("~/Downloads/The_Intelligent_Investor.pdf")

# Schedule: (title, subtitle, page_start, page_end)
SCHEDULE = [
    # Chapter 1: Investment vs Speculation (p32-49 = idx 31-48)
    ("Ch 1: Bagian 1", "Investment vs Speculation — Definisi & Konsep", 31, 35),
    ("Ch 1: Bagian 2", "Hasil — Defensive Investor", 35, 42),
    ("Ch 1: Bagian 3", "Hasil — Aggressive Investor", 42, 48),
    ("Ch 1: Bagian 4", "Komentar Jason Zweig", 48, 60),
    # Chapter 2: Inflation (p61-73)
    ("Ch 2: Bagian 1", "Investor dan Inflasi", 60, 72),
    ("Ch 2: Bagian 2", "Komentar Jason Zweig", 72, 78),
    # Chapter 3: Stock Market History (p79-95)
    ("Ch 3: Bagian 1", "Sejarah Pasar — Pola Satu Abad", 78, 87),
    ("Ch 3: Bagian 2", "Harga Saham 1970 + Komentar", 87, 101),
    # Chapter 4: Portfolio Policy (p102-116)
    ("Ch 4: Bagian 1", "Kebijakan Portofolio Defensive", 101, 115),
    ("Ch 4: Bagian 2", "Komentar Jason Zweig", 115, 125),
    # Chapter 5: Defensive & Stocks (p126-139)
    ("Ch 5: Bagian 1", "Defensive & Common Stocks", 125, 138),
    ("Ch 5: Bagian 2", "Komentar Jason Zweig", 138, 146),
    # Chapter 6: Enterprising - Negative (p147-160)
    ("Ch 6: Bagian 1", "Enterprising — Pendekatan Negatif", 146, 159),
    ("Ch 6: Bagian 2", "Komentar Jason Zweig", 159, 168),
    # Chapter 7: Enterprising - Positive (p169-194)
    ("Ch 7: Bagian 1", "Enterprising — 4 Pendekatan", 168, 182),
    ("Ch 7: Bagian 2", "Enterprising — Growth & Bargain", 182, 193),
    ("Ch 7: Bagian 3", "Komentar Jason Zweig", 193, 201),
    # Chapter 8: Market Fluctuations (p202-228)
    ("Ch 8: Bagian 1", "Mr. Market — Fluktuasi Pasar", 201, 213),
    ("Ch 8: Bagian 2", "DCA & Pandangan Bisnis", 213, 227),
    ("Ch 8: Bagian 3", "Komentar Jason Zweig", 227, 239),
    # Chapter 9: Investment Funds (p240-257)
    ("Ch 9: Bagian 1", "Investasi di Investment Funds", 239, 256),
    ("Ch 9: Bagian 2", "Komentar Jason Zweig", 256, 270),
    # Chapter 10: Advisers (p271-287)
    ("Ch 10: Bagian 1", "Investor dan Penasihat", 270, 286),
    ("Ch 10: Bagian 2", "Komentar Jason Zweig", 286, 293),
    # Chapter 11: Security Analysis (p294-317)
    ("Ch 11: Bagian 1", "Security Analysis — Pendekatan", 293, 316),
    ("Ch 11: Bagian 2", "Komentar Jason Zweig", 316, 323),
    # Chapter 12: Per-Share Earnings (p324-337)
    ("Ch 12: Bagian 1", "Laba per Saham", 323, 336),
    ("Ch 12: Bagian 2", "Komentar Jason Zweig", 336, 343),
    # Chapter 13: Four Companies (p344-355)
    ("Ch 13: Bagian 1", "4 Perusahaan — Perbandingan", 343, 354),
    ("Ch 13: Bagian 2", "Komentar Jason Zweig", 354, 360),
    # Chapter 14: Defensive Selection (p361-382)
    ("Ch 14: Bagian 1", "7 Kriteria — Defensive Selection", 360, 372),
    ("Ch 14: Bagian 2", "Aplikasi 7 Kriteria", 372, 381),
    ("Ch 14: Bagian 3", "Komentar Jason Zweig", 381, 389),
    # Chapter 15: Enterprising Selection (p390-411)
    ("Ch 15: Bagian 1", "Bargain Issues & Special Situations", 389, 401),
    ("Ch 15: Bagian 2", "Aplikasi & Contoh", 401, 410),
    ("Ch 15: Bagian 3", "Komentar Jason Zweig", 410, 416),
    # Chapter 16: Convertibles (p417-433)
    ("Ch 16: Bagian 1", "Convertible Issues & Warrants", 416, 432),
    ("Ch 16: Bagian 2", "Komentar Jason Zweig", 432, 435),
    # Chapter 17: Case Histories (p436-453)
    ("Ch 17: Bagian 1", "4 Case Histories — Pelajaran", 435, 452),
    ("Ch 17: Bagian 2", "Komentar Jason Zweig", 452, 459),
    # Chapter 18: Eight Pairs (p460-488)
    ("Ch 18: Bagian 1", "8 Pasang — Analisis 1", 459, 473),
    ("Ch 18: Bagian 2", "8 Pasang — Analisis 2", 473, 487),
    ("Ch 18: Bagian 3", "Komentar Jason Zweig", 487, 500),
    # Chapter 19: Shareholders & Management (p501-512)
    ("Ch 19: Bagian 1", "Pemegang Saham & Dividen", 500, 511),
    ("Ch 19: Bagian 2", "Komentar Jason Zweig", 511, 525),
    # Chapter 20: Margin of Safety (p526-540)
    ("Ch 20: Bagian 1", "Margin of Safety — Konsep Inti", 525, 539),
    ("Ch 20: Bagian 2", "Komentar + Postscript", 539, 550),
]


def load_state():
    if os.path.exists(STATE_FILE):
        with open(STATE_FILE) as f:
            state = json.load(f)
            if "index" not in state:
                state["index"] = 0
            return state
    return {"index": 0}


def save_state(state):
    os.makedirs(os.path.dirname(STATE_FILE), exist_ok=True)
    with open(STATE_FILE, "w") as f:
        json.dump(state, f, indent=2)


def extract_pdf_text(page_start, page_end):
    """Get clean paragraphs from PDF page range."""
    import fitz  # PyMuPDF
    doc = fitz.open(PDF_PATH)
    paragraphs = []
    current = []
    
    for i in range(page_start, min(page_end, len(doc))):
        text = doc[i].get_text()
        for line in text.split('\n'):
            line = line.strip()
            if not line:
                if current:
                    paragraphs.append(" ".join(current))
                    current = []
            elif re.match(r'^\d+$', line):
                continue  # page numbers
            elif line == "The Intelligent Investor":
                continue  # running header
            elif re.match(r'^(Investment versus Speculation|Results to Be Expected|COMMENTARY ON)', line) and len(line) < 50:
                continue  # running chapter headers
            else:
                current.append(line)
    
    if current:
        paragraphs.append(" ".join(current))
    
    doc.close()
    
    # Clean hyphenated words
    cleaned = []
    for p in paragraphs:
        p = re.sub(r'(\w)-\s+(\w)', r'\1\2', p)
        p = re.sub(r' p\.\.\.$', '', p)
        p = re.sub(r'^CHAPTER \d+ ', '', p)
        p = p.strip()
        if len(p) > 80:
            cleaned.append(p)
    
    return cleaned


def summarize_with_llm(paragraphs, title, subtitle):
    """Use LLM to summarize paragraphs into readable bullet points."""
    # Join first 15 paragraphs (enough context, not too long)
    text = "\n\n".join(paragraphs[:15])
    
    # Truncate if too long (LLM context limit)
    if len(text) > 8000:
        text = text[:8000] + "\n\n[...truncated for length]"
    
    prompt = f"""You are creating a daily digest for readers of The Intelligent Investor (Benjamin Graham).

**Sub-chapter:** {title} — {subtitle}

**Raw text from PDF:**
{text}

**Task:**
Summarize this sub-chapter into 4-5 bullet points that are:
1. **Clear & readable** — no truncated sentences or "..."
2. **Key insights** — focus on Graham's concepts/principles/rules
3. **Actionable** — what investors should do
4. **Natural English** — conversational but professional

Output format:
• [Insight 1]
• [Insight 2]
• [Insight 3]
• [Insight 4]

No intro/outro, just bullet points."""

    # Call LLM via OpenAI-compatible API (LiteLLM proxy)
    try:
        from openai import OpenAI
        
        client = OpenAI(
            base_url="http://localhost:9000/v1",
            api_key="Pagupon123"
        )
        
        response = client.chat.completions.create(
            model="claude-sonnet-4.5",
            messages=[
                {"role": "user", "content": prompt}
            ],
            temperature=0.7,
            max_tokens=1000
        )
        
        summary = response.choices[0].message.content.strip()
        
        # Clean up any markdown formatting
        summary = re.sub(r'```.*?```', '', summary, flags=re.DOTALL)
        summary = summary.strip()
        
        return summary
        
    except Exception as e:
        print(f"⚠️ LLM summarization failed: {e}", file=sys.stderr)
        return None


def fallback_summary(paragraphs):
    """Fallback if LLM fails — pick key sentences."""
    key_sentences = []
    for p in paragraphs[:10]:
        # Look for Graham's key statements
        sentences = re.split(r'[.!?]\s+', p)
        for s in sentences:
            if any(kw in s.lower() for kw in [
                "must", "should", "recommend", "advise", "principle",
                "rule", "margin of safety", "defensive investor",
                "enterprising investor", "our view", "therefore"
            ]) and len(s) > 50:
                key_sentences.append("• " + s.strip())
                if len(key_sentences) >= 4:
                    return "\n".join(key_sentences)
    
    # If not enough found, just take first few sentences
    for p in paragraphs[:5]:
        sentences = re.split(r'[.!?]\s+', p)
        for s in sentences[:2]:
            if len(s) > 50:
                key_sentences.append("• " + s.strip())
                if len(key_sentences) >= 4:
                    break
    
    return "\n".join(key_sentences[:4])


def write_takeaway(title):
    """Return a short takeaway tailored to the sub-chapter."""
    takeaways = {
        "Investment vs Speculation": (
            "Graham draws a clear line: investment = thorough analysis + safety of principal + adequate return. "
            "Speculation = hoping for profit without analysis. Never mix the two."
        ),
        "Defensive Investor": (
            "Defensive investors use the 50-50 rule: half bonds, half blue-chip stocks. "
            "Rebalance when market moves >5%. Focus on safety and simplicity, not maximum returns."
        ),
        "Aggressive Investor": (
            "Aggressive investors can take more risk, but must have time and competence. "
            "Graham is skeptical — most 'aggressive' investors are actually just speculators."
        ),
    }
    
    for key, msg in takeaways.items():
        if key.lower() in title.lower():
            return msg
    
    return "Graham's core principle: be an investor, not a speculator. Analyze before buying, discipline when selling."


def main():
    state = load_state()
    idx = state["index"]
    
    if idx >= len(SCHEDULE):
        print("🎉 **Daily Digest Complete!**\n\nAll 47 sub-chapters of The Intelligent Investor have been covered. Thank you! 📚\n\nTo restart: `echo '{\\\"index\\\": 0}' > ~/.hermes/digest_state.json`")
        return
    
    title, subtitle, p_start, p_end = SCHEDULE[idx]
    paragraphs = extract_pdf_text(p_start, p_end)
    
    if not paragraphs:
        print(f"⚠️ Failed to extract pages {p_start}-{p_end}")
        return
    
    # Try LLM summarization first
    summary = summarize_with_llm(paragraphs, title, subtitle)
    
    # Fallback if LLM fails
    if not summary or len(summary) < 100:
        summary = fallback_summary(paragraphs)
    
    takeaway = write_takeaway(title)
    
    # Build digest
    digest = []
    digest.append(f"📖 **The Intelligent Investor**")
    digest.append(f"## {title}")
    digest.append(f"*{subtitle}*")
    digest.append("")
    digest.append(summary)
    digest.append("")
    digest.append("---")
    digest.append(f"💡 **Graham's Key Message:**")
    digest.append(takeaway)
    digest.append("")
    
    total = len(SCHEDULE)
    pct = (idx + 1) / total * 100
    digest.append(f"📊 Progress: {idx+1}/{total} sub-chapters ({pct:.0f}%)")
    digest.append("📅 Tomorrow: next sub-chapter at 17:00 WIB")
    
    # Advance state
    state["index"] = idx + 1
    save_state(state)
    
    print("\n".join(digest))


if __name__ == "__main__":
    main()
