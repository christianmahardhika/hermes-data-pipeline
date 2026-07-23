#!/usr/bin/env python3
"""
Daily Digest: Financial Intelligence — 10 menit per section.
Sends weekdays at 17:00 WIB. Tracks progress via state file.
Extracts text from EPUB, summarizes with LLM to readable bullet points.
"""

import json, os, re, sys

STATE_FILE = os.path.expanduser("~/.hermes/digest_state_financial_intelligence.json")
EPUB_PATH = "/tmp/financial_intelligence.epub"

# Schedule: (title, subtitle, section_index_start, section_index_end)
# Based on 48 sections extracted, focusing on main chapters (5-42)
SCHEDULE = [
    # Part 1: The Art of Finance
    ("Part 1: Intro", "What Is Financial Intelligence?", 3, 4),
    ("Ch 1", "You Can't Always Trust the Numbers", 4, 5),
    ("Ch 2", "Spotting Assumptions, Estimates, and Biases", 5, 6),
    ("Ch 3", "Why Increase Your Financial Intelligence?", 6, 7),
    ("Ch 4", "The Rules Accountants Follow", 7, 8),
    
    # Part 2: The Income Statement
    ("Part 2: Intro", "Getting What You Want", 8, 9),
    ("Ch 5", "Profit Is an Estimate", 9, 10),
    ("Ch 6", "Cracking the Code of the Income Statement", 10, 11),
    ("Ch 7", "The Issue Is Recognition", 11, 12),
    ("Ch 8: Part 1", "Costs and Expenses — Basics", 12, 13),
    ("Ch 8: Part 2", "Costs and Expenses — Details", 12, 13),
    ("Ch 9", "The Many Forms of Profit", 13, 14),
    
    # Part 3: The Balance Sheet
    ("Part 3: Intro", "Understanding Variance", 14, 15),
    ("Ch 10", "Understanding Balance Sheet Basics", 15, 16),
    ("Ch 11: Part 1", "Assets — Estimates and Assumptions", 16, 17),
    ("Ch 11: Part 2", "Assets — Cash and Receivables", 16, 17),
    ("Ch 12", "On the Other Side — Liabilities", 17, 18),
    ("Ch 13", "Why the Balance Sheet Balances", 18, 19),
    ("Ch 14", "The Income Statement Affects the Balance Sheet", 19, 20),
    
    # Part 4: Cash Flow
    ("Part 4: Intro", "Cash Flow Fundamentals", 20, 21),
    ("Ch 15", "Cash Is a Reality Check", 21, 22),
    ("Ch 16", "Cash Flow Statement Structure", 22, 23),
    ("Ch 17", "The Language of Cash Flow", 23, 24),
    ("Ch 18", "How Cash Connects with Everything Else", 24, 25),
    ("Ch 19", "Why Cash Matters", 25, 26),
    
    # Part 5: Financial Analysis
    ("Ch 20", "The Power of Ratios", 26, 27),
    ("Ch 21: Part 1", "Profitability Ratios — Margins", 27, 28),
    ("Ch 21: Part 2", "Profitability Ratios — Returns", 27, 28),
    ("Ch 22", "Leverage Ratios", 28, 29),
    ("Ch 23", "Liquidity Ratios", 29, 30),
    ("Ch 24", "Efficiency Ratios", 30, 31),
    ("Ch 25", "The Investor's Perspective", 31, 32),
    
    # Part 6: ROI
    ("Part 6: Intro", "Which Ratios Are Most Important?", 32, 33),
    ("Ch 26", "The Building Blocks of ROI", 33, 34),
    ("Ch 27: Part 1", "Figuring ROI — Concepts", 34, 35),
    ("Ch 27: Part 2", "Figuring ROI — Calculations", 35, 36),
    ("Ch 28", "The Magic of Managing the Balance Sheet", 36, 37),
    ("Ch 29", "Your Balance Sheet Levers", 37, 38),
    ("Ch 30", "Homing In on Cash Conversion", 38, 39),
    
    # Part 7: Financial Literacy
    ("Ch 31", "Financial Literacy and Corporate Performance", 40, 41),
    ("Ch 32", "Financial Literacy Strategies", 41, 42),
    ("Ch 33", "Financial Transparency: Our Ultimate Goal", 42, 43),
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


def extract_epub_sections():
    """Extract all sections from EPUB with their content."""
    import ebooklib
    from ebooklib import epub
    from bs4 import BeautifulSoup
    
    book = epub.read_epub(EPUB_PATH)
    sections = []
    
    for item in book.get_items():
        if item.get_type() == ebooklib.ITEM_DOCUMENT:
            soup = BeautifulSoup(item.get_content(), 'html.parser')
            text = soup.get_text(separator='\n', strip=True)
            if len(text.strip()) > 100:
                sections.append(text)
    
    return sections


def clean_text(text):
    """Clean extracted text into readable paragraphs."""
    paragraphs = []
    current = []
    
    for line in text.split('\n'):
        line = line.strip()
        if not line:
            if current:
                paragraphs.append(" ".join(current))
                current = []
        elif re.match(r'^\d+$', line) and len(line) < 4:
            continue  # page numbers
        elif len(line) < 3:
            continue  # too short
        else:
            current.append(line)
    
    if current:
        paragraphs.append(" ".join(current))
    
    # Filter out very short paragraphs
    return [p for p in paragraphs if len(p) > 50]


def summarize_with_llm(text, title, subtitle):
    """Use LLM to summarize section into readable bullet points."""
    # Truncate if too long
    if len(text) > 8000:
        text = text[:8000] + "\n\n[...truncated for length]"
    
    prompt = f"""You are creating a Socratic learning digest for readers of Financial Intelligence (Karen Berman & Joe Knight). The reader is an Indonesian retail investor managing a retirement portfolio.

**Section:** {title} — {subtitle}

**Raw text from book:**
{text}

**Task:**
Transform this section into a Socratic-style reading passage (NOT Q&A format). The writing should:

1. **Open with a provocative question or scenario** — make the reader pause and think. Example: "Bayangkan kamu lihat laporan keuangan perusahaan yang revenue-nya naik 20%, tapi cash flow-nya turun. Apa yang sebenarnya terjadi?"

2. **Guide through discovery** — don't just state facts. Lead the reader step-by-step to understand WHY something works that way. Use "Coba pikirkan..." or "Perhatikan bahwa..." to nudge thinking.

3. **Connect to real life** — relate each concept to TWO contexts:
   a) Stock investing: portfolio decisions, Indonesian stock examples (BBRI, TLKM, PTBA, etc)
   b) Bisnis kopi "Pondo Ngopi": small F&B business perspective — bagaimana konsep ini relevan untuk mengelola kedai kopi (COGS kopi, revenue recognition, cash flow harian, inventory biji kopi, dll)

4. **End with a reflection question** — one question the reader can ponder throughout the day.

**Format:** 
- Write in Indonesian, casual but intelligent tone
- 3-4 paragraphs of flowing prose (NOT bullet points)
- Include 2-3 embedded questions within the text that guide thinking
- Close with "🤔 Renungkan hari ini: [question]"

**Important:** This is a READING format — the reader just reads it like a short essay. No need to answer back."""

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
        summary = re.sub(r'```.*?```', '', summary, flags=re.DOTALL)
        return summary.strip()
        
    except Exception as e:
        print(f"⚠️ LLM summarization failed: {e}", file=sys.stderr)
        return None


def fallback_summary(paragraphs):
    """Fallback if LLM fails — pick key sentences."""
    key_sentences = []
    for p in paragraphs[:10]:
        sentences = re.split(r'[.!?]\s+', p)
        for s in sentences:
            if any(kw in s.lower() for kw in [
                "important", "key", "must", "should", "remember",
                "financial", "ratio", "cash flow", "balance sheet",
                "income statement", "profit", "revenue"
            ]) and len(s) > 50:
                key_sentences.append("• " + s.strip())
                if len(key_sentences) >= 4:
                    return "\n".join(key_sentences)
    
    # If not enough found, take first few sentences
    for p in paragraphs[:5]:
        sentences = re.split(r'[.!?]\s+', p)
        for s in sentences[:2]:
            if len(s) > 50:
                key_sentences.append("• " + s.strip())
                if len(key_sentences) >= 4:
                    break
    
    return "\n".join(key_sentences[:4])


def main():
    state = load_state()
    idx = state["index"]
    
    if idx >= len(SCHEDULE):
        print("🎉 **Daily Digest Complete!**\n\nAll sections of Financial Intelligence have been covered. Thank you! 📚\n\nTo restart: `echo '{\"index\": 0}' > ~/.hermes/digest_state_financial_intelligence.json`")
        return
    
    title, subtitle, sec_start, sec_end = SCHEDULE[idx]
    
    # Extract all sections
    all_sections = extract_epub_sections()
    
    if sec_start >= len(all_sections):
        print(f"⚠️ Section index {sec_start} out of range")
        return
    
    # Get target section(s)
    section_text = "\n\n".join(all_sections[sec_start:sec_end])
    paragraphs = clean_text(section_text)
    
    if not paragraphs:
        print(f"⚠️ Failed to extract section {sec_start}-{sec_end}")
        return
    
    # Combine paragraphs for summarization
    combined_text = "\n\n".join(paragraphs[:15])
    
    # Try LLM summarization
    summary = summarize_with_llm(combined_text, title, subtitle)
    
    # Fallback if LLM fails
    if not summary or len(summary) < 100:
        summary = fallback_summary(paragraphs)
    
    # Build digest
    digest = []
    digest.append(f"📖 **Financial Intelligence — Socratic Digest**")
    digest.append(f"## {title}: {subtitle}")
    digest.append("")
    digest.append(summary)
    digest.append("")
    
    total = len(SCHEDULE)
    pct = (idx + 1) / total * 100
    digest.append(f"📊 Progress: {idx+1}/{total} sections ({pct:.0f}%)")
    digest.append("📅 Tomorrow: next section at 17:00 WIB")
    
    # Advance state
    state["index"] = idx + 1
    save_state(state)
    
    print("\n".join(digest))


if __name__ == "__main__":
    main()
