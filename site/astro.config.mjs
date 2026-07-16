import { defineConfig } from 'astro/config';

// https://astro.build/config
export default defineConfig({
  site: 'https://SCP-00.github.io',
  base: '/athenas',
  output: 'static',
  build: {
    assets: '_assets',
  },
});
