#!/usr/bin/env python3
"""
Daily Curated Tech Collection from HackerNews and GitHub
Focused tech curation for Christian's intelligence system

Author: Christian Mahardhika  
Sources: HackerNews API + GitHub Trending API
Frequency: Daily curation with quality filtering
"""

import asyncio
import json
import logging
from datetime import datetime, timedelta
from typing import Dict, List, Tuple, Optional
import requests
import time
from dataclasses import dataclass, asdict

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

@dataclass
class TechCurationReport:
    """Daily Tech Curation Report Structure"""
    date: str
    hackernews_stories: List[Dict]
    github_trending: List[Dict]
    curated_highlights: List[Dict]
    tech_categories: Dict[str, List[Dict]]
    summary_insights: Dict[str, str]
    
class DailyCuratedTechCollector:
    """Daily Curated Tech Collection from HackerNews and GitHub"""
    
    def __init__(self):
        # API endpoints
        self.hackernews_api = "https://hacker-news.firebaseio.com/v0"
        self.github_trending_api = "https://api.github.com/search/repositories"
        
        # Tech categories for curation
        self.tech_categories = {
            'ai_ml': ['ai', 'machine learning', 'ml', 'artificial intelligence', 'llm', 'gpt', 'neural', 'deep learning'],
            'blockchain': ['blockchain', 'crypto', 'bitcoin', 'ethereum', 'defi', 'web3', 'nft'],
            'cloud_infra': ['cloud', 'aws', 'azure', 'gcp', 'kubernetes', 'docker', 'devops', 'infrastructure'],
            'programming': ['python', 'rust', 'golang', 'javascript', 'typescript', 'programming', 'coding'],
            'security': ['security', 'cybersecurity', 'privacy', 'encryption', 'vulnerability', 'hacking'],
            'startups': ['startup', 'funding', 'vc', 'investment', 'unicorn', 'ipo', 'business'],
            'mobile': ['mobile', 'ios', 'android', 'flutter', 'react native', 'app development'],
            'web_dev': ['web development', 'frontend', 'backend', 'fullstack', 'react', 'vue', 'angular'],
            'fintech': ['fintech', 'financial technology', 'banking technology', 'payment', 'digital wallet'],
            'indonesia_tech': ['indonesia', 'indonesian', 'gojek', 'tokopedia', 'bukalapak', 'jakarta', 'southeast asia']
        }
        
        # Quality scoring weights
        self.quality_weights = {
            'hackernews_score': 0.3,
            'github_stars': 0.25, 
            'recency': 0.2,
            'category_relevance': 0.15,
            'indonesia_relevance': 0.1
        }
    
    def fetch_hackernews_top_stories(self, limit: int = 30) -> List[Dict]:
        """Fetch top stories from HackerNews"""
        logger.info("🔍 Fetching HackerNews top stories...")
        
        try:
            # Get top story IDs
            response = requests.get(f"{self.hackernews_api}/topstories.json", timeout=10)
            story_ids = response.json()[:limit]
            
            stories = []
            for story_id in story_ids:
                try:
                    story_response = requests.get(f"{self.hackernews_api}/item/{story_id}.json", timeout=5)
                    story_data = story_response.json()
                    
                    if story_data and story_data.get('type') == 'story':
                        # Calculate quality score
                        score = story_data.get('score', 0)
                        tech_relevance = self._calculate_tech_relevance(story_data.get('title', ''))
                        
                        stories.append({
                            'id': story_data.get('id'),
                            'title': story_data.get('title', ''),
                            'url': story_data.get('url', ''),
                            'score': score,
                            'by': story_data.get('by', ''),
                            'time': story_data.get('time', 0),
                            'descendants': story_data.get('descendants', 0),
                            'tech_relevance': tech_relevance,
                            'quality_score': self._calculate_hackernews_quality_score(story_data, tech_relevance),
                            'source': 'HackerNews'
                        })
                    
                    time.sleep(0.1)  # Rate limiting
                    
                except Exception as e:
                    logger.warning(f"Error fetching HN story {story_id}: {e}")
                    continue
            
            # Sort by quality score
            stories.sort(key=lambda x: x['quality_score'], reverse=True)
            logger.info(f"✅ Fetched {len(stories)} HackerNews stories")
            return stories
            
        except Exception as e:
            logger.error(f"Error fetching HackerNews stories: {e}")
            return []
    
    def fetch_github_trending(self, language: str = None, days: int = 1) -> List[Dict]:
        """Fetch GitHub trending repositories"""
        logger.info("🔍 Fetching GitHub trending repositories...")
        
        try:
            # Calculate date for trending period
            since_date = (datetime.now() - timedelta(days=days)).strftime('%Y-%m-%d')
            
            # Build query
            query = f"created:>{since_date}"
            if language:
                query += f" language:{language}"
            
            params = {
                'q': query,
                'sort': 'stars',
                'order': 'desc',
                'per_page': 30
            }
            
            response = requests.get(self.github_trending_api, params=params, timeout=10)
            data = response.json()
            
            repositories = []
            for repo in data.get('items', []):
                tech_relevance = self._calculate_tech_relevance(f"{repo.get('name', '')} {repo.get('description', '')}")
                
                repositories.append({
                    'name': repo.get('full_name', ''),
                    'description': repo.get('description', ''),
                    'url': repo.get('html_url', ''),
                    'stars': repo.get('stargazers_count', 0),
                    'language': repo.get('language', ''),
                    'created_at': repo.get('created_at', ''),
                    'updated_at': repo.get('updated_at', ''),
                    'topics': repo.get('topics', []),
                    'tech_relevance': tech_relevance,
                    'quality_score': self._calculate_github_quality_score(repo, tech_relevance),
                    'source': 'GitHub'
                })
            
            # Sort by quality score
            repositories.sort(key=lambda x: x['quality_score'], reverse=True)
            logger.info(f"✅ Fetched {len(repositories)} GitHub repositories")
            return repositories
            
        except Exception as e:
            logger.error(f"Error fetching GitHub trending: {e}")
            return []
    
    def _calculate_tech_relevance(self, text: str) -> Dict[str, float]:
        """Calculate relevance to tech categories"""
        text_lower = text.lower()
        relevance = {}
        
        for category, keywords in self.tech_categories.items():
            score = 0
            for keyword in keywords:
                if keyword.lower() in text_lower:
                    score += 1
            relevance[category] = score / len(keywords) if keywords else 0
        
        return relevance
    
    def _calculate_hackernews_quality_score(self, story: Dict, tech_relevance: Dict) -> float:
        """Calculate quality score for HackerNews story"""
        score = story.get('score', 0)
        comments = story.get('descendants', 0)
        
        # Normalize scores
        score_normalized = min(score / 500, 1.0)  # Max score consideration
        comments_normalized = min(comments / 200, 1.0)  # Max comments consideration
        
        # Tech relevance (average across categories)
        tech_score = sum(tech_relevance.values()) / len(tech_relevance) if tech_relevance else 0
        
        # Recency score (newer is better)
        story_time = story.get('time', 0)
        current_time = time.time()
        hours_old = (current_time - story_time) / 3600
        recency_score = max(1 - (hours_old / 48), 0)  # Decay over 48 hours
        
        # Indonesia relevance bonus
        title_lower = story.get('title', '').lower()
        indonesia_bonus = 0.2 if any(keyword in title_lower for keyword in self.tech_categories['indonesia_tech']) else 0
        
        # Calculate weighted score
        quality_score = (
            score_normalized * 0.3 +
            comments_normalized * 0.15 +
            tech_score * 0.25 +
            recency_score * 0.2 +
            indonesia_bonus * 0.1
        )
        
        return quality_score
    
    def _calculate_github_quality_score(self, repo: Dict, tech_relevance: Dict) -> float:
        """Calculate quality score for GitHub repository"""
        stars = repo.get('stargazers_count', 0)
        
        # Normalize stars
        stars_normalized = min(stars / 1000, 1.0)  # Max stars consideration
        
        # Tech relevance
        tech_score = sum(tech_relevance.values()) / len(tech_relevance) if tech_relevance else 0
        
        # Recency score
        created_at = repo.get('created_at', '')
        try:
            created_time = datetime.fromisoformat(created_at.replace('Z', '+00:00'))
            hours_old = (datetime.now() - created_time.replace(tzinfo=None)).total_seconds() / 3600
            recency_score = max(1 - (hours_old / 168), 0)  # Decay over 1 week
        except:
            recency_score = 0
        
        # Language bonus (popular languages)
        language = repo.get('language') or ''
        language_lower = language.lower() if language else ''
        language_bonus = 0.1 if language_lower in ['python', 'javascript', 'typescript', 'rust', 'go'] else 0
        
        # Indonesia relevance bonus
        description = repo.get('description') or ''
        name = repo.get('name') or ''
        description_lower = description.lower() if description else ''
        name_lower = name.lower() if name else ''
        indonesia_bonus = 0.15 if any(keyword in f"{name_lower} {description_lower}" for keyword in self.tech_categories['indonesia_tech']) else 0
        
        # Calculate weighted score
        quality_score = (
            stars_normalized * 0.35 +
            tech_score * 0.25 +
            recency_score * 0.2 +
            language_bonus * 0.1 +
            indonesia_bonus * 0.1
        )
        
        return quality_score
    
    def curate_highlights(self, hackernews_stories: List[Dict], github_repos: List[Dict]) -> List[Dict]:
        """Curate top highlights from both sources"""
        logger.info("🎯 Curating top tech highlights...")
        
        # Combine and sort all items by quality score
        all_items = hackernews_stories + github_repos
        all_items.sort(key=lambda x: x['quality_score'], reverse=True)
        
        # Select top highlights with diversity
        highlights = []
        categories_covered = set()
        
        for item in all_items:
            if len(highlights) >= 10:  # Limit to top 10 highlights
                break
            
            # Find primary category
            tech_relevance = item.get('tech_relevance', {})
            primary_category = max(tech_relevance.items(), key=lambda x: x[1])[0] if tech_relevance else 'general'
            
            # Add if category not over-represented or if very high quality
            if primary_category not in categories_covered or item['quality_score'] > 0.7:
                highlights.append({
                    **item,
                    'primary_category': primary_category,
                    'highlight_reason': self._generate_highlight_reason(item)
                })
                categories_covered.add(primary_category)
        
        logger.info(f"✅ Curated {len(highlights)} tech highlights")
        return highlights
    
    def _generate_highlight_reason(self, item: Dict) -> str:
        """Generate reason why item is highlighted"""
        source = item.get('source', '')
        quality_score = item.get('quality_score', 0)
        
        if source == 'HackerNews':
            score = item.get('score', 0)
            if score > 300:
                return f"High engagement ({score} points)"
            elif item.get('tech_relevance', {}).get('ai_ml', 0) > 0.5:
                return "AI/ML trending topic"
            else:
                return "Community interest"
        else:  # GitHub
            stars = item.get('stars', 0)
            if stars > 500:
                return f"Popular repository ({stars} stars)"
            elif item.get('language') in ['Rust', 'Go', 'Python']:
                return f"Trending {item.get('language')} project"
            else:
                return "New trending repository"
    
    def categorize_content(self, hackernews_stories: List[Dict], github_repos: List[Dict]) -> Dict[str, List[Dict]]:
        """Categorize content by tech categories"""
        categorized = {category: [] for category in self.tech_categories.keys()}
        
        all_items = hackernews_stories + github_repos
        
        for item in all_items:
            tech_relevance = item.get('tech_relevance', {})
            
            # Assign to primary category
            if tech_relevance:
                primary_category = max(tech_relevance.items(), key=lambda x: x[1])[0]
                if tech_relevance[primary_category] > 0.1:  # Minimum relevance threshold
                    categorized[primary_category].append(item)
        
        # Sort items within each category by quality score
        for category in categorized:
            categorized[category].sort(key=lambda x: x['quality_score'], reverse=True)
            categorized[category] = categorized[category][:5]  # Top 5 per category
        
        return categorized
    
    def generate_summary_insights(self, hackernews_stories: List[Dict], github_repos: List[Dict], highlights: List[Dict]) -> Dict[str, str]:
        """Generate summary insights from collected data"""
        insights = {}
        
        # Overall trends
        total_items = len(hackernews_stories) + len(github_repos)
        avg_quality = sum(item['quality_score'] for item in hackernews_stories + github_repos) / total_items if total_items > 0 else 0
        
        insights['daily_summary'] = f"Curated {total_items} tech items with average quality {avg_quality:.2f}"
        
        # Top categories
        all_items = hackernews_stories + github_repos
        category_counts = {}
        for item in all_items:
            tech_relevance = item.get('tech_relevance', {})
            if tech_relevance:
                primary_category = max(tech_relevance.items(), key=lambda x: x[1])[0]
                category_counts[primary_category] = category_counts.get(primary_category, 0) + 1
        
        if category_counts:
            top_category = max(category_counts.items(), key=lambda x: x[1])
            insights['trending_category'] = f"{top_category[0].replace('_', ' ').title()} dominates with {top_category[1]} items"
        
        # Source breakdown
        hn_count = len(hackernews_stories)
        gh_count = len(github_repos)
        insights['source_breakdown'] = f"HackerNews: {hn_count} stories, GitHub: {gh_count} repositories"
        
        # Indonesia relevance
        indonesia_items = [item for item in all_items if item.get('tech_relevance', {}).get('indonesia_tech', 0) > 0]
        if indonesia_items:
            insights['indonesia_tech'] = f"{len(indonesia_items)} items with Indonesian tech relevance"
        
        return insights
    
    def generate_daily_report(self) -> TechCurationReport:
        """Generate comprehensive daily tech curation report"""
        logger.info("🎯 Generating Daily Tech Curation Report...")
        
        # Fetch data from both sources
        hackernews_stories = self.fetch_hackernews_top_stories()
        github_repos = self.fetch_github_trending()
        
        # Curate highlights
        highlights = self.curate_highlights(hackernews_stories, github_repos)
        
        # Categorize content
        categorized_content = self.categorize_content(hackernews_stories, github_repos)
        
        # Generate insights
        insights = self.generate_summary_insights(hackernews_stories, github_repos, highlights)
        
        # Create comprehensive report
        report = TechCurationReport(
            date=datetime.now().strftime('%Y-%m-%d'),
            hackernews_stories=hackernews_stories[:15],  # Top 15 HN stories
            github_trending=github_repos[:15],  # Top 15 GitHub repos
            curated_highlights=highlights,
            tech_categories=categorized_content,
            summary_insights=insights
        )
        
        return report
    
    def format_telegram_report(self, report: TechCurationReport) -> str:
        """Format report for Telegram delivery"""
        output = "💻 **DAILY CURATED TECH REPORT**\n"
        output += f"📅 {report.date}\n\n"
        
        # Curated Highlights
        output += "⭐ **TOP TECH HIGHLIGHTS**\n"
        for i, highlight in enumerate(report.curated_highlights[:5], 1):
            source_emoji = "🔥" if highlight['source'] == 'HackerNews' else "⭐"
            output += f"{source_emoji} **{highlight.get('title', highlight.get('name', 'Unknown'))}**\n"
            output += f"   {highlight['highlight_reason']} | Quality: {highlight['quality_score']:.2f}\n"
            if highlight.get('url'):
                output += f"   🔗 [Link]({highlight['url']})\n"
            output += "\n"
        
        # Summary insights
        output += "📊 **DAILY INSIGHTS**\n"
        for insight_type, insight_text in report.summary_insights.items():
            output += f"• {insight_text}\n"
        
        # Top categories with content
        output += "\n🎯 **TRENDING CATEGORIES**\n"
        for category, items in report.tech_categories.items():
            if items:
                category_name = category.replace('_', ' ').title()
                output += f"**{category_name}**: {len(items)} items\n"
        
        output += f"\n✅ Daily Tech Curation Complete!"
        return output

def main():
    """Main execution function"""
    print("💻 Daily Curated Tech Collection")
    print("=" * 80)
    
    collector = DailyCuratedTechCollector()
    
    # Generate daily report
    report = collector.generate_daily_report()
    
    # Save detailed report
    timestamp = datetime.now().strftime("%Y%m%d")
    output_file = f"daily_curated_tech_{timestamp}.json"
    
    with open(output_file, 'w') as f:
        json.dump(asdict(report), f, indent=2)
    
    # Generate Telegram summary
    telegram_summary = collector.format_telegram_report(report)
    summary_file = f"tech_curation_summary_{timestamp}.txt"
    
    with open(summary_file, 'w') as f:
        f.write(telegram_summary)
    
    print(f"💻 Daily Tech Curation Report Generated!")
    print(f"📄 Detailed Report: {output_file}")
    print(f"📱 Telegram Summary: {summary_file}")
    print(f"⭐ Curated Highlights: {len(report.curated_highlights)}")
    print(f"🔥 HackerNews Stories: {len(report.hackernews_stories)}")
    print(f"⭐ GitHub Trending: {len(report.github_trending)}")
    
    # Display summary
    print("\n" + telegram_summary)

if __name__ == "__main__":
    main()