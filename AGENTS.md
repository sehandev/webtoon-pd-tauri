# AGENTS.md

## 언어

- 영어로 물어봐도 한글로 대답한다.
- IT 용어와 외래어는 영어로 유지한다.
- 코드나 주석은 번역하지 않고 원문 그대로 유지한다.

## Dev Server

- dev server는 직접 실행하지 않는다.
- dev server가 필요하면 `pnpm run dev`처럼 개발자가 실행할 명령어를 명확히 쓴다.
- 이미 실행 중인 dev server가 있으면 새 server나 다른 port를 띄우지 말고 기존 server를 그대로 사용한다.
- 기존 dev server가 있는지 불확실하거나 접속이 안 되면, 임의로 새로 띄우지 말고 개발자에게 사용할 URL 또는 실행 상태를 물어본다.

## TanStack Start RSC

이 template은 TanStack Start RSC를 활성화한다. `"use client"` directive는 RSC 환경에서 browser-only 코드가 들어가는 module의 맨 위에 붙인다.

### `"use client"`를 붙이는 경우

- `useState`, `useEffect`, `useRef`, `useReducer`, `useSyncExternalStore` 같은 React client hooks를 사용하는 component
- `onClick`, `onChange`, `onSubmit`, keyboard/mouse handler처럼 event handler가 필요한 component
- `window`, `document`, `localStorage`, `matchMedia`, `ResizeObserver` 같은 browser API를 사용하는 module
- Zustand store를 component에서 구독하는 UI
- TanStack Form UI
- Base UI처럼 interactive primitive를 감싼 component
- shadcn/ui interactive components: `button`, `dialog`, `dropdown-menu`, `select`, `tooltip`, `popover`, `sheet`, `tabs` 등

### `"use client"`를 붙이지 않는 경우

- pure server/render-only component
- data fetch 후 markup만 렌더링하는 component
- static layout, typography, card shell
- `@tanstack/react-start/rsc` helper로 render할 server component
- server-only code, env/binding/R2/Supabase service role 접근 코드

### Boundary 규칙

- `"use client"`를 붙인 파일과 그 파일이 import하는 하위 module은 client boundary가 된다.
- R2 binding, secret, server-only Supabase client는 client component 안으로 import하지 않는다.
- client component가 server data를 필요로 하면 route loader, server function, RSC helper를 통해 전달한다.
- shadcn/ui component를 추가할 때 `components.json`의 `rsc: true`를 유지해서 CLI가 client component에 directive를 붙일 수 있게 한다.
