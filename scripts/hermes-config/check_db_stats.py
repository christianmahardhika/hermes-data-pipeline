#!/usr/bin/env python3
from arango import ArangoClient
import json

try:
    client = ArangoClient(hosts='http://localhost:8529')
    db = client.db('news_analysis')
    articles = db.collection('articles')

    # Get total count
    total_count = articles.count()
    print(f'Total articles in database: {total_count}')

    # Get Indonesian articles
    indonesian_articles = list(articles.find({'country': 'Indonesia'}))
    print(f'Indonesian articles: {len(indonesian_articles)}')

    # Breakdown by source
    sources = {}
    categories = {}
    stocks = set()
    
    for article in indonesian_articles:
        source = article.get('source', 'Unknown')
        category = article.get('category', 'Unknown')
        sources[source] = sources.get(source, 0) + 1
        categories[category] = categories.get(category, 0) + 1
        
        if 'stock_symbol' in article:
            stocks.add(article['stock_symbol'])

    print('\nSource breakdown:')
    for source, count in sources.items():
        print(f'  {source}: {count} articles')

    print('\nCategory breakdown:')
    for category, count in categories.items():
        print(f'  {category}: {count} articles')
    
    if stocks:
        print(f'\nStock symbols covered: {", ".join(sorted(stocks))}')

    print('\nMost recent Indonesian articles:')
    recent = list(articles.find({'country': 'Indonesia'}, skip=0, limit=5))
    for i, article in enumerate(recent, 1):
        title = article.get('title', 'No title')[:60]
        source = article.get('source', 'Unknown')
        print(f'  {i}. [{source}] {title}...')

except Exception as e:
    print(f"Database query error: {e}")