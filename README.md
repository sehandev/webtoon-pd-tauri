# Ziward Frontend Template

TanStack Start, Cloudflare Workers, R2, Supabase SSR, Tauri v2, TanStack Query/Form, Zustand, shadcn/ui Base UI, Tailwind CSS v4, Zod, Biome를 기본으로 쓰는 frontend template입니다.

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

Tauri v2 desktop target은 `src-tauri`에 둡니다. `pnpm run build`는 Cloudflare/TanStack Start RSC build로 유지하고, Tauri bundle에 들어가는 static frontend는 `pnpm run build:tauri-frontend`가 `vite.tauri.config.ts`로 `dist/tauri`에 생성합니다.

개발 중에는 dev server를 별도로 실행한 뒤 Tauri를 실행합니다.

```sh
pnpm run dev
pnpm run tauri:dev
```

desktop bundle은 각 OS에서 빌드합니다.

```sh
pnpm run tauri:build
```

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
