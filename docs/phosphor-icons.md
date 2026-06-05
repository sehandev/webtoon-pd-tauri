# Phosphor Icons Convention

이 template은 icon library로 `@phosphor-icons/react`를 사용한다.

## Package

`@phosphor-icons/react`를 유지한다. Legacy package인 `phosphor-react`는 사용하지 않는다.

```tsx
import { HouseIcon } from "@phosphor-icons/react";
```

기본 import는 package root에서 named import로 한다. Development compile 최적화를 이유로 `@phosphor-icons/react/dist/csr/House` 같은 per-icon path import는 사용하지 않는다.

## Defaults

Application root에 `IconContext.Provider`를 두고 project 기본값을 설정한다.

```tsx
import { IconContext } from "@phosphor-icons/react";

function AppIconProvider({ children }: { children: React.ReactNode }) {
  return (
    <IconContext.Provider
      value={{
        color: "currentColor",
        size: "1em",
        weight: "regular",
        mirrored: false,
      }}
    >
      {children}
    </IconContext.Provider>
  );
}
```

Provider는 global default만 담당한다. 실제 크기, 색상, 상태 표현은 component 가까이에서 `className`과 prop으로 local styling한다.

```tsx
import { HouseIcon } from "@phosphor-icons/react";

function NavigationItem() {
  return <HouseIcon className="size-4 text-muted-foreground" />;
}
```

## Color

Icon color는 `currentColor`를 기준으로 한다. Hex value, rgb value, inline `color` prop을 직접 쓰지 않고 Tailwind text color를 상속시킨다.

```tsx
<HouseIcon className="size-4 text-foreground" />
<HouseIcon className="size-4 text-muted-foreground" />
```

## Size

Icon size는 `size` prop보다 Tailwind size class를 우선한다.

```tsx
<HouseIcon className="size-4" />
```

`size` prop은 외부 API가 number/string size를 요구하거나, class composition보다 prop 전달이 더 명확한 wrapper에서만 사용한다.

## Weight

Icon weight는 의미에 맞게 제한해서 사용한다.

- `regular`: default
- `fill`: selected, active, completed 같은 filled state
- `duotone`: product illustration 성격의 icon 또는 강조가 필요한 empty state
- `bold`: destructive, high-emphasis, toolbar action처럼 시각적 무게가 필요한 경우
- `thin`, `light`: 일반 UI control에서는 사용하지 않는다. 별도 visual direction이 있을 때만 사용한다.

```tsx
<HouseIcon className="size-4" weight="regular" />
<HouseIcon className="size-4 text-primary" weight="fill" />
```

## Accessibility

Decorative icon은 assistive technology에서 숨긴다.

```tsx
<HouseIcon aria-hidden className="size-4" />
```

Icon-only button은 icon이 아니라 button에 accessible name을 둔다.

```tsx
<button type="button" aria-label="Open navigation">
  <ListIcon aria-hidden className="size-4" />
</button>
```

의미 있는 non-interactive standalone icon은 `aria-label`을 icon에 직접 둘 수 있지만, 가능한 경우 주변 text나 owning control이 의미를 전달하게 한다.

## RTL

`mirrored`는 RTL 대응이 필요한 directional icon에만 사용한다. 현재 template의 `components.json`은 `rtl: false`이므로 기본값은 `false`다.

```tsx
<ArrowRightIcon mirrored={isRtl} className="size-4" />
```

## SSR And RSC

일반 client component와 일반 SSR render에서는 package root import를 사용한다.

```tsx
import { HouseIcon } from "@phosphor-icons/react";
```

React Server Components처럼 React Context API를 사용할 수 없는 위치에서는 `/ssr` entry를 사용한다. 이 entry는 `IconContext.Provider`의 default를 상속하지 않으므로 필요한 `className`, `weight`, accessibility prop을 해당 icon에 직접 전달한다.

```tsx
import { HouseIcon } from "@phosphor-icons/react/ssr";

export function ServerIcon() {
  return <HouseIcon aria-hidden className="size-4 text-muted-foreground" weight="regular" />;
}
```

## Avoid

전체 icon namespace import는 사용하지 않는다.

```tsx
// Do not use.
import * as Icon from "@phosphor-icons/react";
```

Icon 내부에 arbitrary SVG children을 넣는 composability pattern은 기본 UI에서는 사용하지 않는다. Animation, filter, custom background layer가 꼭 필요한 brand-specific icon에만 제한한다.

Custom icon을 `IconBase`로 감싸는 방식은 자체 icon set을 Phosphor의 weight/context system에 통합해야 할 때만 사용한다. 일반 asset은 `public/logo.svg`처럼 static asset으로 둔다.
