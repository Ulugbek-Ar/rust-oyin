# ZULMAT

Neon arena survivor, Rust + macroquad. Zulmatda yorug'lik bo'l.

## Ishga tushirish
```
cargo run --release
```

## Android APK
GitHub Actions har push da APK yig'adi (Actions → ZULMAT-apk). Telegramga avtomatik yuborish uchun repo Secrets ga `TELEGRAM_BOT_TOKEN` va `TELEGRAM_CHAT_ID` qo'shing; commit xabarida `[tg]` bo'lsa yoki workflow qo'lda ishga tushirilsa kanalga jo'natiladi.

## Boshqaruv
- **WASD / strelkalar** — harakat
- **Sichqoncha** — nishon (o'q avtomatik otiladi)
- **Space / O'ng tugma** — dash (qisqa daxlsizlik)
- **1/2/3 yoki klik** — daraja oshganda kuchaytirish tanlash
- **Telefonda:** chap tomon — joystik, o'ng tomon — dash, nishon avtomatik

## Dushmanlar
- Uchburchak — quvuvchi
- Kvadrat — sakrovchi (sariq chiziq = hujum ogohlantirishi)
- Beshburchak — bo'linuvchi
- Oltiburchak — tank
