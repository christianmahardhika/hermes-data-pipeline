# Research-First Development

Selalu riset dan baca code yang ada SEBELUM menulis code baru.
Diadaptasi dari ECC "search-first" skill dan "Read before you write" principle.

## Core Principle

> "Looks orthogonal" is dangerous. If unsure why code is structured a way, ask.

Sebelum menambah code, WAJIB baca:
1. Exports dari module yang akan diubah
2. Immediate callers (siapa yang pakai function/struct ini)
3. Shared utilities yang mungkin sudah solve problem yang sama

## Checklist Sebelum Menulis Code

### Untuk Handler/Endpoint Baru:
- [ ] Baca `routes/routes.go` — cek endpoint conflicts
- [ ] Baca `middleware/auth.go` — pahami auth extraction pattern
- [ ] Baca minimal 1 existing handler yang mirip — ikuti pattern-nya
- [ ] Grep model yang akan dipakai — cek semua consumers

### Untuk Model/Struct Baru:
- [ ] Baca `models/*.go` — cek apakah sudah ada struct serupa
- [ ] Grep field names — hindari naming conflicts
- [ ] Cek migration history — apakah table sudah pernah ada

### Untuk Service/Business Logic:
- [ ] Baca existing services — cek apakah logic sudah ada
- [ ] Baca interfaces — pahami contract yang harus dipenuhi
- [ ] Cek `utils/` — mungkin helper sudah tersedia

### Untuk Frontend Component:
- [ ] Baca existing components serupa — ikuti pattern
- [ ] Cek `services/` — API client mungkin sudah ada
- [ ] Cek `stores/` — state management mungkin sudah handle
- [ ] Cek `utils/` — helper functions yang bisa di-reuse

## Anti-Patterns

- ❌ Langsung menulis handler tanpa baca auth middleware pattern
- ❌ Membuat utility function yang sudah ada di codebase
- ❌ Assume format response tanpa baca existing handlers
- ❌ Membuat mock baru padahal sudah ada mock pattern di test files
- ❌ Menulis SQL/query tanpa cek apakah GORM scope sudah ada

## Enforcement

Dari Learning #21: "ALWAYS read the auth middleware source AND at least one existing handler before writing new handlers."

Dari Learning #1: "When adding a field to a model, always grep the codebase for all functions that format/serialize/display that model's data."

Dari Learning #15: "Design review MUST cross-reference routes.go for endpoint conflicts."

## Workflow

```
1. SEARCH — grep/find existing patterns
2. READ — understand the pattern and why it exists  
3. PLAN — decide how new code fits into existing structure
4. WRITE — implement following discovered patterns
5. VERIFY — ensure new code is consistent with existing
```
