# Knowledge base

## Copy pasting in the browser

To enable copy-pasting in the browser, you will need to add the `--cfg=web_sys_unstable_apis` flag. For more information, please refer to the following links :

- <https://github.com/emilk/egui/discussions/>
- <https://github.com/emilk/eframe_template/blob/main/.cargo/config.toml#L6>

## Coverage

Code coverage made by [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) is available at <https://tramex.github.io/tramex/coverage/>

## CORS Errors

If the website is running on HTTPS, connections to insecure endpoints (like `ws` instead of `wss`) are forbidden. You have three options to solve this issue:

- disable CORS in your browser
- add an SSL certificate to your ws server
- use a WS proxy to translate the `ws` connection to a secure one (`wss`)
- use a local WS proxy to remove CORS issue

```bash
# example of a WS proxy (and with a orign header)
npx @n4n5/proxy-ws -t ws://10.0.0.1:9001 -h '{"origin":"toto"}'
# will redirect ws://127.0.0.1:9001 -> ws://10.0.0.1:9001
# cors will not be an issue anymore with the local address 127.0.0.1
```
