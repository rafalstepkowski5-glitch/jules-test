# Proposal: cargo-deny policy and audit report

## Summary
- Wykonane kroki:
  - Zamiana `sqlx` na `sqlx-core` + `sqlx-sqlite` w `Cargo.toml` i przerobienie kodu w `src/db.rs` na ręczne mapowanie wierszy (parametryzacja zapytań).
  - Regeneracja `Cargo.lock` i uruchomienie `cargo audit` — wynik: brak krytycznych podatności zgłaszających `rsa` (usunięto z lockfile).

## Cel proponowanej polityki
- Utrzymać jasno zdefiniowaną listę dopuszczalnych licencji (allowlist).
- Zablokować (lub wymusić przegląd) pakiety z krytycznymi podatnościami lub niedozwolonymi licencjami.
- Umożliwić dodanie uzasadnionych wyjątków (wyłączeń) po przeglądzie zespołu bezpieczeństwa.

## Rekomendowane kroki operacyjne
1. Dodać plik `deny.toml` z polityką licencji i podstawowymi regułami (przykład poniżej).
2. Dodać do CI krok `cargo deny check advisories bans licenses sources` (już jest w `workflows/release.yml`).
3. Jeśli `cargo deny` zwróci odmowy na istniejące zależności, zarejestruj je w raporcie i rozważ:
   - Upgrade zależności,
   - Zastąpienie alternatywną biblioteką,
   - Przygotowanie wyjątku z akceptacją ryzyka podpisaną przez security owner.

## Proposed `deny.toml` (starter)

```toml
# Allow only commonly used permissive licenses
[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause", "ISC"]

# Treat advisories as failures (no automatic ignore)
[advisories]
# no 'allow' entries by default; team must explicitly accept exceptions
ignore = []

# Optional bans for crates we never want pulled (example)
[bans.crates]
# crates = ["some-unsafe-crate"]

# Sources can be restricted (disallow git dependencies if desired)
[sources]
# blocked = ["git+https://github.com/suspicious/*"]
```

## Notes & justification
- Wcześniejsze `cargo-audit` zgłaszało `RUSTSEC-2023-0071 (rsa)` — usunięcie holistyczne wymagało usunięcia opcjonalnych backendów `sqlx-mysql` oraz `sqlx-postgres` z lockfile. Zrobiono to przez eksplicytne zależności `sqlx-core` + `sqlx-sqlite` oraz usunięcie derive macros w kodzie.
- Polityka licencji jest konserwatywna; jeżeli projekt potrzebuje szerszego zestawu licencji (np. MPL, LGPL), dodajemy je po przeglądzie prawnym.

## Next steps (do podjęcia przez zespół)
- Przejrzeć proponowany `deny.toml` i zaakceptować lub wskazać dodatkowe licencje.
- Dodać procedurę zatwierdzania wyjątków (jeden PR z uzasadnieniem + akceptacja security owner).
- Rozważyć okresowy skan `cargo audit` w harmonogramie (np. cotygodniowy) i automatyczne raporty.

---

Plik ten można zaktualizować pod kątem polityki organizacyjnej; jeśli chcesz, wdrożę `deny.toml` w repo i uruchomię `cargo deny` aby zebrać obecne odmowy i przygotować listę wyjątków.
