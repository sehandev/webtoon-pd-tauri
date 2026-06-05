import { TanStackDevtools } from "@tanstack/react-devtools";
import { createRootRoute, HeadContent, Scripts } from "@tanstack/react-router";
import { TanStackRouterDevtoolsPanel } from "@tanstack/react-router-devtools";

import appCss from "../styles.css?url";

const SUIT_VARIABLE_STYLESHEET =
  "https://cdn.jsdelivr.net/gh/sun-typeface/SUIT@2/fonts/variable/woff2/SUIT-Variable.css";
const PRETENDARD_VARIABLE_STYLESHEET =
  "https://cdn.jsdelivr.net/gh/orioncactus/pretendard@v1.3.9/dist/web/variable/pretendardvariable.css";

export const Route = createRootRoute({
  head: () => ({
    meta: [
      {
        charSet: "utf-8",
      },
      {
        name: "viewport",
        content: "width=device-width, initial-scale=1",
      },
      {
        name: "application-name",
        content: "Ziward Frontend Template",
      },
      {
        name: "apple-mobile-web-app-title",
        content: "Ziward Frontend Template",
      },
      {
        title: "Ziward Frontend Template",
      },
    ],
    links: [
      {
        rel: "preload",
        href: SUIT_VARIABLE_STYLESHEET,
        as: "style",
      },
      {
        rel: "preload",
        href: PRETENDARD_VARIABLE_STYLESHEET,
        as: "style",
      },
      {
        rel: "stylesheet",
        href: SUIT_VARIABLE_STYLESHEET,
      },
      {
        rel: "stylesheet",
        href: PRETENDARD_VARIABLE_STYLESHEET,
      },
      {
        rel: "stylesheet",
        href: appCss,
      },
      {
        rel: "icon",
        type: "image/svg+xml",
        href: "/logo.svg",
      },
      {
        rel: "manifest",
        href: "/manifest.json",
      },
    ],
  }),
  notFoundComponent: () => (
    <main className="container mx-auto p-4 pt-16">
      <h1>404</h1>
      <p>The requested page could not be found.</p>
    </main>
  ),
  shellComponent: RootDocument,
});

function RootDocument({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <head>
        <HeadContent />
      </head>
      <body>
        {children}
        <TanStackDevtools
          config={{
            position: "top-left",
          }}
          plugins={[
            {
              name: "Tanstack Router",
              render: <TanStackRouterDevtoolsPanel />,
            },
          ]}
        />
        <Scripts />
      </body>
    </html>
  );
}
