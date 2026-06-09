# Ziward Frontend Template

TanStack Start, Cloudflare Workers, R2, Supabase SSR, Tauri v2, TanStack Query/Form, Zustand, shadcn/ui Base UI, Tailwind CSS v4, Zod, Biome를 기본으로 쓰는 frontend template입니다.

## Webtoon PD MVP 실행 가이드

이 MVP는 Tauri App에서 원본 파일을 선택하고, local renderer와 `avifenc`로 AVIF tile을 만든 뒤, Worker가 발급한 R2 presigned URL과 Supabase RPC로 업로드 결과를 finalize하는 구조입니다.

지원 입력 파일은 `.clip`, `.psd`, `.psb`, `.jpg`, `.jpeg`, `.png`, `.webp`, `.avif`입니다. Tile 규격은 width 최대 `2048px`, tile height `2048px`, AVIF encode option은 lossless 4:4:4입니다.

### 공통 준비

```sh
pnpm install
uv sync --project src-tauri/sidecars/image-renderer
```

`avifenc`는 `libavif`에 포함된 CLI가 필요합니다. macOS Homebrew 기준 설치 예시는 다음과 같습니다.

```sh
brew install libavif
```

`.clip` 변환을 사용하려면 CLIP to PSD/PSB converter가 별도로 필요합니다. converter가 없으면 `.clip` 입력은 `manual_export_required` error로 실패합니다.

```sh
export WEBTOON_PD_CLIP_TO_PSD=/absolute/path/to/clip_to_psd.py
```

### App 입력값

App 왼쪽 panel에는 backend와 workspace 값을 넣습니다.

| Field | 설명 |
| --- | --- |
| Worker URL | R2 presigned URL을 발급하는 Worker origin. 예: `https://example.workers.dev` |
| Supabase URL | Supabase project URL |
| Supabase anon key | Supabase anon public key |
| Access token | 현재 로그인한 사용자의 Supabase access token |
| Organization ID | 업로드가 속한 organization id |
| Project ID | 작품 id |
| Episode ID | 화수 id |
| Stage ID | 과정 id |
| User ID | 업로드/변환을 실행하는 사용자 id |
| Renderer bin | 선택값. image renderer 실행 파일 또는 script absolute path |
| avifenc bin | 선택값. `avifenc` absolute path |

Tool path 탐색 우선순위는 `App 입력값` → `환경변수` → `app resource/binaries` → `PATH`입니다.

| Tool | 환경변수 | 기본 PATH 이름 |
| --- | --- | --- |
| image renderer | `WEBTOON_PD_IMAGE_RENDERER_BIN` | `image-renderer` |
| AVIF encoder | `WEBTOON_PD_AVIFENC_BIN` | `avifenc` |
| CLIP converter | `WEBTOON_PD_CLIP_TO_PSD` | `clip_to_psd.py` 또는 `clip_to_psd` |

Python renderer를 dev에서 그대로 쓰려면 `uv run` wrapper를 만들고 `Renderer bin` 또는 `WEBTOON_PD_IMAGE_RENDERER_BIN`에 wrapper path를 넣습니다. `Renderer bin`에 `.py` file을 직접 넣으면 app은 `python3 <script.py>`로 실행하므로 해당 `python3` 환경에 `Pillow`, `psd-tools`, `pillow-avif-plugin`이 설치되어 있어야 합니다.

```sh
PROJECT_ROOT="$(pwd)"
cat > /tmp/webtoon-pd-image-renderer <<SH
#!/bin/sh
exec uv run --project "$PROJECT_ROOT/src-tauri/sidecars/image-renderer" python "$PROJECT_ROOT/src-tauri/sidecars/image-renderer/image_renderer.py" "\$@"
SH
chmod +x /tmp/webtoon-pd-image-renderer
export WEBTOON_PD_IMAGE_RENDERER_BIN=/tmp/webtoon-pd-image-renderer
```

### Dev 실행

Tauri dev는 `src-tauri/tauri.conf.json`의 `devUrl`인 `http://localhost:3000`을 사용합니다. `beforeDevCommand`가 `pnpm run dev`로 설정되어 있으므로 `pnpm run tauri:dev`만 실행하면 Vite dev server와 Tauri App이 함께 실행됩니다.

