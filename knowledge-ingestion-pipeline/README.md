# Knowledge Ingestion Pipeline

Ingests books, PDFs, and documents into vector database for semantic search and RAG.

## Data Sources

- **PDF** — Books, research papers, reports
- **EPUB** — E-books
- **TXT/MD** — Plain text, markdown notes

## Target Collections

| Collection | Purpose |
|------------|---------|
| `pagupon-kb` | Investing, business, finance books |
| `pondo-business-kb` | F&B business knowledge |

## Architecture

```
┌─────────────────┐
│  Documents      │
│  (PDF/EPUB/TXT) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Extractor     │  PyMuPDF, marker-pdf, ebooklib
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Chunker       │  Semantic chunking (512-1024 tokens)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Embedder      │  TEI multilingual-e5-base (768 dim)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Qdrant       │  Vector storage + metadata
└─────────────────┘
```

## Features (Planned)

- [ ] PDF text extraction (PyMuPDF)
- [ ] OCR for scanned PDFs (marker-pdf)
- [ ] EPUB extraction (ebooklib)
- [ ] Semantic chunking with overlap
- [ ] Metadata extraction (title, author, chapter)
- [ ] Deduplication by content hash
- [ ] Progress tracking and resume

## Requirements

- Python 3.10+
- PyMuPDF (fitz)
- marker-pdf (for OCR)
- ebooklib
- TEI embedding service
- Qdrant vector database

## Usage

```bash
# Install deps
pip install -r requirements.txt

# Ingest a PDF
python ingest.py --file book.pdf --collection pagupon-kb

# Ingest a directory
python ingest.py --dir ~/Books/investing/ --collection pagupon-kb

# List ingested documents
python ingest.py --list --collection pagupon-kb
```

## Environment Variables

```bash
# Services
TEI_URL=http://localhost:8082
QDRANT_URL=http://localhost:6333

# Chunking
CHUNK_SIZE=512
CHUNK_OVERLAP=50
```

## Chunk Metadata

Each chunk stored with:
- `source_file` — Original filename
- `title` — Document title
- `author` — Document author (if available)
- `chapter` — Chapter/section name
- `page` — Page number (PDF only)
- `chunk_index` — Position in document
- `content_hash` — For deduplication
