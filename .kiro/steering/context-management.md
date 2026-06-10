# Context Management

Panduan mengelola context window agar agent tetap efektif di session panjang.
Terinspirasi dari ECC (Everything Claude Code) strategic compaction patterns.

## Kapan Harus Compact

Compact context di breakpoint logis:
- Setelah fase riset/eksplorasi, sebelum mulai implementasi
- Setelah menyelesaikan satu milestone, sebelum mulai yang berikutnya
- Setelah debugging selesai, sebelum lanjut feature work
- Setelah pendekatan gagal, sebelum coba pendekatan baru

## Kapan JANGAN Compact

- Di tengah implementasi (kehilangan variable names, file paths, partial state)
- Saat ada multi-file refactor yang belum selesai
- Saat debugging yang butuh full stack trace context

## Context Window Rules

### Hindari Last 20% Context untuk:
- Large-scale refactoring (multi-file)
- Feature implementation yang span banyak file
- Debugging complex interactions antar service

### Task yang Aman di Low Context:
- Single-file edits
- Independent utility creation
- Documentation updates
- Simple bug fixes

## Strategi untuk Session Panjang

1. **Checkpoint mental** — setelah setiap significant step, summarize apa yang sudah done dan apa yang tersisa
2. **File-first, memory-second** — tulis progress ke file (progress.md, TODO comments) daripada rely on context memory
3. **Scope narrowing** — jika context mulai penuh, fokus ke satu file/function, selesaikan, baru pindah
4. **Parallel delegation** — untuk task besar, delegate ke sub-agent yang punya fresh context

## Anti-Patterns

- ❌ Membaca semua file sekaligus di awal session
- ❌ Menyimpan full file content di memory padahal hanya butuh beberapa baris
- ❌ Tidak pernah compact sampai auto-compact trigger (terlalu late)
- ✅ Baca file on-demand, hanya bagian yang relevan
- ✅ Compact proaktif di breakpoint logis
- ✅ Gunakan grep/search untuk navigasi, bukan read-all