```sh
WEBTOON_PD_IMAGE_RENDERER_BIN=/tmp/webtoon-pd-image-renderer \
WEBTOON_PD_AVIFENC_BIN=/opt/homebrew/bin/avifenc \
pnpm run tauri:dev
```

Dev server 없이 static frontend만 확인하려면 Tauri용 frontend build를 만들 수 있습니다.

```sh
pnpm run build:tauri-frontend
```

Python renderer만 standalone으로 확인하려면 sample 입력 파일을 넣어 manifest와 PNG tile을 생성합니다.

```sh
uv run --project src-tauri/sidecars/image-renderer python src-tauri/sidecars/image-renderer/image_renderer.py \
  --input /absolute/path/source.png \
  --output-dir /tmp/webtoon-rendered \
  --max-width 2048 \
  --tile-height 2048 \
  --manifest /tmp/webtoon-rendered/manifest.json
```

### Prod 실행

배포용 desktop bundle은 OS별 machine에서 빌드합니다.

```sh
pnpm run tauri:build
```

macOS bundle을 직접 실행하는 예시는 다음과 같습니다.

```sh
open "src-tauri/target/release/bundle/macos/Ziward Frontend Template.app"
```

Bundle packaging 없이 release binary smoke test만 하려면 다음 command를 사용합니다.

```sh
pnpm exec tauri build --no-bundle
./src-tauri/target/release/ziward-frontend-template
```

Production에서 `.app`을 Finder나 `open`으로 실행하면 shell 환경변수가 전달되지 않을 수 있습니다. 이 경우 App의 `Renderer bin`, `avifenc bin` field에 absolute path를 넣거나, sidecar binary를 app resource에 포함해야 합니다. 현재 Rust code는 app resource directory의 `binaries/image-renderer-<target-triple>`, `binaries/image-renderer`, `binaries/avifenc-<target-triple>`, `binaries/avifenc`를 찾습니다. 실제 distribution에 포함하려면 `src-tauri/tauri.conf.json`의 bundle resource 또는 external binary 설정을 추가해야 합니다.

Cloudflare Workers API를 배포해야 실제 R2 presigned URL 발급이 가능합니다.

```sh
pnpm run deploy
```

### Backend Contract

Tauri App이 기대하는 Worker/Supabase contract는 다음과 같습니다.

| 구분 | Endpoint |
| --- | --- |
| R2 upload URL 발급 | `POST {Worker URL}/api/r2/presigned-put-urls` |
| Asset finalize | `POST {Supabase URL}/rest/v1/rpc/finalize_asset_processing` |
| WYSIWYG 저장 | `POST {Supabase URL}/rest/v1/rpc/upsert_wysiwyg_document` |
| WYSIWYG 조회 | `GET {Supabase URL}/rest/v1/wysiwyg_documents?stage_id=eq.<Stage ID>` |

`POST /api/r2/presigned-put-urls` 요청은 Supabase access token을 `Authorization: Bearer <token>`으로 보내고, body는 다음 shape를 사용합니다.

```json
{
  "objects": [
    {
      "key": "originals/example.png",
      "contentType": "image/png",
      "sizeBytes": 1234
    }
  ]
}
```

응답은 object별 `url`과 선택적인 `headers`를 반환해야 합니다.

```json
{
  "urls": [
    {
      "key": "originals/example.png",
      "url": "https://...",
      "headers": {}
    }
  ]
}
```

업로드 전 변환 실패는 Supabase나 local state에 기록하지 않습니다. 원본과 AVIF tile upload가 끝난 뒤 `finalize_asset_processing` RPC를 호출합니다.

## Dependency Policy

- package manager는 `pnpm`만 사용합니다.
- scaffold 기준 command는 `pnpm dlx shadcn@latest init --preset b1GKwL7Wi --base base --template start --pointer`입니다.
- `shadcn add`가 특정 component에 필요한 dependency를 설치하는 것은 허용하지만, 아직 쓰지 않는 component dependency를 미리 설치하지 않습니다.
- TanStack Start RSC는 experimental이지만 template에서 활성화합니다.
- `tw-animate-css`는 `shadcn init` baseline에서 자동 추가되는 dependency라서 유지합니다.
- test를 위한 dependency와 script는 기본 template에 넣지 않습니다.
- lint/format/import organize는 Biome 하나로 통일합니다. ESLint와 Prettier는 기본 template에서 제거합니다.
- Cloudflare Workers runtime에서는 AWS S3 SDK를 쓰지 않고 R2 bucket binding을 사용합니다.
- Tauri desktop target은 Mac/Windows package를 기준으로 두고, Cloudflare SSR build와 분리된 Vite SPA build를 사용합니다.

