#!/usr/bin/env python3
"""
Daily Tech Curation - Replacement for missing Rust binary
Collects and curates tech news from multiple sources
"""

import requests
import json
from datetime import datetime
import os
import sys

def get_github_trending():
    """Collect trending GitHub repositories and releases"""
    github_items = []
    
    try:
        print("📡 Fetching GitHub trending repositories...")
        
        # GitHub trending repos (today)
        from datetime import datetime, timedelta
        yesterday = (datetime.now() - timedelta(days=1)).strftime('%Y-%m-%d')
        
        # Search for trending repos created recently or with recent activity
        trending_url = f"https://api.github.com/search/repositories?q=created:>{yesterday}&sort=stars&order=desc&per_page=5"
        
        headers = {'User-Agent': 'TechIntelligence/1.0'}
        response = requests.get(trending_url, headers=headers, timeout=10)
        
        if response.status_code == 200:
            data = response.json()
            for repo in data.get('items', [])[:3]:  # Top 3 trending
                github_items.append({
                    'source': 'GitHub-Trending',
                    'title': f"{repo['full_name']} - {repo['description'][:100] if repo['description'] else 'No description'}",
                    'url': repo['html_url'],
                    'score': repo['stargazers_count'],
                    'language': repo.get('language', 'Unknown'),
                    'type': 'repo'
                })
        
        # GitHub popular AI/ML repos (weekly)
        ai_url = "https://api.github.com/search/repositories?q=machine+learning+OR+artificial+intelligence+OR+deep+learning&sort=stars&order=desc&per_page=3"
        ai_response = requests.get(ai_url, headers=headers, timeout=10)
        
        if ai_response.status_code == 200:
            ai_data = ai_response.json()
            for repo in ai_data.get('items', [])[:2]:  # Top 2 AI repos
                github_items.append({
                    'source': 'GitHub-AI',
                    'title': f"{repo['full_name']} - {repo['description'][:100] if repo['description'] else 'AI/ML Project'}",
                    'url': repo['html_url'],
                    'score': repo['stargazers_count'],
                    'language': repo.get('language', 'Unknown'),
                    'type': 'ai-repo'
                })
                
    except Exception as e:
        print(f"❌ GitHub error: {e}")
    
    return github_items

def get_tech_news():
    """Collect tech news from various sources"""
    news_items = []
    
    # HackerNews API
    try:
        print("📡 Fetching HackerNews top stories...")
        hn_response = requests.get("https://hacker-news.firebaseio.com/v0/topstories.json", timeout=10)
        if hn_response.status_code == 200:
            story_ids = hn_response.json()[:10]  # Top 10 stories
            
            for story_id in story_ids[:5]:  # Limit to 5 for speed
                try:
                    story_response = requests.get(f"https://hacker-news.firebaseio.com/v0/item/{story_id}.json", timeout=5)
                    if story_response.status_code == 200:
                        story = story_response.json()
                        if story.get('title') and story.get('score', 0) > 50:  # Only popular stories
                            news_items.append({
                                'source': 'HackerNews',
                                'title': story['title'],
                                'url': story.get('url', f"https://news.ycombinator.com/item?id={story_id}"),
                                'score': story.get('score', 0),
                                'type': 'tech'
                            })
                except:
                    continue
                    
    except Exception as e:
        print(f"❌ HackerNews error: {e}")
    
    # GitHub trending integration
    github_items = get_github_trending()
    news_items.extend(github_items)
    
    # Reddit Tech (via JSON API)
    try:
        print("📡 Fetching Reddit tech news...")
        reddit_response = requests.get("https://www.reddit.com/r/technology/hot.json?limit=10", 
                                     headers={'User-Agent': 'TechCurator/1.0'}, timeout=10)
        if reddit_response.status_code == 200:
            reddit_data = reddit_response.json()
            for post in reddit_data['data']['children'][:5]:
                post_data = post['data']
                if post_data.get('score', 0) > 100:  # Popular posts only
                    news_items.append({
                        'source': 'Reddit',
                        'title': post_data['title'],
                        'url': post_data['url'],
                        'score': post_data['score'],
                        'type': 'tech'
                    })
    except Exception as e:
        print(f"❌ Reddit error: {e}")
    
    return news_items

