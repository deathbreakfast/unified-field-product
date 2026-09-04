# uf-appearance

Optional Appearance product offering for Unified Field shells.

Ships the desktop app-bar appearance popover ([`AppBarAppearanceButton`]) and
registers it into the default utilities pack via [`ensure_linked`]. Preference
storage and Valence services remain in [`uf-product`].

Docs: `cargo doc -p uf-appearance --features ssr --open`.

## Verify

```bash
cargo check -p uf-appearance --features ssr
cargo check -p shell-chrome-host --features ssr
```

## Related

- Shell offerings: [`uf-integrations`](../uf-integrations/) (`offering-appearance` / `full`)
- Appearance preferences API: [`uf-product`](../uf-product/)
- Teaching host: [`examples/shell-chrome-host`](../examples/shell-chrome-host/)

[`AppBarAppearanceButton`]: https://docs.rs/uf-appearance/latest/uf_appearance/fn.AppBarAppearanceButton.html
[`ensure_linked`]: https://docs.rs/uf-appearance/latest/uf_appearance/fn.ensure_linked.html
[`uf-product`]: ../uf-product/