## Confirmed Dependencies

### Core

```sh
pnpm add @tanstack/react-query @tanstack/react-form zod zustand @supabase/supabase-js @supabase/ssr use-sync-external-store
```

`shadcn init`과 TanStack Start scaffold가 만드는 framework/UI dependency는 유지합니다.

- `@tanstack/react-start`
- `@tanstack/react-router`
- `@tanstack/react-router-ssr-query`
- `@tanstack/react-devtools`
- `@tanstack/react-router-devtools`
- `react`
- `react-dom`
- `@base-ui/react`
- `@phosphor-icons/react`
- `class-variance-authority`
- `clsx`
- `tailwind-merge`
- `tailwindcss`
- `@tailwindcss/vite`
- `tw-animate-css`
- `shadcn`
- `vite-tsconfig-paths`
- `@tauri-apps/api`
- `use-sync-external-store`

### Cloudflare

```sh
pnpm add -D @cloudflare/vite-plugin wrangler
```

Cloudflare Workers deploy target은 `wrangler`와 `@cloudflare/vite-plugin`을 사용합니다. R2는 `wrangler.jsonc`의 `r2_buckets` binding으로 연결하고 application code에서는 binding으로 받은 `R2Bucket` API를 사용합니다.

### Tauri

```sh
pnpm add @tauri-apps/api
pnpm add -D @tauri-apps/cli
```

Tauri v2 desktop target은 `src-tauri`에 둡니다. `pnpm run build`는 Cloudflare/TanStack Start RSC build로 유지하고, Tauri bundle에 들어가는 static frontend는 `pnpm run build:tauri-frontend`가 `vite.tauri.config.ts`로 `dist/tauri`에 생성합니다. Dev/prod 실행 command와 변환 tool option은 `Webtoon PD MVP 실행 가이드`를 따릅니다.

Mac bundle에는 `icon.icns`, Windows bundle에는 `icon.ico`와 Windows AppX icon set을 사용합니다.

### Tooling

```sh
pnpm add -D @biomejs/biome typescript vite @vitejs/plugin-react @vitejs/plugin-rsc @tanstack/router-plugin @tanstack/devtools-vite @types/node @types/react @types/react-dom
```

기본 script는 다음만 둡니다.

```json
{
  "scripts": {
    "dev": "vite dev --port 3000",
    "build": "vite build",
    "build:tauri-frontend": "vite build --config vite.tauri.config.ts",
    "preview": "vite preview",
    "deploy": "pnpm run build && wrangler deploy",
    "cf-typegen": "wrangler types",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build",
    "check": "pnpm run typecheck && pnpm run lint",
    "lint": "biome check . --write --unsafe",
    "format": "biome format . --write",
    "typecheck": "tsc --noEmit"
  }
}
```

## Removed From Scaffold

`shadcn init --template start`가 생성하더라도 이 template baseline에서는 제거합니다.

```sh
pnpm remove @tanstack/eslint-config prettier prettier-plugin-tailwindcss vitest @testing-library/dom @testing-library/react jsdom @fontsource-variable/noto-sans nitro
```

관련 script와 CSS import도 함께 제거합니다.

- `test` script 제거
- `eslint` 기반 `lint` script 제거
- `prettier` 기반 `format` script 제거
- `eslint.config.js` 제거
- `.prettierrc` 제거
- `.prettierignore` 제거
- `@import "@fontsource-variable/noto-sans";` 제거
- `nitro()` Vite plugin 제거

`sonner` 같은 component-specific dependency는 실제로 `pnpm dlx shadcn@latest add ...`를 실행해서 선택한 component가 요구할 때 설치합니다.

## UI Baseline

`components.json`은 scaffold 결과를 source of truth로 둡니다.

- `style`: `base-lyra`
- `base`: `base`
- `baseColor`: `neutral`
- `cssVariables`: `true`
- `iconLibrary`: `phosphor`
- `rsc`: `true`
- `rtl`: `false`
- `pointer`: enabled
- `radius`: `0.625rem`