def generate_summary(news_items):
    """Generate formatted summary"""
    date_str = datetime.now().strftime("%Y-%m-%d")
    
    # Group by source
    hn_items = [item for item in news_items if item['source'] == 'HackerNews']
    reddit_items = [item for item in news_items if item['source'] == 'Reddit']
    github_trending = [item for item in news_items if item['source'] == 'GitHub-Trending']
    github_ai = [item for item in news_items if item['source'] == 'GitHub-AI']
    
    summary = f"""🔥 **DAILY TECH INTELLIGENCE** (Enhanced)
📅 {date_str}

🎯 **SOURCE INTELLIGENCE**
📊 **HackerNews**: {len(hn_items)} stories
📊 **GitHub Trending**: {len(github_trending)} repos
📊 **GitHub AI/ML**: {len(github_ai)} projects  
📊 **Reddit**: {len(reddit_items)} posts

🏆 **TOP TECH STORIES**

"""

    # Add top stories from each source
    if hn_items:
        summary += "**🟠 HackerNews Hot:**\n"
        for item in hn_items[:3]:
            summary += f"• {item['title']} ({item['score']} points)\n"
        summary += "\n"
    
    if github_trending:
        summary += "**⭐ GitHub Trending:**\n"
        for item in github_trending:
            summary += f"• {item['title'][:80]}... ({item['score']} ⭐)\n"
        summary += "\n"
    
    if github_ai:
        summary += "**🤖 GitHub AI/ML:**\n"
        for item in github_ai:
            lang_info = f" [{item['language']}]" if item.get('language') else ""
            summary += f"• {item['title'][:80]}...{lang_info} ({item['score']} ⭐)\n"
        summary += "\n"
    
    if reddit_items:
        summary += "**🔴 Reddit Tech:**\n" 
        for item in reddit_items[:3]:
            summary += f"• {item['title']} ({item['score']} upvotes)\n"
        summary += "\n"

    total_sources = len([s for s in [hn_items, github_trending, github_ai, reddit_items] if s])
    summary += f"""📊 **SUMMARY INSIGHTS**
• Processed {len(news_items)} tech items across {total_sources} source types
• HackerNews: {len(hn_items)}, GitHub: {len(github_trending + github_ai)}, Reddit: {len(reddit_items)}
• AI/ML Focus: {len(github_ai)} trending AI repositories tracked

✅ Daily Tech Intelligence Complete!
🚀 **Enhanced with GitHub** - Multi-source tech trend analysis
"""
    
    return summary

def main():
    print("🚀 Starting Daily Tech Curation...")
    
    try:
        # Collect news
        news_items = get_tech_news()
        print(f"📊 Collected {len(news_items)} tech stories")
        
        # Generate summary
        summary = generate_summary(news_items)
        
        # Save summary to file
        date_str = datetime.now().strftime("%Y%m%d")
        summary_file = f"tech_curation_summary_{date_str}.txt"
        
        with open(summary_file, 'w', encoding='utf-8') as f:
            f.write(summary)
        
        print(f"💾 Saved summary to {summary_file}")
        
        # Also save raw data as JSON
        data_file = f"tech_curation_data_{date_str}.json"
        with open(data_file, 'w', encoding='utf-8') as f:
            json.dump(news_items, f, indent=2, ensure_ascii=False)
        
        print(f"💾 Saved raw data to {data_file}")
        
        # Print summary to stdout (for cron job delivery)
        print("\n" + "="*50)
        print(summary)
        
    except Exception as e:
        print(f"❌ Error in tech curation: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()