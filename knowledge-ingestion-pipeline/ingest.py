#!/usr/bin/env python3
"""
Knowledge Base Ingestion Script
Part of hermes-data-pipeline project

Ingest PDF/EPUB files into Qdrant knowledge base with proper metadata extraction.
"""

import argparse
import os
import sys
import uuid
import re
from pathlib import Path
from typing import List, Dict, Optional, Tuple

# Core dependencies
from qdrant_client import QdrantClient
from qdrant_client.models import Distance, VectorParams, PointStruct
import requests

# Document processing
try:
    import fitz  # PyMuPDF
except ImportError:
    fitz = None

try:
    import ebooklib
    from ebooklib import epub
    from bs4 import BeautifulSoup
except ImportError:
    ebooklib = None
    BeautifulSoup = None


class DocumentIngester:
    """Ingest documents into Qdrant knowledge base with TEI embeddings."""
    
    def __init__(self, qdrant_url: str = "http://localhost:6333", tei_url: str = "http://localhost:8082"):
        self.client = QdrantClient(url=qdrant_url)
        self.tei_url = tei_url
        self.chunk_size = 1000  # Characters per chunk
        self.chunk_overlap = 200  # Overlap between chunks
        
    def _get_embedding(self, text: str) -> List[float]:
        """Get embedding from TEI service."""
        try:
            response = requests.post(
                f"{self.tei_url}/embed",
                json={"inputs": text},
                headers={"Content-Type": "application/json"},
                timeout=30
            )
            response.raise_for_status()
            return response.json()[0]  # TEI returns list of embeddings
        except Exception as e:
            print(f"TEI embedding error: {e}")
            # Fallback to zero vector with correct dimension (768 for multilingual-e5-base)
            return [0.0] * 768
    
    def _ensure_collection(self, collection_name: str):
        """Ensure Qdrant collection exists with correct vector configuration."""
        try:
            collections = self.client.get_collections()
            existing_names = [c.name for c in collections.collections]
            
            if collection_name not in existing_names:
                print(f"Creating collection: {collection_name}")
                self.client.create_collection(
                    collection_name=collection_name,
                    vectors_config=VectorParams(size=768, distance=Distance.COSINE),
                )
            else:
                print(f"Collection exists: {collection_name}")
                
        except Exception as e:
            print(f"Error with collection {collection_name}: {e}")
            sys.exit(1)
    
    def _chunk_text(self, text: str, title: str = "", chapter: str = "") -> List[str]:
        """Split text into overlapping chunks."""
        # Clean text
        text = re.sub(r'\s+', ' ', text.strip())
        
        if len(text) <= self.chunk_size:
            return [text]
        
        chunks = []
        start = 0
        
        while start < len(text):
            end = start + self.chunk_size
            
            # Try to break at sentence boundary
            if end < len(text):
                # Look for sentence endings in the last 100 characters
                break_point = text.rfind('.', start + self.chunk_size - 100, end)
                if break_point > start:
                    end = break_point + 1
            
            chunk = text[start:end].strip()
            if chunk:
                chunks.append(chunk)
            
            # Move start forward with overlap
            start = max(start + 1, end - self.chunk_overlap)
        
        return chunks
    
    def _extract_pdf_metadata(self, pdf_path: str) -> Tuple[str, str]:
        """Extract title and author from PDF metadata."""
        if not fitz:
            return "", ""
            
        try:
            doc = fitz.open(pdf_path)
            metadata = doc.metadata
            title = metadata.get('title', '') or Path(pdf_path).stem
            author = metadata.get('author', '')
            doc.close()
            return title, author
        except Exception as e:
            print(f"PDF metadata extraction error: {e}")
            return Path(pdf_path).stem, ""
    
    def _process_pdf(self, pdf_path: str) -> List[Dict]:
        """Process PDF file and extract text with metadata."""
        if not fitz:
            print("ERROR: PyMuPDF not installed. Install with: pip install PyMuPDF")
            return []
        
        try:
            doc = fitz.open(pdf_path)
            title, author = self._extract_pdf_metadata(pdf_path)
            
            documents = []
            
            # EXPANDED MODE: Process first 20 pages for better content base
            max_pages = min(20, len(doc))
            for page_num in range(max_pages):
                page = doc[page_num]
                text = page.get_text()
                
                if not text.strip():
                    continue
                
                # Determine chapter/section from page content
                chapter = f"Page {page_num + 1}"
                
                # Look for chapter headings (basic heuristic)
                lines = text.split('\n')
                for line in lines[:5]:  # Check first few lines
                    line = line.strip()
                    if (line and len(line) < 100 and 
                        ('chapter' in line.lower() or 'part' in line.lower() or
                         line.isupper() or re.match(r'^[A-Z][^a-z]*$', line))):
                        chapter = line
                        break
                
                # Chunk the page text
                chunks = self._chunk_text(text, title, chapter)
                
                for i, chunk in enumerate(chunks):
                    if len(chunk.strip()) < 50:  # Skip very short chunks
                        continue
                        
                    documents.append({
                        "text": chunk,
                        "title": title,
                        "author": author,
                        "chapter": f"{chapter} (part {i+1})" if len(chunks) > 1 else chapter,
                        "type": "book",
                        "source_page": page_num + 1,
                        "ticker": ""  # For books, not stocks
                    })
            
            doc.close()
            print(f"Extracted {len(documents)} chunks from PDF: {title}")
            return documents
            
        except Exception as e:
            print(f"PDF processing error: {e}")
            return []
    
    def _process_epub(self, epub_path: str) -> List[Dict]:
        """Process EPUB file and extract text with metadata."""
        if not ebooklib or not BeautifulSoup:
            print("ERROR: ebooklib and beautifulsoup4 not installed. Install with: pip install ebooklib beautifulsoup4")
            return []
        
        try:
            book = epub.read_epub(epub_path)
            
            # Extract metadata
            title = book.get_metadata('DC', 'title')[0][0] if book.get_metadata('DC', 'title') else Path(epub_path).stem
            authors = book.get_metadata('DC', 'creator')
            author = authors[0][0] if authors else ""
            
            documents = []
            
            for item in book.get_items():
                if item.get_type() == ebooklib.ITEM_DOCUMENT:
                    # Extract text from HTML
                    soup = BeautifulSoup(item.get_content(), 'html.parser')
                    text = soup.get_text()
                    
                    if not text.strip():
                        continue
                    
                    # Try to get chapter title from HTML
                    chapter = "Unknown Chapter"
                    try:
                        h_tags = soup.find_all(['h1', 'h2', 'h3']) if soup else []
                        if h_tags:
                            chapter = h_tags[0].get_text().strip()
                        elif hasattr(item, 'file_name'):
                            chapter = Path(item.file_name).stem
                    except Exception as e:
                        print(f"Chapter extraction warning: {e}")
                        chapter = f"Chapter {len(documents)//10 + 1}"
                    
                    # Chunk the chapter text
                    chunks = self._chunk_text(text, title, chapter)
                    
                    for i, chunk in enumerate(chunks):
                        if len(chunk.strip()) < 50:  # Skip very short chunks
                            continue
                            
                        documents.append({
                            "text": chunk,
                            "title": title,
                            "author": author,
                            "chapter": f"{chapter} (part {i+1})" if len(chunks) > 1 else chapter,
                            "type": "book",
                            "source_file": item.file_name if hasattr(item, 'file_name') else "",
                            "ticker": ""  # For books, not stocks
                        })
            
            print(f"Extracted {len(documents)} chunks from EPUB: {title}")
            return documents
            
        except Exception as e:
            print(f"EPUB processing error: {e}")
            return []
    
    def ingest_file(self, file_path: str, collection_name: str) -> bool:
        """Ingest a single file into the specified collection."""
        file_path = Path(file_path)
        
        if not file_path.exists():
            print(f"ERROR: File not found: {file_path}")
            return False
        
        print(f"Processing: {file_path}")
        print(f"Target collection: {collection_name}")
        
        # Ensure collection exists
        self._ensure_collection(collection_name)
        
        # Process file based on extension
        documents = []
        
        if file_path.suffix.lower() == '.pdf':
            documents = self._process_pdf(str(file_path))
        elif file_path.suffix.lower() == '.epub':
            documents = self._process_epub(str(file_path))
        else:
            print(f"ERROR: Unsupported file type: {file_path.suffix}")
            return False
        
        if not documents:
            print("ERROR: No documents extracted")
            return False
        
        # Generate embeddings and ingest
        print(f"Generating embeddings for {len(documents)} chunks...")
        points = []
        
        for i, doc in enumerate(documents):
            if i % 10 == 0:
                print(f"  Processing chunk {i+1}/{len(documents)}")
            
            # Generate embedding
            embedding = self._get_embedding(doc["text"])
            
            # Create point
            point = PointStruct(
                id=str(uuid.uuid4()),
                vector=embedding,
                payload=doc
            )
            points.append(point)
        
        # Batch upload to Qdrant
        print(f"Uploading {len(points)} points to Qdrant...")
        try:
            self.client.upsert(
                collection_name=collection_name,
                points=points
            )
            print(f"✅ Successfully ingested {len(points)} chunks")
            return True
            
        except Exception as e:
            print(f"ERROR: Qdrant upload failed: {e}")
            return False


def main():
    parser = argparse.ArgumentParser(description="Ingest PDF/EPUB files into Qdrant knowledge base")
    parser.add_argument("--file", required=True, help="Path to PDF or EPUB file")
    parser.add_argument("--collection", required=True, help="Target Qdrant collection name")
    parser.add_argument("--qdrant-url", default="http://localhost:6333", help="Qdrant server URL")
    parser.add_argument("--tei-url", default="http://localhost:8082", help="TEI embedding server URL")
    
    args = parser.parse_args()
    
    # Create ingester
    ingester = DocumentIngester(qdrant_url=args.qdrant_url, tei_url=args.tei_url)
    
    # Process file
    success = ingester.ingest_file(args.file, args.collection)
    
    if success:
        print(f"🎉 Ingestion completed successfully!")
        sys.exit(0)
    else:
        print(f"💥 Ingestion failed!")
        sys.exit(1)


if __name__ == "__main__":
    main()