## Font

Font는 CDN stylesheet를 document head에서 preload한 뒤 stylesheet로 link하고 Tailwind CSS v4 theme token에서 `--font-sans`로 사용합니다.

```html
<link rel="preload" href="https://cdn.jsdelivr.net/gh/sun-typeface/SUIT@2/fonts/variable/woff2/SUIT-Variable.css" as="style" />
<link rel="preload" href="https://cdn.jsdelivr.net/gh/orioncactus/pretendard@v1.3.9/dist/web/variable/pretendardvariable.css" as="style" />
<link rel="stylesheet" href="https://cdn.jsdelivr.net/gh/sun-typeface/SUIT@2/fonts/variable/woff2/SUIT-Variable.css" />
<link rel="stylesheet" href="https://cdn.jsdelivr.net/gh/orioncactus/pretendard@v1.3.9/dist/web/variable/pretendardvariable.css" />
```

```css
@theme inline {
  --font-heading: var(--font-sans);
  --font-sans: "SUIT Variable", "Pretendard Variable", -apple-system, BlinkMacSystemFont, system-ui, Roboto, "Helvetica Neue", "Segoe UI", "Apple SD Gothic Neo", "Noto Sans KR", "Malgun Gothic", "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", sans-serif;
}
```

## State Boundaries

- TanStack Query: server state, cache, mutation, invalidation
- TanStack Router: route state, loader, search params
- TanStack Form: form state and validation
- Zustand: client-only UI/application state
- Supabase SSR: cookie-based auth session and authenticated server/client clients
- R2 binding: server-side object storage access

Zustand에는 Supabase session, access token, refresh token, R2 object data 같은 server-owned state를 저장하지 않습니다.

## TanStack Start RSC

RSC는 TanStack Start에서 experimental입니다. 이 template은 부분 적용을 전제로 RSC build pipeline만 활성화합니다.

```ts
tanstackStart({
  rsc: {
    enabled: true,
  },
});
```

`@vitejs/plugin-rsc`는 `tanstackStart(...)` 뒤, `viteReact()` 앞에 둡니다.

```ts
plugins: [
  cloudflare({ viteEnvironment: { name: "ssr" } }),
  devtools(),
  viteTsConfigPaths({ projects: ["./tsconfig.json"] }),
  tailwindcss(),
  tanstackStart({ rsc: { enabled: true } }),
  rsc(),
  viteReact(),
];
```

RSC로 렌더링하는 UI는 `@tanstack/react-start/rsc`의 helper를 사용합니다. 일반 route/component를 자동으로 server component로 바꾸지 않습니다.

```tsx
import { createServerFn } from "@tanstack/react-start";
import { renderServerComponent } from "@tanstack/react-start/rsc";

function Greeting() {
  return <h1>Hello from RSC</h1>;
}

const getGreeting = createServerFn().handler(async () => {
  const Renderable = await renderServerComponent(<Greeting />);
  return { Renderable };
});
```

shadcn/ui는 `components.json`의 `rsc: true`를 사용합니다. CLI가 client component에 `use client` directive를 추가할 수 있게 하기 위함입니다.

## Cloudflare Environment And Bindings

Cloudflare Workers target에서는 server-side code에서 `cloudflare:workers`의 `env` object로 environment variables, secrets, bindings를 읽습니다. module scope에서 값을 복사해 두지 말고 request/server function 실행 시점에 읽습니다.

Regular env variables는 `wrangler.jsonc`의 `vars`에 넣으면 `pnpm run deploy` 때 Worker에 적용됩니다.

```jsonc
{
  "vars": {
    "SUPABASE_URL": "https://example.supabase.co",
    "SUPABASE_ANON_KEY": "replace-with-supabase-anon-key",
    "PUBLIC_APP_ORIGIN": "https://ziward-frontend-template.<account>.workers.dev"
  }
}
```

Secret value는 `wrangler.jsonc`에 넣지 않습니다. Wrangler command 또는 Cloudflare dashboard로 등록합니다.

```sh
pnpm wrangler secret put SUPABASE_SERVICE_ROLE_KEY
```

