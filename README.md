# ZULMAT QAL'ASI

Piksel action RPG, Rust + macroquad. Grafika, ovoz va musiqa to'liq kod bilan yaratilgan.

Qilich kombosi, sakrash, olov sehri, iksirlar; tasodifiy qavatlar, eshiklari yopiladigan xonalar,
xazina va boss (Soya Lordi); daraja va qobiliyat tanlash.

## Ishga tushirish
```
cargo run --release              # kompyuter
cargo build --release --target wasm32-unknown-unknown   # brauzer (site/ papkasi)
```

## Boshqaruv
- **WASD** — yurish, **J / sichqoncha** — qilich, **Space** — sakrash, **K / o'ng tugma** — olov, **Q** — iksir
- **Telefonda:** chap tomon — joystik, o'ngda HUJUM / SAKRASH / OLOV / IKSIR tugmalari

## Android APK
GitHub Actions har push da APK yig'adi, apksigner bilan imzolaydi va
`https://github.com/Ulugbek-Ar/rust-oyin/releases/download/apk/ZULMAT.apk` ga joylaydi.
Ikonkalarni qayta yaratish: `cargo run --release -- --icons`.
