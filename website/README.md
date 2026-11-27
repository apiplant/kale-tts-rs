# kale-tts-rs website

The public project site is a static SolidJS build with separate HTML entries for the overview, examples, documentation, and 404 page. The example files come from the Rust runtime. The local browser tool lives in ../studio and is packaged separately in the Rust executable.

## Build and preview

~~~sh
pnpm install --frozen-lockfile
pnpm build
pnpm run pages:dev
~~~

Cloudflare Pages builds this directory on every push to `main`. It runs `pnpm install --frozen-lockfile && pnpm build` from `website/` and publishes `dist/` to [kale-tts-rs.apiplant.com](https://kale-tts-rs.apiplant.com/). Pull requests get preview deployments.