CI/CD에서 code deploy와 secret update를 같은 command로 처리해야 하면 `.env.production` 또는 JSON file을 만들고 `--secrets-file`을 사용합니다. 실제 secret file은 commit하지 않습니다.

```sh
pnpm wrangler deploy --secrets-file .env.production
```

Local development에서는 `.dev.vars` 또는 `.env` 중 하나만 사용합니다. 이 repo는 `.dev.vars.example`을 예시로 둡니다.

```sh
cp .dev.vars.example .dev.vars
```

Required binding:

```jsonc
{
  "r2_buckets": [
    {
      "binding": "R2_BUCKET",
      "bucket_name": "ziward-frontend-template"
    }
  ]
}
```

`wrangler.jsonc`의 `vars`, `secrets.required`, binding이 바뀌면 type을 다시 생성합니다.

```sh
pnpm run cf-typegen
```

## Cloudflare Guide Check

Cloudflare TanStack Start guide 기준 적용 상태입니다.

| Item | Before | After |
| --- | --- | --- |
| Cloudflare package | scaffold에는 deploy target package가 없음 | `@cloudflare/vite-plugin`, `wrangler` 추가 |
| Deployment adapter | `nitro` generic deploy adapter | `cloudflare` deploy adapter로 교체 |
| Vite plugin | `tanstackStart()`와 React, `nitro()` 중심 | `cloudflare({ viteEnvironment: { name: "ssr" } })`와 `rsc()`를 추가하고 `nitro()` 제거 |
| Wrangler config | 없음 | `wrangler.jsonc` 추가, `main`은 `@tanstack/react-start/server-entry` |
| Compatibility | 없음 | `compatibility_date`, `nodejs_compat`, `observability.enabled` 설정 |
| Scripts | `dev`, `build`, `preview` 중심 | `deploy`, `cf-typegen`, `build:tauri-frontend`, `tauri:dev`, `tauri:build` 추가 |
| R2 binding | 없음 | `R2_BUCKET` binding 추가, `worker-configuration.d.ts` 생성 |
| Env variables | README의 rough env 이름만 있음 | deploy에 적용되는 `vars`, `wrangler secret put`, `--secrets-file` 예시 추가 |

### Nitro Decision

Nitro는 TanStack CLI 기준으로 “Generic Nitro adapter”입니다. Cloudflare가 아닌 Node-compatible host까지 열어두는 이점이 있지만, 이 template은 Cloudflare Workers 전용 배포를 기본값으로 잡습니다. TanStack CLI에서도 `cloudflare`와 `nitro`는 둘 다 deployment add-on이고 exclusive 관계라서 동시에 유지하지 않습니다.

`nitro()`를 제거한 뒤 `pnpm run check`와 `pnpm run build`가 통과했으므로 baseline에서는 제거합니다.

### TanStack CLI Metadata

`.cta.json`은 Cloudflare deploy와 runtime에는 필요하지 않지만, TanStack CLI의 `tanstack add ...` 흐름에서 현재 project scaffold 상태를 파악하는 metadata입니다. 이 template에서는 `chosenAddOns`를 `["biome", "cloudflare"]`로 복원합니다.

## Explicitly Excluded

- `@aws-sdk/*`
- `aws-sdk`
- `eslint`
- `prettier`
- `husky`
- `lint-staged`
- `vitest`
- `@playwright/test`
- `@testing-library/*`
- `react-hook-form`
- `formik`

## References

- [shadcn CLI](https://ui.shadcn.com/docs/cli)
- [shadcn components.json](https://ui.shadcn.com/docs/components-json)
- [Cloudflare TanStack Start guide](https://developers.cloudflare.com/workers/framework-guides/web-apps/tanstack-start/)
- [Cloudflare Workers environment variables](https://developers.cloudflare.com/workers/configuration/environment-variables/)
- [Cloudflare Workers secrets](https://developers.cloudflare.com/workers/configuration/secrets/)
- [Cloudflare R2 Workers API](https://developers.cloudflare.com/r2/get-started/workers-api/)
- [Supabase SSR Auth](https://supabase.com/docs/guides/auth/server-side)
- [TanStack Start environment variables](https://tanstack.com/start/latest/docs/framework/react/guide/environment-variables)
- [Tauri v2 configuration](https://v2.tauri.app/reference/config/)
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/)
