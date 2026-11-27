# kale-tts-rs website

The public project site is a static SolidJS build with separate HTML entries for the overview, examples, documentation, and 404 page. The example files come from the Rust runtime. The local browser tool lives in ../studio and is packaged separately in the Rust executable.

## Build and publish

~~~sh
pnpm install --frozen-lockfile
pnpm build
wrangler pages dev dist
~~~

With Wrangler authenticated to the Cloudflare account, run `pnpm run deploy` to publish the build to [kale-tts-rs-website.pages.dev](https://kale-tts-rs-website.pages.dev/).
