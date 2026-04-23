import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Outconceive UI',
  description: 'The parallel strings web framework — no trees, no reconciliation',
  ignoreDeadLinks: true,

  themeConfig: {
    nav: [
      { text: 'Guide', link: '/v1.0/guide/getting-started' },
      { text: 'API', link: '/v1.0/api/markout' },
      { text: 'Architecture', link: '/v1.0/architecture/overview' },
      {
        text: 'v1.0',
        items: [
          { text: 'v1.0 (Latest)', link: '/v1.0/guide/getting-started', activeMatch: '/v1.0/' }
        ]
      }
    ],

    sidebar: {
      '/v1.0/guide/': [
        {
          text: 'Introduction',
          items: [
            { text: 'Getting Started', link: '/v1.0/guide/getting-started' },
            { text: 'Core Concepts', link: '/v1.0/guide/core-concepts' },
            { text: 'Markout Syntax', link: '/v1.0/guide/markout' },
          ]
        },
        {
          text: 'Features',
          items: [
            { text: 'Components', link: '/v1.0/guide/components' },
            { text: 'State & Reactivity', link: '/v1.0/guide/state' },
            { text: 'Routing', link: '/v1.0/guide/routing' },
            { text: 'Lists & Repeaters', link: '/v1.0/guide/lists' },
            { text: 'Theming', link: '/v1.0/guide/theming' },
          ]
        },
        {
          text: 'Advanced',
          items: [
            { text: 'Templates & Slots', link: '/v1.0/guide/templates' },
            { text: 'Multi-Mount', link: '/v1.0/guide/multi-mount' },
            { text: 'SSR & Hydration', link: '/v1.0/guide/ssr' },
            { text: 'Visual IDE', link: '/v1.0/guide/ide' },
          ]
        }
      ],
      '/v1.0/api/': [
        {
          text: 'Reference',
          items: [
            { text: 'Markout Syntax', link: '/v1.0/api/markout' },
            { text: 'JavaScript API', link: '/v1.0/api/javascript' },
            { text: 'WASM API', link: '/v1.0/api/wasm' },
            { text: 'CSS Reference', link: '/v1.0/api/css' },
          ]
        }
      ],
      '/v1.0/architecture/': [
        {
          text: 'Architecture',
          items: [
            { text: 'Overview', link: '/v1.0/architecture/overview' },
            { text: 'Parallel Strings', link: '/v1.0/architecture/parallel-strings' },
            { text: 'Why No Trees', link: '/v1.0/architecture/why-no-trees' },
            { text: 'Performance', link: '/v1.0/architecture/performance' },
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/outconceive-ui/outconceive' }
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2025-present Outconceive UI'
    },

    search: {
      provider: 'local'
    }
  }
})
